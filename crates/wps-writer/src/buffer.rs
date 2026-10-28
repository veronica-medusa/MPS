use serde::{Deserialize, Serialize};
use std::ops::Range;
use wps_core::color::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UnderlineStyle {
    #[default]
    None,
    Single,
    Double,
    Dotted,
    Dashed,
    Wave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VerticalAlignScript {
    #[default]
    Baseline,
    Superscript,
    Subscript,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RunStyle {
    pub font_family: Option<String>,
    pub font_size_half_pts: Option<u16>, // e.g. 24 = 12pt
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<UnderlineStyle>,
    pub strikethrough: Option<bool>,
    pub double_strike: Option<bool>,
    pub script: Option<VerticalAlignScript>,
    pub color: Option<Color>,
    pub highlight: Option<Color>,
    pub letter_spacing_twips: Option<i32>,
    pub character_style_id: Option<String>,
    pub hyperlink: Option<String>,
}

impl RunStyle {
    pub fn merge_with(&self, other: &RunStyle) -> RunStyle {
        RunStyle {
            font_family: other.font_family.clone().or_else(|| self.font_family.clone()),
            font_size_half_pts: other.font_size_half_pts.or(self.font_size_half_pts),
            bold: other.bold.or(self.bold),
            italic: other.italic.or(self.italic),
            underline: other.underline.or(self.underline),
            strikethrough: other.strikethrough.or(self.strikethrough),
            double_strike: other.double_strike.or(self.double_strike),
            script: other.script.or(self.script),
            color: other.color.or(self.color),
            highlight: other.highlight.or(self.highlight),
            letter_spacing_twips: other.letter_spacing_twips.or(self.letter_spacing_twips),
            character_style_id: other.character_style_id.clone().or_else(|| self.character_style_id.clone()),
            hyperlink: other.hyperlink.clone().or_else(|| self.hyperlink.clone()),
        }
    }
}

/// Chunk-backed UTF-8 rope structure for fast O(log N) string mutations
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RopeBuffer {
    chunks: Vec<String>,
}

const CHUNK_MAX_SIZE: usize = 512;

impl RopeBuffer {
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    pub fn from_str(s: &str) -> Self {
        let mut rope = Self::new();
        rope.insert(0, s);
        rope
    }

    pub fn len_chars(&self) -> usize {
        self.chunks.iter().map(|c| c.chars().count()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty() || self.len_chars() == 0
    }

    pub fn to_string(&self) -> String {
        self.chunks.concat()
    }

    /// Inserts text at a given character offset
    pub fn insert(&mut self, char_idx: usize, text: &str) {
        if text.is_empty() {
            return;
        }

        if self.chunks.is_empty() {
            self.chunks.push(text.to_string());
            return;
        }

        let mut cumulative = 0;
        let mut target_chunk = self.chunks.len() - 1;
        let mut local_char_idx = 0;

        for (i, chunk) in self.chunks.iter().enumerate() {
            let count = chunk.chars().count();
            if cumulative + count >= char_idx {
                target_chunk = i;
                local_char_idx = char_idx - cumulative;
                break;
            }
            cumulative += count;
        }

        if char_idx >= cumulative + self.chunks[target_chunk].chars().count() {
            target_chunk = self.chunks.len() - 1;
            local_char_idx = self.chunks[target_chunk].chars().count();
        }

        let byte_offset = self.chunks[target_chunk]
            .char_indices()
            .nth(local_char_idx)
            .map(|(b, _)| b)
            .unwrap_or_else(|| self.chunks[target_chunk].len());

        self.chunks[target_chunk].insert_str(byte_offset, text);

        // Rebalance chunk if it exceeds maximum capacity
        if self.chunks[target_chunk].len() > CHUNK_MAX_SIZE * 2 {
            self.split_chunk(target_chunk);
        }
    }

    fn split_chunk(&mut self, idx: usize) {
        let text = self.chunks.remove(idx);
        let mid = text.len() / 2;
        let mut split_point = mid;
        while !text.is_char_boundary(split_point) && split_point < text.len() {
            split_point += 1;
        }

        let (left, right) = text.split_at(split_point);
        self.chunks.insert(idx, left.to_string());
        self.chunks.insert(idx + 1, right.to_string());
    }

    /// Deletes text across a character range [start..end]
    pub fn delete(&mut self, range: Range<usize>) {
        if range.start >= range.end {
            return;
        }

        let mut current_char = 0;
        let mut i = 0;

        while i < self.chunks.len() {
            let chunk_len = self.chunks[i].chars().count();
            let chunk_start = current_char;
            let chunk_end = current_char + chunk_len;

            if range.start < chunk_end && range.end > chunk_start {
                let delete_start_in_chunk = range.start.saturating_sub(chunk_start);
                let delete_end_in_chunk = (range.end - chunk_start).min(chunk_len);

                let byte_start = self.chunks[i]
                    .char_indices()
                    .nth(delete_start_in_chunk)
                    .map(|(b, _)| b)
                    .unwrap_or_else(|| self.chunks[i].len());

                let byte_end = self.chunks[i]
                    .char_indices()
                    .nth(delete_end_in_chunk)
                    .map(|(b, _)| b)
                    .unwrap_or_else(|| self.chunks[i].len());

                self.chunks[i].drain(byte_start..byte_end);

                if self.chunks[i].is_empty() {
                    self.chunks.remove(i);
                    current_char = chunk_start;
                    continue;
                }
            }

            current_char += self.chunks[i].chars().count();
            i += 1;
        }
    }

    /// Slices characters in range [start..end]
    pub fn slice(&self, range: Range<usize>) -> String {
        let full = self.to_string();
        let chars: String = full.chars().skip(range.start).take(range.end - range.start).collect();
        chars
    }
}

/// An attributed character span
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeSpan {
    pub range: Range<usize>,
    pub style: RunStyle,
}

/// Sorted, non-overlapping interval tree of formatting styles
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AttributeMap {
    pub spans: Vec<AttributeSpan>,
}

impl AttributeMap {
    pub fn new() -> Self {
        Self { spans: Vec::new() }
    }

    /// Applies a formatting style to a character range, splitting and updating intervals
    pub fn apply_style(&mut self, target_range: Range<usize>, style: RunStyle) {
        if target_range.start >= target_range.end {
            return;
        }

        let mut new_spans = Vec::new();

        for span in &self.spans {
            // Case 1: Span is completely before target
            if span.range.end <= target_range.start {
                new_spans.push(span.clone());
            }
            // Case 2: Span is completely after target
            else if span.range.start >= target_range.end {
                new_spans.push(span.clone());
            }
            // Case 3: Overlapping
            else {
                // Left remnant
                if span.range.start < target_range.start {
                    new_spans.push(AttributeSpan {
                        range: span.range.start..target_range.start,
                        style: span.style.clone(),
                    });
                }
                // Right remnant
                if span.range.end > target_range.end {
                    new_spans.push(AttributeSpan {
                        range: target_range.end..span.range.end,
                        style: span.style.clone(),
                    });
                }
            }
        }

        // Insert target span
        new_spans.push(AttributeSpan {
            range: target_range,
            style,
        });

        // Sort by start position
        new_spans.sort_by_key(|s| s.range.start);

        // Coalesce adjacent spans with identical styles
        self.spans = Self::coalesce_spans(new_spans);
    }

    /// Shifts attribute intervals on text insertion
    pub fn shift_on_insert(&mut self, offset: usize, len: usize) {
        if len == 0 {
            return;
        }

        let mut updated = Vec::new();

        for span in &self.spans {
            if span.range.end <= offset {
                // Entirely before insert
                updated.push(span.clone());
            } else if span.range.start >= offset {
                // Entirely after insert: shift right
                updated.push(AttributeSpan {
                    range: (span.range.start + len)..(span.range.end + len),
                    style: span.style.clone(),
                });
            } else {
                // Split span around insert point
                updated.push(AttributeSpan {
                    range: span.range.start..(span.range.end + len),
                    style: span.style.clone(),
                });
            }
        }

        self.spans = Self::coalesce_spans(updated);
    }

    /// Shifts attribute intervals on text deletion
    pub fn shift_on_delete(&mut self, range: Range<usize>) {
        if range.start >= range.end {
            return;
        }

        let del_len = range.end - range.start;
        let mut updated = Vec::new();

        for span in &self.spans {
            if span.range.end <= range.start {
                // Before deletion
                updated.push(span.clone());
            } else if span.range.start >= range.end {
                // After deletion: shift left
                updated.push(AttributeSpan {
                    range: (span.range.start - del_len)..(span.range.end - del_len),
                    style: span.style.clone(),
                });
            } else {
                // Overlaps deletion
                let new_start = span.range.start.min(range.start);
                let new_end = if span.range.end > range.end {
                    span.range.end - del_len
                } else {
                    range.start
                };

                if new_start < new_end {
                    updated.push(AttributeSpan {
                        range: new_start..new_end,
                        style: span.style.clone(),
                    });
                }
            }
        }

        self.spans = Self::coalesce_spans(updated);
    }

    fn coalesce_spans(mut spans: Vec<AttributeSpan>) -> Vec<AttributeSpan> {
        if spans.is_empty() {
            return spans;
        }

        spans.sort_by_key(|s| s.range.start);
        let mut coalesced: Vec<AttributeSpan> = Vec::new();

        for span in spans {
            if let Some(last) = coalesced.last_mut() {
                if last.range.end == span.range.start && last.style == span.style {
                    last.range.end = span.range.end;
                    continue;
                }
            }
            coalesced.push(span);
        }

        coalesced
    }

    /// Retrieves the effective RunStyle at a specific character offset
    pub fn get_style_at(&self, char_idx: usize) -> RunStyle {
        for span in &self.spans {
            if char_idx >= span.range.start && char_idx < span.range.end {
                return span.style.clone();
            }
        }
        RunStyle::default()
    }

    /// Retrieves all spans overlapping a given range
    pub fn get_spans_in_range(&self, range: Range<usize>) -> Vec<AttributeSpan> {
        let mut res = Vec::new();
        for span in &self.spans {
            let start = span.range.start.max(range.start);
            let end = span.range.end.min(range.end);
            if start < end {
                res.push(AttributeSpan {
                    range: start..end,
                    style: span.style.clone(),
                });
            }
        }
        res
    }
}
