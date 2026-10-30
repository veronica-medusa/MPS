//! Spatial-Grounded `@PdfAI` RAG Agent (`wps-ai`)
//!
//! Implements:
//! 1. Spatial BM25 document indexing across `EditableParagraphBlock` and `ReconstructedTableGrid`.
//! 2. Multi-page token inverted index with term frequency (TF) and inverse document frequency (IDF).
//! 3. Grounded citation chips linking exact answers to `page_index` and `PdfRect` bounding boxes.

use crate::provider::{BuiltInLocalEngine, LlmClient, LlmPromptRequest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wps_pdf::layout_reconstructor::{
    EditableParagraphBlock, LayoutElement, PdfRect, ReconstructedPageLayout, ReconstructedTableGrid,
};

/// Grounded citation chip highlighting the exact spatial region in the PDF viewer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfCitationChip {
    pub citation_index: usize,
    pub page_index: usize,
    pub bbox: PdfRect,
    pub snippet_preview: String,
    pub block_id: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfIndexedBlock {
    pub page_index: usize,
    pub block_id: u64,
    pub bbox: PdfRect,
    pub text: String,
    pub token_freqs: HashMap<String, usize>,
    pub total_tokens: usize,
}

#[derive(Debug, Clone, Default)]
pub struct PdfSpatialIndex {
    pub blocks: Vec<PdfIndexedBlock>,
    pub doc_freqs: HashMap<String, usize>,
    pub total_block_tokens: usize,
}

impl PdfSpatialIndex {
    pub fn new() -> Self {
        Self::default()
    }

    fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| s.len() >= 2)
            .map(|s| s.to_string())
            .collect()
    }

    /// Indexes an editable paragraph block from layout reconstruction.
    pub fn index_paragraph(&mut self, page_index: usize, block: &EditableParagraphBlock) {
        let text = block.full_text();
        if text.trim().is_empty() {
            return;
        }

        let tokens = Self::tokenize(&text);
        let mut freqs = HashMap::new();
        for t in &tokens {
            *freqs.entry(t.clone()).or_insert(0) += 1;
        }

        for k in freqs.keys() {
            *self.doc_freqs.entry(k.clone()).or_insert(0) += 1;
        }

        self.total_block_tokens += tokens.len();
        self.blocks.push(PdfIndexedBlock {
            page_index,
            block_id: block.id,
            bbox: block.bbox,
            text,
            token_freqs: freqs,
            total_tokens: tokens.len(),
        });
    }

    /// Indexes a reconstructed table grid.
    pub fn index_table(&mut self, page_index: usize, table: &ReconstructedTableGrid) {
        let mut table_text = Vec::new();
        for cell in &table.cells {
            let t = cell.text();
            if !t.is_empty() {
                table_text.push(t);
            }
        }

        let combined = table_text.join(" | ");
        if combined.trim().is_empty() {
            return;
        }

        let tokens = Self::tokenize(&combined);
        let mut freqs = HashMap::new();
        for t in &tokens {
            *freqs.entry(t.clone()).or_insert(0) += 1;
        }

        for k in freqs.keys() {
            *self.doc_freqs.entry(k.clone()).or_insert(0) += 1;
        }

        self.total_block_tokens += tokens.len();
        self.blocks.push(PdfIndexedBlock {
            page_index,
            block_id: table.id,
            bbox: table.bbox,
            text: combined,
            token_freqs: freqs,
            total_tokens: tokens.len(),
        });
    }

    /// Bulk indexes a reconstructed page layout.
    pub fn index_reconstructed_layout(&mut self, page_index: usize, layout: &ReconstructedPageLayout) {
        for header in &layout.headers {
            self.index_paragraph(page_index, header);
        }
        for element in layout.reading_order_elements() {
            match element {
                LayoutElement::Paragraph(p) => self.index_paragraph(page_index, p),
                LayoutElement::Table(t) => self.index_table(page_index, t),
            }
        }
        for footer in &layout.footers {
            self.index_paragraph(page_index, footer);
        }
    }

    /// BM25 ranking over indexed PDF text blocks.
    pub fn query(&self, query_str: &str, top_k: usize) -> Vec<(usize, f64)> {
        let q_tokens = Self::tokenize(query_str);
        if q_tokens.is_empty() || self.blocks.is_empty() {
            return Vec::new();
        }

        let n = self.blocks.len() as f64;
        let avg_dl = (self.total_block_tokens as f64 / n).max(1.0);
        let k1 = 1.2;
        let b = 0.75;

        let mut scored: Vec<(usize, f64)> = self
            .blocks
            .iter()
            .enumerate()
            .map(|(idx, block)| {
                let mut score = 0.0;
                let dl = block.total_tokens as f64;

                for token in &q_tokens {
                    if let Some(&tf) = block.token_freqs.get(token) {
                        let df = *self.doc_freqs.get(token).unwrap_or(&1) as f64;
                        let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();
                        let tf_f = tf as f64;
                        let term_score = idf * (tf_f * (k1 + 1.0)) / (tf_f + k1 * (1.0 - b + b * (dl / avg_dl)));
                        score += term_score;
                    }
                }
                (idx, score)
            })
            .filter(|(_, s)| *s > 0.0)
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }
}

pub struct PdfAi;

impl PdfAi {
    /// Executes spatial RAG against indexed PDF layouts, returning response grounded with citation chips.
    pub fn query_pdf_rag(
        index: &PdfSpatialIndex,
        question: &str,
    ) -> (String, Vec<PdfCitationChip>) {
        let hits = index.query(question, 3);
        if hits.is_empty() {
            return (
                "No matching content found in the active PDF document.".to_string(),
                Vec::new(),
            );
        }

        let mut citations = Vec::new();
        let mut context_snippets = Vec::new();

        for (c_idx, &(block_idx, _score)) in hits.iter().enumerate() {
            let block = &index.blocks[block_idx];
            let chip_idx = c_idx + 1;
            let snippet = if block.text.len() > 120 {
                format!("{}...", &block.text[..120].trim())
            } else {
                block.text.clone()
            };

            citations.push(PdfCitationChip {
                citation_index: chip_idx,
                page_index: block.page_index,
                bbox: block.bbox,
                snippet_preview: snippet.clone(),
                block_id: block.block_id,
            });

            context_snippets.push(format!("[{}] (Page {}) {}", chip_idx, block.page_index + 1, block.text));
        }

        let engine = BuiltInLocalEngine::new();
        let context_str = context_snippets.join("\n\n");
        let req = LlmPromptRequest {
            system_prompt: "You are an expert PDF research analyst. Answer the user's question referencing citation tags like [1] or [2] where appropriate.".to_string(),
            user_prompt: format!(
                "Question: {}\n\nDocument Context:\n{}",
                question, context_str
            ),
            temperature: 0.2,
            max_tokens: 256,
        };

        let generated_text = match engine.generate_completion(&req) {
            Ok(res) if !res.content.is_empty() => {
                if res.content.contains('[') {
                    res.content
                } else {
                    format!("{} [1]", res.content.trim())
                }
            }
            _ => {
                format!(
                    "Based on the document context [1], the relevant section indicates: \"{}\".",
                    citations[0].snippet_preview
                )
            }
        };

        (generated_text, citations)
    }
}
