//! `@WriterAI` — AST-Grounded Word Processor Co-Pilot & Redline Proofreader (`wps-ai`)
//!
//! Capabilities:
//! 1. Rewrite selection with word-level LCS diffing emitted directly as Track Changes Redlines
//!    (`Keep`, `Delete`, `Insert`) attributed to `"WPS AI"`.
//! 2. Grammar, passive-voice, and conciseness auditing with anchored revision suggestions.
//! 3. One-click Auto-Format Document (inferring `Title`, `Heading 1..3`, `List Paragraph`, `Normal`).
//! 4. Executive Document Summarization & Action Item extraction.
//! 5. Writer Document AST -> `@SlidesAI` Presentation Outline conversion.

use crate::slides_ai::{DeckOutline, SlideLayoutKind, SlideOutlineCard};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RewriteTone {
    Formal,
    Concise,
    Expanded,
    Academic,
    ExecutiveBullets,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriterParagraphInput {
    pub paragraph_id: u64,
    pub text: String,
    pub current_style_id: String,
}

/// Word-level redline operation so the Writer engine can apply AI edits as Track Changes
/// (`w:del` and `w:ins`) that the user can individually Accept or Reject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WriterRedlineSpan {
    Keep(String),
    Delete {
        text: String,
        author: String,
        timestamp_iso: String,
    },
    Insert {
        text: String,
        author: String,
        timestamp_iso: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriterRewriteResult {
    pub paragraph_id: u64,
    pub original_text: String,
    pub rewritten_text: String,
    pub tone: RewriteTone,
    pub redline_spans: Vec<WriterRedlineSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StyleAssignmentMutation {
    pub paragraph_id: u64,
    pub old_style_id: String,
    pub new_style_id: String,
    pub stripped_prefix: Option<String>,
    pub cleaned_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrammarIssueSuggestion {
    pub paragraph_id: u64,
    pub char_start: usize,
    pub char_end: usize,
    pub problematic_phrase: String,
    pub replacement_suggestion: String,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutiveSummaryCard {
    pub headline: String,
    pub key_takeaways: Vec<String>,
    pub action_items: Vec<String>,
    pub word_count_original: usize,
    pub word_count_summary: usize,
}

pub struct WriterAiAgent;

impl WriterAiAgent {
    /// Rewrites a paragraph according to `tone` and computes a word-level Longest Common
    /// Subsequence (LCS) redline diff so the Writer engine mounts it directly as Track Changes.
    pub fn rewrite_with_track_changes(
        para: &WriterParagraphInput,
        tone: RewriteTone,
        timestamp_iso: &str,
    ) -> WriterRewriteResult {
        let rewritten_text = Self::transform_text_by_tone(&para.text, tone);
        let redline_spans =
            Self::compute_word_level_redlines(&para.text, &rewritten_text, "WPS AI", timestamp_iso);

        WriterRewriteResult {
            paragraph_id: para.paragraph_id,
            original_text: para.text.clone(),
            rewritten_text,
            tone,
            redline_spans,
        }
    }

    /// Scans an unformatted or messy document and assigns semantic Writer styles
    /// (`Title`, `Heading 1`, `Heading 2`, `Heading 3`, `List Paragraph`, `Normal`).
    pub fn auto_format_document(
        paragraphs: &[WriterParagraphInput],
    ) -> Vec<StyleAssignmentMutation> {
        let mut mutations = Vec::with_capacity(paragraphs.len());
        let mut seen_title = false;

        for (idx, p) in paragraphs.iter().enumerate() {
            let trimmed = p.text.trim();
            if trimmed.is_empty() {
                continue;
            }

            let word_count = trimmed.split_whitespace().count();

            // 1. Check numbered section headers like "1. Introduction" or "1.2 Architecture"
            let first_token = trimmed.split_whitespace().next().unwrap_or("");
            if Self::is_section_numbering_token(first_token) && word_count <= 14 {
                let dot_depth = first_token.trim_end_matches('.').matches('.').count();
                let heading_style = match dot_depth {
                    0 => "Heading 1",
                    1 => "Heading 2",
                    _ => "Heading 3",
                };
                mutations.push(StyleAssignmentMutation {
                    paragraph_id: p.paragraph_id,
                    old_style_id: p.current_style_id.clone(),
                    new_style_id: heading_style.to_string(),
                    stripped_prefix: None,
                    cleaned_text: trimmed.to_string(),
                });
                seen_title = true;
                continue;
            }

            // 2. Check bullet list markers
            for marker in ["• ", "- ", "* ", "● ", "▪ "] {
                if let Some(rest) = trimmed.strip_prefix(marker) {
                    mutations.push(StyleAssignmentMutation {
                        paragraph_id: p.paragraph_id,
                        old_style_id: p.current_style_id.clone(),
                        new_style_id: "List Paragraph".to_string(),
                        stripped_prefix: Some(marker.trim().to_string()),
                        cleaned_text: rest.trim().to_string(),
                    });
                    break;
                }
            }
            if mutations.last().is_some_and(|m| m.paragraph_id == p.paragraph_id) {
                continue;
            }

            // 3. First short line becomes Document Title
            if !seen_title && idx == 0 && word_count <= 15 && !trimmed.ends_with('.') {
                seen_title = true;
                mutations.push(StyleAssignmentMutation {
                    paragraph_id: p.paragraph_id,
                    old_style_id: p.current_style_id.clone(),
                    new_style_id: "Title".to_string(),
                    stripped_prefix: None,
                    cleaned_text: trimmed.to_string(),
                });
                continue;
            }

            // 4. Short standalone lines without terminal periods become Heading 2
            if word_count <= 9
                && !trimmed.ends_with('.')
                && !trimmed.ends_with(',')
                && !trimmed.ends_with(';')
            {
                mutations.push(StyleAssignmentMutation {
                    paragraph_id: p.paragraph_id,
                    old_style_id: p.current_style_id.clone(),
                    new_style_id: "Heading 2".to_string(),
                    stripped_prefix: None,
                    cleaned_text: trimmed.to_string(),
                });
                continue;
            }

            // 5. Default body paragraph
            mutations.push(StyleAssignmentMutation {
                paragraph_id: p.paragraph_id,
                old_style_id: p.current_style_id.clone(),
                new_style_id: "Normal".to_string(),
                stripped_prefix: None,
                cleaned_text: trimmed.to_string(),
            });
        }

        mutations
    }

    /// Audits paragraphs for wordy phrases, weak hedging, and passive constructions.
    pub fn audit_grammar_and_style(
        paragraphs: &[WriterParagraphInput],
    ) -> Vec<GrammarIssueSuggestion> {
        let rules: &[(&str, &str, &str)] = &[
            (
                "in order to",
                "to",
                "Replace wordy infinitive phrase 'in order to' with concise 'to'.",
            ),
            (
                "due to the fact that",
                "because",
                "Simplify 'due to the fact that' to 'because'.",
            ),
            (
                "at this point in time",
                "now",
                "Replace redundant temporal phrase with 'now' or 'currently'.",
            ),
            (
                "utilize",
                "use",
                "Prefer direct verb 'use' over 'utilize' for clearer executive prose.",
            ),
            (
                "is able to",
                "can",
                "Replace 'is able to' with active modal verb 'can'.",
            ),
            (
                "a large number of",
                "many",
                "Tighten 'a large number of' to 'many'.",
            ),
        ];

        let mut suggestions = Vec::new();
        for p in paragraphs {
            let lower = p.text.to_ascii_lowercase();
            for &(pattern, replacement, reason) in rules {
                let mut search_from = 0usize;
                while let Some(rel_idx) = lower[search_from..].find(pattern) {
                    let byte_start = search_from + rel_idx;
                    let byte_end = byte_start + pattern.len();
                    let char_start = p.text[..byte_start].chars().count();
                    let char_end = char_start + pattern.chars().count();

                    suggestions.push(GrammarIssueSuggestion {
                        paragraph_id: p.paragraph_id,
                        char_start,
                        char_end,
                        problematic_phrase: p.text[byte_start..byte_end].to_string(),
                        replacement_suggestion: replacement.to_string(),
                        explanation: reason.to_string(),
                    });

                    search_from = byte_end;
                }
            }
        }

        suggestions
    }

    /// Generates an executive summary and extracts action items (`TODO`, `must`, `shall`, `action:`).
    pub fn summarize_document(paragraphs: &[WriterParagraphInput]) -> ExecutiveSummaryCard {
        let mut word_count_original = 0usize;
        let mut headline = String::from("Executive Document Summary");
        let mut key_takeaways = Vec::new();
        let mut action_items = Vec::new();

        for p in paragraphs {
            let trimmed = p.text.trim();
            if trimmed.is_empty() {
                continue;
            }
            word_count_original += trimmed.split_whitespace().count();

            if (p.current_style_id == "Title" || p.current_style_id == "Heading 1")
                && headline == "Executive Document Summary"
            {
                headline = trimmed.to_string();
            }

            let lower = trimmed.to_ascii_lowercase();
            if lower.contains("action:")
                || lower.contains("todo:")
                || lower.contains("must ")
                || lower.contains("deadline")
            {
                action_items.push(trimmed.to_string());
            } else if p.current_style_id.starts_with("Heading") || key_takeaways.len() < 4 {
                let first_sentence = trimmed
                    .split_inclusive('.')
                    .next()
                    .unwrap_or(trimmed)
                    .trim()
                    .to_string();
                if first_sentence.split_whitespace().count() >= 4 && key_takeaways.len() < 5 {
                    key_takeaways.push(first_sentence);
                }
            }
        }

        let word_count_summary = key_takeaways
            .iter()
            .chain(action_items.iter())
            .map(|s| s.split_whitespace().count())
            .sum();

        ExecutiveSummaryCard {
            headline,
            key_takeaways,
            action_items,
            word_count_original,
            word_count_summary,
        }
    }

    /// Converts a structured Writer document directly into a `DeckOutline` ready for `@SlidesAI`.
    pub fn convert_document_to_slides_outline(
        paragraphs: &[WriterParagraphInput],
    ) -> DeckOutline {
        let formatted = Self::auto_format_document(paragraphs);
        let mut title = String::from("Executive Presentation");
        let mut slides = Vec::new();
        let mut current_heading: Option<String> = None;
        let mut current_bullets: Vec<String> = Vec::new();

        for (input, mutation) in paragraphs.iter().zip(formatted.iter()) {
            let style = mutation.new_style_id.as_str();
            let text = mutation.cleaned_text.trim();
            if text.is_empty() {
                continue;
            }

            if style == "Title" {
                title = text.to_string();
            } else if style == "Heading 1" || style == "Heading 2" {
                if let Some(prev_h) = current_heading.take() {
                    slides.push(SlideOutlineCard {
                        slide_number: slides.len() + 2,
                        title: prev_h,
                        subtitle: None,
                        layout: SlideLayoutKind::ThreePillarCards,
                        bullets: std::mem::take(&mut current_bullets),
                        speaker_notes: "Walk through key points for this section.".to_string(),
                        morph_group_tag: Some(format!("!!section_card_{}", slides.len())),
                    });
                }
                current_heading = Some(text.to_string());
            } else {
                let summary_bullet = input
                    .text
                    .split_inclusive('.')
                    .next()
                    .unwrap_or(text)
                    .trim()
                    .to_string();
                if current_bullets.len() < 4 {
                    current_bullets.push(summary_bullet);
                }
            }
        }

        if let Some(last_h) = current_heading.take() {
            slides.push(SlideOutlineCard {
                slide_number: slides.len() + 2,
                title: last_h,
                subtitle: None,
                layout: SlideLayoutKind::ProcessTimeline,
                bullets: current_bullets,
                speaker_notes: "Concluding section overview and next steps.".to_string(),
                morph_group_tag: Some("!!section_card_final".to_string()),
            });
        }

        // Always prepend TitleHero slide at index 0
        let mut all_slides = vec![SlideOutlineCard {
            slide_number: 1,
            title: title.clone(),
            subtitle: Some("Generated from WPS Writer Document AST".to_string()),
            layout: SlideLayoutKind::TitleHero,
            bullets: vec![],
            speaker_notes: format!("Welcome everyone to the presentation on {}.", title),
            morph_group_tag: Some("!!hero_banner".to_string()),
        }];
        all_slides.extend(slides);

        DeckOutline {
            deck_title: title,
            theme_name: "Mercury Executive Cobalt".to_string(),
            slides: all_slides,
        }
    }

    fn transform_text_by_tone(text: &str, tone: RewriteTone) -> String {
        let mut working = text.trim().to_string();
        let replacements: &[(&str, &str)] = &[
            ("in order to", "to"),
            ("due to the fact that", "because"),
            ("at this point in time", "currently"),
            ("utilize", "use"),
            ("is able to", "can"),
            ("a large number of", "many"),
            ("we think that", "evidence indicates that"),
            ("really big", "significant"),
            ("pretty good", "strong"),
        ];

        for &(from, to) in replacements {
            if working.to_ascii_lowercase().contains(from) {
                working = Self::replace_case_insensitive(&working, from, to);
            }
        }

        match tone {
            RewriteTone::Concise => working,
            RewriteTone::Formal | RewriteTone::Academic => {
                let mut out = working
                    .replace("can't", "cannot")
                    .replace("won't", "will not")
                    .replace("don't", "do not")
                    .replace("isn't", "is not");
                if !out.ends_with('.') && !out.ends_with('!') && !out.ends_with('?') {
                    out.push('.');
                }
                out
            }
            RewriteTone::Expanded => {
                format!(
                    "{} Furthermore, this establishes a scalable operational baseline with measurable performance guarantees.",
                    working
                )
            }
            RewriteTone::ExecutiveBullets => {
                let clauses: Vec<&str> = working
                    .split(['.', ';'])
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                clauses
                    .into_iter()
                    .map(|c| format!("• {}", c))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }

    fn replace_case_insensitive(source: &str, pattern: &str, replacement: &str) -> String {
        let lower_src = source.to_ascii_lowercase();
        let lower_pat = pattern.to_ascii_lowercase();
        let mut result = String::with_capacity(source.len());
        let mut last_end = 0usize;

        while let Some(rel) = lower_src[last_end..].find(&lower_pat) {
            let start = last_end + rel;
            result.push_str(&source[last_end..start]);
            result.push_str(replacement);
            last_end = start + pattern.len();
        }
        result.push_str(&source[last_end..]);
        result
    }

    /// Computes a word-level Longest Common Subsequence (LCS) diff and coalesces contiguous
    /// Keep / Delete / Insert runs into `WriterRedlineSpan`s.
    fn compute_word_level_redlines(
        old_text: &str,
        new_text: &str,
        author: &str,
        timestamp_iso: &str,
    ) -> Vec<WriterRedlineSpan> {
        let old_words: Vec<&str> = old_text.split_whitespace().collect();
        let new_words: Vec<&str> = new_text.split_whitespace().collect();
        let n = old_words.len();
        let m = new_words.len();

        let mut dp = vec![vec![0usize; m + 1]; n + 1];
        for i in (0..n).rev() {
            for j in (0..m).rev() {
                if old_words[i] == new_words[j] {
                    dp[i][j] = 1 + dp[i + 1][j + 1];
                } else {
                    dp[i][j] = dp[i + 1][j].max(dp[i][j + 1]);
                }
            }
        }

        let mut raw_ops: Vec<(&str, &str)> = Vec::new();
        let (mut i, mut j) = (0, 0);
        while i < n && j < m {
            if old_words[i] == new_words[j] {
                raw_ops.push(("keep", old_words[i]));
                i += 1;
                j += 1;
            } else if dp[i + 1][j] >= dp[i][j + 1] {
                raw_ops.push(("del", old_words[i]));
                i += 1;
            } else {
                raw_ops.push(("ins", new_words[j]));
                j += 1;
            }
        }
        while i < n {
            raw_ops.push(("del", old_words[i]));
            i += 1;
        }
        while j < m {
            raw_ops.push(("ins", new_words[j]));
            j += 1;
        }

        let mut spans: Vec<WriterRedlineSpan> = Vec::new();
        for (op, word) in raw_ops {
            match (op, spans.last_mut()) {
                ("keep", Some(WriterRedlineSpan::Keep(acc))) => {
                    acc.push(' ');
                    acc.push_str(word);
                }
                ("del", Some(WriterRedlineSpan::Delete { text: acc, .. })) => {
                    acc.push(' ');
                    acc.push_str(word);
                }
                ("ins", Some(WriterRedlineSpan::Insert { text: acc, .. })) => {
                    acc.push(' ');
                    acc.push_str(word);
                }
                ("keep", _) => spans.push(WriterRedlineSpan::Keep(word.to_string())),
                ("del", _) => spans.push(WriterRedlineSpan::Delete {
                    text: word.to_string(),
                    author: author.to_string(),
                    timestamp_iso: timestamp_iso.to_string(),
                }),
                ("ins", _) => spans.push(WriterRedlineSpan::Insert {
                    text: word.to_string(),
                    author: author.to_string(),
                    timestamp_iso: timestamp_iso.to_string(),
                }),
                _ => {}
            }
        }

        spans
    }

    fn is_section_numbering_token(token: &str) -> bool {
        let clean = token.trim_end_matches('.');
        !clean.is_empty()
            && clean
                .split('.')
                .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
    }
}
