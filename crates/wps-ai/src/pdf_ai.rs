//! `@PdfAI` — Hybrid BM25 + Vector Spatial BBox RAG Engine (`wps-ai`)
//!
//! Indexes every reconstructed paragraph and table on a PDF page along with its exact
//! `(page_index, PdfRect)` bounding box. Answers user questions with grounded inline
//! `[1]`, `[2]` citations and `PdfCitationChip` records that scroll to and highlight
//! the exact source region on the PDF canvas.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Bounding box in PDF page points (`(0, 0)` at top-left).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PdfBBox {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl PdfBBox {
    pub fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self { x0, y0, x1, y1 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PdfChunkKind {
    TitleOrHeading,
    BodyParagraph,
    TableGrid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatialPdfChunk {
    pub block_id: u64,
    pub page_index: usize,
    pub bbox: PdfBBox,
    pub kind: PdfChunkKind,
    pub text: String,
    token_frequencies: BTreeMap<String, usize>,
    token_count: usize,
    dense_embedding: Vec<f32>,
}

/// Clickable citation badge returned to the Mercury UI that scrolls the PDF viewport
/// to `page_index` and highlights `bbox` in Mercury Violet (`#7048E8`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfCitationChip {
    pub citation_index: usize, // 1-based (`[1]`, `[2]`)
    pub block_id: u64,
    pub page_index: usize,
    pub bbox: PdfBBox,
    pub relevance_score: f32,
    pub snippet_preview: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfGroundedAnswer {
    pub query: String,
    pub synthesized_markdown: String,
    pub citations: Vec<PdfCitationChip>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PdfSpatialRagIndex {
    chunks: Vec<SpatialPdfChunk>,
    doc_freq: BTreeMap<String, usize>,
    avg_chunk_len: f32,
}

impl PdfSpatialRagIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Indexes a reconstructed paragraph or table block with its exact page and `PdfBBox`.
    pub fn add_spatial_chunk(
        &mut self,
        block_id: u64,
        page_index: usize,
        bbox: PdfBBox,
        kind: PdfChunkKind,
        text: String,
    ) {
        let tokens = Self::tokenize(&text);
        let token_count = tokens.len().max(1);
        let mut token_frequencies = BTreeMap::new();
        for tok in tokens {
            *token_frequencies.entry(tok).or_insert(0) += 1;
        }

        for unique_tok in token_frequencies.keys() {
            *self.doc_freq.entry(unique_tok.clone()).or_insert(0) += 1;
        }

        let dense_embedding = Self::compute_ngram_embedding(&text);

        self.chunks.push(SpatialPdfChunk {
            block_id,
            page_index,
            bbox,
            kind,
            text,
            token_frequencies,
            token_count,
            dense_embedding,
        });

        let total_tokens: usize = self.chunks.iter().map(|c| c.token_count).sum();
        self.avg_chunk_len = (total_tokens as f32) / (self.chunks.len().max(1) as f32);
    }

    /// Runs hybrid BM25 (`k1 = 1.5, b = 0.75`) + 64-dim dense cosine similarity retrieval
    /// and synthesizes a grounded response with clickable `[Page X, BBox]` citation chips.
    pub fn query_with_bbox_citations(&self, query: &str, top_k: usize) -> PdfGroundedAnswer {
        if self.chunks.is_empty() {
            return PdfGroundedAnswer {
                query: query.to_string(),
                synthesized_markdown: "No PDF content indexed.".to_string(),
                citations: vec![],
            };
        }

        let query_tokens = Self::tokenize(query);
        let query_embedding = Self::compute_ngram_embedding(query);
        let total_docs = self.chunks.len() as f32;

        let mut scored: Vec<(f32, &SpatialPdfChunk)> = Vec::with_capacity(self.chunks.len());

        for chunk in &self.chunks {
            // 1. Okapi BM25 lexical score
            let mut bm25 = 0.0f32;
            let k1 = 1.5f32;
            let b = 0.75f32;

            for q_tok in &query_tokens {
                let tf = *chunk.token_frequencies.get(q_tok).unwrap_or(&0) as f32;
                if tf > 0.0 {
                    let df = *self.doc_freq.get(q_tok).unwrap_or(&1) as f32;
                    let idf = ((total_docs - df + 0.5) / (df + 0.5) + 1.0).ln();
                    let norm_len = (chunk.token_count as f32) / self.avg_chunk_len.max(1.0);
                    let denom = tf + k1 * (1.0 - b + b * norm_len);
                    bm25 += idf * ((tf * (k1 + 1.0)) / denom);
                }
            }

            // 2. Character 3-gram hashed cosine similarity (captures morphological & numeric matches)
            let cosine = Self::cosine_similarity(&query_embedding, &chunk.dense_embedding);

            // 3. Structural role boost (tables and headings often carry high-signal facts)
            let role_boost = match chunk.kind {
                PdfChunkKind::TableGrid => 1.18,
                PdfChunkKind::TitleOrHeading => 1.10,
                PdfChunkKind::BodyParagraph => 1.0,
            };

            let combined_score = (bm25 * 0.75 + cosine * 2.5) * role_boost;
            if combined_score > 0.01 {
                scored.push((combined_score, chunk));
            }
        }

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let k = top_k.clamp(1, 5);
        let mut citations = Vec::new();
        let mut markdown_lines = Vec::new();

        for (idx, (score, chunk)) in scored.into_iter().take(k).enumerate() {
            let citation_num = idx + 1;
            let snippet = if chunk.text.chars().count() > 140 {
                let prefix: String = chunk.text.chars().take(140).collect();
                format!("{}...", prefix)
            } else {
                chunk.text.clone()
            };

            citations.push(PdfCitationChip {
                citation_index: citation_num,
                block_id: chunk.block_id,
                page_index: chunk.page_index,
                bbox: chunk.bbox,
                relevance_score: score,
                snippet_preview: snippet,
            });

            markdown_lines.push(format!(
                "- {} **[{}]** *(Page {}, BBox `[{:.0}, {:.0}, {:.0}, {:.0}]`)*",
                chunk.text.trim(),
                citation_num,
                chunk.page_index + 1,
                chunk.bbox.x0,
                chunk.bbox.y0,
                chunk.bbox.x1,
                chunk.bbox.y1
            ));
        }

        let synthesized_markdown = if markdown_lines.is_empty() {
            "No matching sections found in the PDF for your query.".to_string()
        } else {
            format!(
                "Based on the indexed PDF layout blocks:\n\n{}",
                markdown_lines.join("\n")
            )
        };

        PdfGroundedAnswer {
            query: query.to_string(),
            synthesized_markdown,
            citations,
        }
    }

    fn tokenize(text: &str) -> Vec<String> {
        let stop_words = [
            "the", "and", "for", "that", "this", "with", "from", "what", "are", "was", "were",
            "into", "how", "much", "does",
        ];
        text.to_ascii_lowercase()
            .split(|c: char| !c.is_ascii_alphanumeric() && c != '$' && c != '%')
            .filter(|t| t.len() >= 2 && !stop_words.contains(t))
            .map(|t| t.to_string())
            .collect()
    }

    /// Computes a unit-normalized 64-dimensional character 3-gram feature embedding
    /// for fast, zero-dependency semantic & fuzzy token similarity.
    fn compute_ngram_embedding(text: &str) -> Vec<f32> {
        let mut vec = vec![0.0f32; 64];
        let clean: Vec<char> = text
            .to_ascii_lowercase()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
            .collect();

        if clean.is_empty() {
            return vec;
        }

        for window in clean.windows(3) {
            let mut h: u64 = 0xcbf29ce484222325;
            for &ch in window {
                h ^= ch as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
            let bucket = (h as usize) % 64;
            let sign = if ((h >> 32) & 1) == 0 { 1.0 } else { -1.0 };
            vec[bucket] += sign;
        }

        let norm = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-6 {
            for v in &mut vec {
                *v /= norm;
            }
        }
        vec
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| x * y)
            .sum::<f32>()
            .clamp(0.0, 1.0)
    }
}
