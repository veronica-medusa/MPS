//! PDF AcroForms, Digital Signatures, True Permanent Redaction & AES-256 Security (`wps-pdf`)
//!
//! Implements:
//! 1. True Permanent Redaction: Purges intersecting characters from `PageScene` glyph runs,
//!    overwrites byte ranges inside raw PDF content streams, zeroes intersecting RGBA pixels
//!    inside embedded raster images, and scrubs metadata/annotations/bookmarks.
//! 2. Regex PII Redaction Scanner: Computes character-level sub-bounding boxes for SSNs,
//!    emails, credit cards, phone numbers, or custom patterns.
//! 3. Interactive AcroForms (`/AcroForm`): Text, Checkbox, Radio, Combo/List, and Signature
//!    fields with JSON import/export and permanent content-stream flattening.
//! 4. Cryptographic Digital Signatures: `/ByteRange` SHA-256 digest signing and tamper verification.
//! 5. ISO 32000-2 Revision 5/6 AES-256 Encryption & Permission enforcement (`User` & `Owner` keys).

use crate::layout_reconstructor::{
    PageScene, PdfRect, PositionedGlyphRun, RgbaColor, VectorFilledRect,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ============================================================================
// 1. DOCUMENT SECURITY & REDACTION DATA MODELS
// ============================================================================

/// Represents an embedded raster image on a PDF page with mutable pixel data
/// so redactions permanently destroy underlying pixel regions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfRasterImage {
    pub object_id: u64,
    pub page_index: usize,
    pub bbox: PdfRect,
    pub width_px: u32,
    pub height_px: u32,
    /// Row-major RGBA8 pixel buffer (`width_px * height_px * 4` bytes).
    pub rgba_pixels: Vec<u8>,
}

/// Represents a PDF annotation entry that may also need scrubbing during redaction
/// or baking during form/annotation flattening.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfAnnotationRecord {
    pub id: u64,
    pub page_index: usize,
    pub bbox: PdfRect,
    pub author: String,
    pub contents: String,
    pub color: RgbaColor,
}

/// Complete mutable PDF container passed to the redaction, form, and encryption engines.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PdfSecurityDocument {
    pub pages: Vec<PageScene>,
    /// Raw decompressed content stream bytes (`BT ... ET`) per page.
    pub raw_content_streams: Vec<Vec<u8>>,
    pub images: Vec<PdfRasterImage>,
    pub annotations: Vec<PdfAnnotationRecord>,
    pub form_fields: Vec<AcroFormField>,
    pub signatures: Vec<DigitalSignatureRecord>,
    pub metadata: BTreeMap<String, String>,
    pub bookmarks: Vec<(String, usize, f32)>,
    pub encryption: Option<EncryptionHeader>,
}

/// A proposed redaction zone before permanent application.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RedactionProposal {
    pub id: u64,
    pub page_index: usize,
    pub rect: PdfRect,
    pub fill_color: RgbaColor,
    pub overlay_text: Option<String>,
    pub overlay_text_color: RgbaColor,
}

/// Audit log returned after executing `apply_redactions_permanently`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionAuditReport {
    pub proposals_applied: usize,
    pub purged_glyph_runs: usize,
    pub split_partial_glyph_runs: usize,
    pub redacted_characters: usize,
    pub scrubbed_stream_bytes: usize,
    pub scrubbed_image_pixels: usize,
    pub removed_annotations: usize,
    pub scrubbed_metadata_entries: usize,
}

/// Built-in PII patterns for one-click "Search & Redact".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RedactionPattern {
    UsSsn,
    CreditCard,
    EmailAddress,
    PhoneNumber,
    CustomSubstring(String),
}

// ============================================================================
// 2. TRUE PERMANENT BYTE-STRIPPING REDACTION ENGINE
// ============================================================================

pub struct RedactionEngine;

impl RedactionEngine {
    /// Scans all pages for matches of `pattern`, computes exact character-level sub-bounding
    /// boxes for each matched span, and returns ready-to-review `RedactionProposal` items.
    pub fn search_and_propose_redactions(
        doc: &PdfSecurityDocument,
        pattern: &RedactionPattern,
        overlay_label: Option<String>,
        next_proposal_id: &mut u64,
    ) -> Vec<RedactionProposal> {
        let mut proposals = Vec::new();

        for page in &doc.pages {
            for run in &page.glyph_runs {
                let matched_char_ranges = Self::find_pattern_char_ranges(&run.text, pattern);
                if matched_char_ranges.is_empty() {
                    continue;
                }

                let total_chars = run.text.chars().count().max(1) as f32;
                let char_width = run.bbox.width() / total_chars;

                for (start_char, end_char) in matched_char_ranges {
                    let sub_x0 = run.bbox.x0 + (start_char as f32) * char_width;
                    let sub_x1 = run.bbox.x0 + (end_char as f32) * char_width;
                    let rect = PdfRect::new(sub_x0, run.bbox.y0, sub_x1, run.bbox.y1);

                    let id = *next_proposal_id;
                    *next_proposal_id += 1;

                    proposals.push(RedactionProposal {
                        id,
                        page_index: page.page_index,
                        rect,
                        fill_color: RgbaColor { r: 0, g: 0, b: 0, a: 255 },
                        overlay_text: overlay_label.clone(),
                        overlay_text_color: RgbaColor {
                            r: 255,
                            g: 255,
                            b: 255,
                            a: 255,
                        },
                    });
                }
            }
        }

        proposals
    }

    /// Permanently destroys all underlying glyphs, content stream bytes, raster image pixels,
    /// annotations, bookmarks, and metadata strings intersecting `proposals`.
    pub fn apply_redactions_permanently(
        doc: &mut PdfSecurityDocument,
        proposals: &[RedactionProposal],
    ) -> RedactionAuditReport {
        let mut report = RedactionAuditReport {
            proposals_applied: proposals.len(),
            ..Default::default()
        };

        if proposals.is_empty() {
            return report;
        }

        let mut purged_strings: Vec<String> = Vec::new();
        let mut next_run_id = doc
            .pages
            .iter()
            .flat_map(|p| p.glyph_runs.iter().map(|r| r.id))
            .max()
            .unwrap_or(1000)
            + 1;

        // 1. Purge & split glyph runs and scrub raw content streams per page
        for page_idx in 0..doc.pages.len() {
            let page_proposals: Vec<&RedactionProposal> = proposals
                .iter()
                .filter(|p| p.page_index == page_idx)
                .collect();

            if page_proposals.is_empty() {
                continue;
            }

            let original_runs = std::mem::take(&mut doc.pages[page_idx].glyph_runs);
            let mut surviving_runs: Vec<PositionedGlyphRun> = Vec::new();

            for run in original_runs {
                let intersecting: Vec<&RedactionProposal> = page_proposals
                    .iter()
                    .copied()
                    .filter(|prop| prop.rect.intersects(&run.bbox))
                    .collect();

                if intersecting.is_empty() {
                    surviving_runs.push(run);
                    continue;
                }

                // Scrub raw content stream bytes if byte range is recorded
                if let Some((b_start, b_end)) = run.stream_byte_range {
                    if let Some(stream) = doc.raw_content_streams.get_mut(page_idx) {
                        let safe_end = b_end.min(stream.len());
                        if b_start < safe_end {
                            for byte in &mut stream[b_start..safe_end] {
                                *byte = b' ';
                            }
                            report.scrubbed_stream_bytes += safe_end - b_start;
                        }
                    }
                }

                // Perform character-level spatial culling so partial-word redactions keep
                // non-overlapping prefix/suffix characters while destroying overlapping chars
                let chars: Vec<char> = run.text.chars().collect();
                let char_count = chars.len().max(1);
                let char_w = run.bbox.width() / (char_count as f32);

                let mut kept_segments: Vec<(usize, usize, String)> = Vec::new();
                let mut current_seg_start: Option<usize> = None;
                let mut current_seg_text = String::new();
                let mut removed_in_run = String::new();

                for (c_idx, &ch) in chars.iter().enumerate() {
                    let c_x0 = run.bbox.x0 + (c_idx as f32) * char_w;
                    let c_x1 = c_x0 + char_w;
                    let char_rect = PdfRect::new(c_x0, run.bbox.y0, c_x1, run.bbox.y1);

                    // Check if the character center or >35% of its width intersects any redaction box
                    let is_redacted = intersecting.iter().any(|prop| {
                        prop.rect.contains_point(char_rect.center_x(), char_rect.center_y(), 0.0)
                            || (prop.rect.intersects(&char_rect)
                                && prop.rect.horizontal_overlap_ratio(&char_rect) >= 0.35)
                    });

                    if is_redacted {
                        report.redacted_characters += 1;
                        removed_in_run.push(ch);
                        if let Some(seg_start) = current_seg_start.take() {
                            kept_segments.push((
                                seg_start,
                                c_idx,
                                std::mem::take(&mut current_seg_text),
                            ));
                        }
                    } else {
                        if current_seg_start.is_none() {
                            current_seg_start = Some(c_idx);
                        }
                        current_seg_text.push(ch);
                    }
                }

                if let Some(seg_start) = current_seg_start.take() {
                    kept_segments.push((seg_start, chars.len(), current_seg_text));
                }

                let trimmed_removed = removed_in_run.trim().to_string();
                if trimmed_removed.len() >= 3 {
                    purged_strings.push(trimmed_removed);
                }

                if kept_segments.is_empty() {
                    report.purged_glyph_runs += 1;
                } else {
                    report.split_partial_glyph_runs += 1;
                    for (seg_start, seg_end, seg_text) in kept_segments {
                        if seg_text.trim().is_empty() {
                            continue;
                        }
                        let seg_x0 = run.bbox.x0 + (seg_start as f32) * char_w;
                        let seg_x1 = run.bbox.x0 + (seg_end as f32) * char_w;
                        surviving_runs.push(PositionedGlyphRun {
                            id: {
                                let id = next_run_id;
                                next_run_id += 1;
                                id
                            },
                            text: seg_text,
                            bbox: PdfRect::new(seg_x0, run.bbox.y0, seg_x1, run.bbox.y1),
                            baseline_y: run.baseline_y,
                            font_name: run.font_name.clone(),
                            font_size_pt: run.font_size_pt,
                            is_bold: run.is_bold,
                            is_italic: run.is_italic,
                            color: run.color,
                            stream_byte_range: None,
                        });
                    }
                }
            }

            // Paint opaque redaction boxes and optional overlay codes onto the page scene
            for prop in &page_proposals {
                doc.pages[page_idx].filled_rects.push(VectorFilledRect {
                    rect: prop.rect,
                    fill_color: prop.fill_color,
                });

                if let Some(overlay) = &prop.overlay_text {
                    if !overlay.is_empty() {
                        let font_sz = (prop.rect.height() * 0.65).clamp(6.0, 11.0);
                        surviving_runs.push(PositionedGlyphRun {
                            id: {
                                let id = next_run_id;
                                next_run_id += 1;
                                id
                            },
                            text: overlay.clone(),
                            bbox: prop.rect,
                            baseline_y: prop.rect.y1 - (prop.rect.height() * 0.22),
                            font_name: "Helvetica-Bold".to_string(),
                            font_size_pt: font_sz,
                            is_bold: true,
                            is_italic: false,
                            color: prop.overlay_text_color,
                            stream_byte_range: None,
                        });
                    }
                }
            }

            doc.pages[page_idx].glyph_runs = surviving_runs;

            // Rebuild canonical PDF content stream for the page so no residual text operators remain
            let rebuilt_stream = Self::serialize_page_content_stream(&doc.pages[page_idx]);
            if page_idx < doc.raw_content_streams.len() {
                doc.raw_content_streams[page_idx] = rebuilt_stream;
            } else {
                doc.raw_content_streams.push(rebuilt_stream);
            }
        }

        // 2. Permanently scrub intersecting RGBA pixels in embedded raster images
        for image in &mut doc.images {
            for prop in proposals.iter().filter(|p| p.page_index == image.page_index) {
                if !prop.rect.intersects(&image.bbox) {
                    continue;
                }

                let img_w = image.bbox.width().max(1.0);
                let img_h = image.bbox.height().max(1.0);

                let norm_x0 = ((prop.rect.x0 - image.bbox.x0) / img_w).clamp(0.0, 1.0);
                let norm_x1 = ((prop.rect.x1 - image.bbox.x0) / img_w).clamp(0.0, 1.0);
                let norm_y0 = ((prop.rect.y0 - image.bbox.y0) / img_h).clamp(0.0, 1.0);
                let norm_y1 = ((prop.rect.y1 - image.bbox.y0) / img_h).clamp(0.0, 1.0);

                let px_x0 = (norm_x0 * (image.width_px as f32)).floor() as u32;
                let px_x1 = (norm_x1 * (image.width_px as f32)).ceil() as u32;
                let px_y0 = (norm_y0 * (image.height_px as f32)).floor() as u32;
                let px_y1 = (norm_y1 * (image.height_px as f32)).ceil() as u32;

                for py in px_y0..px_y1.min(image.height_px) {
                    for px in px_x0..px_x1.min(image.width_px) {
                        let offset = ((py * image.width_px + px) * 4) as usize;
                        if offset + 3 < image.rgba_pixels.len() {
                            image.rgba_pixels[offset] = prop.fill_color.r;
                            image.rgba_pixels[offset + 1] = prop.fill_color.g;
                            image.rgba_pixels[offset + 2] = prop.fill_color.b;
                            image.rgba_pixels[offset + 3] = 255;
                            report.scrubbed_image_pixels += 1;
                        }
                    }
                }
            }
        }

        // 3. Remove overlapping annotations and scrub purged strings from metadata & bookmarks
        let before_annots = doc.annotations.len();
        doc.annotations.retain(|annot| {
            !proposals
                .iter()
                .any(|p| p.page_index == annot.page_index && p.rect.intersects(&annot.bbox))
        });
        report.removed_annotations = before_annots - doc.annotations.len();

        for value in doc.metadata.values_mut() {
            for secret in &purged_strings {
                if !secret.is_empty() && value.contains(secret) {
                    *value = value.replace(secret, "[REDACTED]");
                    report.scrubbed_metadata_entries += 1;
                }
            }
        }

        for (title, _, _) in &mut doc.bookmarks {
            for secret in &purged_strings {
                if !secret.is_empty() && title.contains(secret) {
                    *title = title.replace(secret, "[REDACTED]");
                }
            }
        }

        report
    }

    /// Re-serializes a sanitized `PageScene` into clean ISO 32000 content-stream operators.
    pub fn serialize_page_content_stream(scene: &PageScene) -> Vec<u8> {
        let mut out = String::new();
        for rect in &scene.filled_rects {
            let r = (rect.fill_color.r as f32) / 255.0;
            let g = (rect.fill_color.g as f32) / 255.0;
            let b = (rect.fill_color.b as f32) / 255.0;
            out.push_str(&format!(
                "q {:.3} {:.3} {:.3} rg {:.2} {:.2} {:.2} {:.2} re f Q\n",
                r,
                g,
                b,
                rect.rect.x0,
                rect.rect.y0,
                rect.rect.width(),
                rect.rect.height()
            ));
        }

        for run in &scene.glyph_runs {
            let escaped = run
                .text
                .replace('\\', "\\\\")
                .replace('(', "\\(")
                .replace(')', "\\)");
            out.push_str(&format!(
                "BT /{} {:.2} Tf 1 0 0 1 {:.2} {:.2} Tm ({}) Tj ET\n",
                run.font_name, run.font_size_pt, run.bbox.x0, run.baseline_y, escaped
            ));
        }

        out.into_bytes()
    }

    fn find_pattern_char_ranges(text: &str, pattern: &RedactionPattern) -> Vec<(usize, usize)> {
        let chars: Vec<char> = text.chars().collect();
        let mut ranges = Vec::new();

        match pattern {
            RedactionPattern::CustomSubstring(needle) => {
                if needle.is_empty() {
                    return ranges;
                }
                let needle_chars: Vec<char> = needle.chars().collect();
                let n_len = needle_chars.len();
                let mut i = 0;
                while i + n_len <= chars.len() {
                    if chars[i..i + n_len] == needle_chars[..] {
                        ranges.push((i, i + n_len));
                        i += n_len;
                    } else {
                        i += 1;
                    }
                }
            }
            RedactionPattern::UsSsn => {
                // Matches `DDD-DD-DDDD` (11 chars)
                if chars.len() >= 11 {
                    let mut i = 0;
                    while i + 11 <= chars.len() {
                        let s = &chars[i..i + 11];
                        let is_ssn = s[0].is_ascii_digit()
                            && s[1].is_ascii_digit()
                            && s[2].is_ascii_digit()
                            && s[3] == '-'
                            && s[4].is_ascii_digit()
                            && s[5].is_ascii_digit()
                            && s[6] == '-'
                            && s[7].is_ascii_digit()
                            && s[8].is_ascii_digit()
                            && s[9].is_ascii_digit()
                            && s[10].is_ascii_digit();
                        if is_ssn {
                            ranges.push((i, i + 11));
                            i += 11;
                        } else {
                            i += 1;
                        }
                    }
                }
            }
            RedactionPattern::EmailAddress => {
                // Token scan for `user@domain.tld`
                let mut i = 0;
                while i < chars.len() {
                    while i < chars.len() && chars[i].is_whitespace() {
                        i += 1;
                    }
                    let start = i;
                    while i < chars.len() && !chars[i].is_whitespace() {
                        i += 1;
                    }
                    let end = i;
                    if end > start + 4 {
                        let slice: String = chars[start..end].iter().collect();
                        let clean = slice.trim_matches(|c: char| ",;:()<>\"'".contains(c));
                        if let Some(at_pos) = clean.find('@') {
                            if at_pos >= 1 && clean[at_pos + 1..].contains('.') {
                                ranges.push((start, end));
                            }
                        }
                    }
                }
            }
            RedactionPattern::CreditCard => {
                // Matches 16-digit card `DDDD-DDDD-DDDD-DDDD` or `DDDD DDDD DDDD DDDD` (19 chars)
                if chars.len() >= 19 {
                    let mut i = 0;
                    while i + 19 <= chars.len() {
                        let s = &chars[i..i + 19];
                        let sep = s[4];
                        if (sep == '-' || sep == ' ')
                            && s[9] == sep
                            && s[14] == sep
                            && s[0..4].iter().all(|c| c.is_ascii_digit())
                            && s[5..9].iter().all(|c| c.is_ascii_digit())
                            && s[10..14].iter().all(|c| c.is_ascii_digit())
                            && s[15..19].iter().all(|c| c.is_ascii_digit())
                        {
                            ranges.push((i, i + 19));
                            i += 19;
                        } else {
                            i += 1;
                        }
                    }
                }
            }
            RedactionPattern::PhoneNumber => {
                // Matches `DDD-DDD-DDDD` (12 chars)
                if chars.len() >= 12 {
                    let mut i = 0;
                    while i + 12 <= chars.len() {
                        let s = &chars[i..i + 12];
                        if s[0..3].iter().all(|c| c.is_ascii_digit())
                            && s[3] == '-'
                            && s[4..7].iter().all(|c| c.is_ascii_digit())
                            && s[7] == '-'
                            && s[8..12].iter().all(|c| c.is_ascii_digit())
                        {
                            ranges.push((i, i + 12));
                            i += 12;
                        } else {
                            i += 1;
                        }
                    }
                }
            }
        }

        ranges
    }
}

// ============================================================================
// 3. INTERACTIVE ACROFORMS & DIGITAL SIGNATURES (`/AcroForm` & `/Sig`)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AcroFieldType {
    Text {
        value: String,
        multiline: bool,
        max_len: Option<usize>,
    },
    CheckBox {
        checked: bool,
    },
    RadioGroup {
        options: Vec<(String, PdfRect)>,
        selected_option: Option<String>,
    },
    ComboBox {
        options: Vec<String>,
        selected: Option<String>,
        editable: bool,
    },
    SignatureBox {
        signed_by: Option<String>,
        signature_id: Option<u64>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcroFormField {
    pub id: u64,
    pub name: String,
    pub page_index: usize,
    pub rect: PdfRect,
    pub required: bool,
    pub read_only: bool,
    pub field_type: AcroFieldType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SignatureVisualAppearance {
    DrawnVectorStrokes(Vec<Vec<(f32, f32)>>),
    TypedCalligraphic { text: String, font_name: String },
    ImageStamp { width: u32, height: u32, rgba: Vec<u8> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DigitalSignatureRecord {
    pub id: u64,
    pub field_name: String,
    pub signer_common_name: String,
    pub signer_email: String,
    pub reason: String,
    pub location: String,
    pub timestamp_iso8601: String,
    pub page_index: usize,
    pub rect: PdfRect,
    pub appearance: SignatureVisualAppearance,
    /// SHA-256 digest of the canonical document content at signing time (`/ByteRange`).
    pub content_sha256_digest: [u8; 32],
    /// Keyed HMAC/CMS signature Tag over `content_sha256_digest` + signer attributes.
    pub pkcs7_signature_tag: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureVerificationStatus {
    pub signature_id: u64,
    pub signer_common_name: String,
    pub is_valid: bool,
    pub document_modified_since_signing: bool,
}

pub struct AcroFormAndSignatureManager;

impl AcroFormAndSignatureManager {
    /// Sets the value of an AcroForm field by name.
    pub fn set_field_value(
        doc: &mut PdfSecurityDocument,
        field_name: &str,
        new_value: &str,
    ) -> Result<(), String> {
        let field = doc
            .form_fields
            .iter_mut()
            .find(|f| f.name == field_name)
            .ok_or_else(|| format!("Form field '{}' not found", field_name))?;

        if field.read_only {
            return Err(format!("Form field '{}' is read-only", field_name));
        }

        match &mut field.field_type {
            AcroFieldType::Text { value, max_len, .. } => {
                if let Some(limit) = *max_len {
                    *value = new_value.chars().take(limit).collect();
                } else {
                    *value = new_value.to_string();
                }
            }
            AcroFieldType::CheckBox { checked } => {
                *checked = matches!(
                    new_value.to_ascii_lowercase().as_str(),
                    "true" | "1" | "yes" | "on"
                );
            }
            AcroFieldType::RadioGroup {
                options,
                selected_option,
            } => {
                if options.iter().any(|(opt, _)| opt == new_value) {
                    *selected_option = Some(new_value.to_string());
                } else {
                    return Err(format!("Invalid radio option '{}'", new_value));
                }
            }
            AcroFieldType::ComboBox {
                options,
                selected,
                editable,
            } => {
                if *editable || options.iter().any(|opt| opt == new_value) {
                    *selected = Some(new_value.to_string());
                } else {
                    return Err(format!("Option '{}' not in ComboBox list", new_value));
                }
            }
            AcroFieldType::SignatureBox { .. } => {
                return Err("Use sign_document() to sign a SignatureBox field".to_string());
            }
        }

        Ok(())
    }

    /// Exports all AcroForm field values as a flat key-value map.
    pub fn export_form_data(doc: &PdfSecurityDocument) -> BTreeMap<String, String> {
        let mut map = BTreeMap::new();
        for f in &doc.form_fields {
            let val = match &f.field_type {
                AcroFieldType::Text { value, .. } => value.clone(),
                AcroFieldType::CheckBox { checked } => checked.to_string(),
                AcroFieldType::RadioGroup {
                    selected_option, ..
                } => selected_option.clone().unwrap_or_default(),
                AcroFieldType::ComboBox { selected, .. } => selected.clone().unwrap_or_default(),
                AcroFieldType::SignatureBox { signed_by, .. } => {
                    signed_by.clone().unwrap_or_default()
                }
            };
            map.insert(f.name.clone(), val);
        }
        map
    }

    /// Flattens all interactive AcroForm fields and annotations permanently into static
    /// `PositionedGlyphRun`s on their respective pages and clears `form_fields`.
    pub fn flatten_forms_and_annotations(doc: &mut PdfSecurityDocument) {
        let mut next_id = doc
            .pages
            .iter()
            .flat_map(|p| p.glyph_runs.iter().map(|r| r.id))
            .max()
            .unwrap_or(5000)
            + 1;

        let fields = std::mem::take(&mut doc.form_fields);
        for field in fields {
            if field.page_index >= doc.pages.len() {
                continue;
            }
            let display_text = match field.field_type {
                AcroFieldType::Text { value, .. } => value,
                AcroFieldType::CheckBox { checked } => {
                    if checked {
                        "☑".to_string()
                    } else {
                        "☐".to_string()
                    }
                }
                AcroFieldType::RadioGroup {
                    selected_option, ..
                } => selected_option.unwrap_or_default(),
                AcroFieldType::ComboBox { selected, .. } => selected.unwrap_or_default(),
                AcroFieldType::SignatureBox { signed_by, .. } => {
                    signed_by.map(|s| format!("Signed by: {}", s)).unwrap_or_default()
                }
            };

            if !display_text.is_empty() {
                doc.pages[field.page_index].glyph_runs.push(PositionedGlyphRun {
                    id: {
                        let id = next_id;
                        next_id += 1;
                        id
                    },
                    text: display_text,
                    bbox: field.rect,
                    baseline_y: field.rect.y1 - 3.0,
                    font_name: "Helvetica".to_string(),
                    font_size_pt: (field.rect.height() * 0.70).clamp(8.0, 14.0),
                    is_bold: false,
                    is_italic: false,
                    color: RgbaColor::default(),
                    stream_byte_range: None,
                });
            }
        }

        for page_idx in 0..doc.pages.len() {
            let stream = RedactionEngine::serialize_page_content_stream(&doc.pages[page_idx]);
            if page_idx < doc.raw_content_streams.len() {
                doc.raw_content_streams[page_idx] = stream;
            } else {
                doc.raw_content_streams.push(stream);
            }
        }
    }

    /// Computes a canonical SHA-256 `/ByteRange` digest over the document's pages, forms, and images,
    /// binds a visual + cryptographic signature record, and locks the target `SignatureBox`.
    pub fn sign_document(
        doc: &mut PdfSecurityDocument,
        field_name: &str,
        signer_common_name: &str,
        signer_email: &str,
        reason: &str,
        location: &str,
        timestamp_iso8601: &str,
        appearance: SignatureVisualAppearance,
        signing_private_key: &[u8],
    ) -> Result<DigitalSignatureRecord, String> {
        let (page_index, rect) = {
            let field = doc
                .form_fields
                .iter_mut()
                .find(|f| f.name == field_name)
                .ok_or_else(|| format!("Signature field '{}' not found", field_name))?;

            field.read_only = true;
            if let AcroFieldType::SignatureBox { signed_by, .. } = &mut field.field_type {
                *signed_by = Some(signer_common_name.to_string());
            }
            (field.page_index, field.rect)
        };

        let content_sha256_digest = Self::compute_document_byterange_sha256(doc);
        let pkcs7_signature_tag = Self::compute_cms_hmac_sha256(
            &content_sha256_digest,
            signer_common_name,
            signer_email,
            timestamp_iso8601,
            signing_private_key,
        );

        let sig_id = (doc.signatures.len() as u64) + 1;
        let record = DigitalSignatureRecord {
            id: sig_id,
            field_name: field_name.to_string(),
            signer_common_name: signer_common_name.to_string(),
            signer_email: signer_email.to_string(),
            reason: reason.to_string(),
            location: location.to_string(),
            timestamp_iso8601: timestamp_iso8601.to_string(),
            page_index,
            rect,
            appearance,
            content_sha256_digest,
            pkcs7_signature_tag,
        };

        doc.signatures.push(record.clone());
        Ok(record)
    }

    /// Verifies all digital signatures on the document against current content streams and forms.
    pub fn verify_signatures(
        doc: &PdfSecurityDocument,
        verification_key: &[u8],
    ) -> Vec<SignatureVerificationStatus> {
        let current_digest = Self::compute_document_byterange_sha256(doc);

        doc.signatures
            .iter()
            .map(|sig| {
                let expected_tag = Self::compute_cms_hmac_sha256(
                    &sig.content_sha256_digest,
                    &sig.signer_common_name,
                    &sig.signer_email,
                    &sig.timestamp_iso8601,
                    verification_key,
                );
                let digest_matches = sig.content_sha256_digest == current_digest;
                let tag_matches = sig.pkcs7_signature_tag == expected_tag;
                SignatureVerificationStatus {
                    signature_id: sig.id,
                    signer_common_name: sig.signer_common_name.clone(),
                    is_valid: digest_matches && tag_matches,
                    document_modified_since_signing: !digest_matches,
                }
            })
            .collect()
    }

    fn compute_document_byterange_sha256(doc: &PdfSecurityDocument) -> [u8; 32] {
        let mut payload = Vec::new();
        for page in &doc.pages {
            payload.extend_from_slice(&RedactionEngine::serialize_page_content_stream(page));
        }
        for img in &doc.images {
            payload.extend_from_slice(&img.rgba_pixels);
        }
        for (k, v) in Self::export_form_data(doc) {
            payload.extend_from_slice(k.as_bytes());
            payload.push(b'=');
            payload.extend_from_slice(v.as_bytes());
            payload.push(b'\n');
        }
        Sha256::digest(&payload)
    }

    fn compute_cms_hmac_sha256(
        content_digest: &[u8; 32],
        cn: &str,
        email: &str,
        timestamp: &str,
        key: &[u8],
    ) -> [u8; 32] {
        let mut msg = Vec::with_capacity(128);
        msg.extend_from_slice(content_digest);
        msg.extend_from_slice(cn.as_bytes());
        msg.extend_from_slice(email.as_bytes());
        msg.extend_from_slice(timestamp.as_bytes());
        Sha256::hmac(key, &msg)
    }
}

// ============================================================================
// 4. ISO 32000-2 AES-256 ENCRYPTION & PERMISSIONS ENGINE
// ============================================================================

/// ISO 32000-2 Table 22 User Access Permission Flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PdfPermissions {
    pub allow_print: bool,
    pub allow_modify_contents: bool,
    pub allow_copy_and_extract: bool,
    pub allow_annotations_and_forms: bool,
    pub allow_assemble_pages: bool,
    pub allow_high_quality_print: bool,
}

impl Default for PdfPermissions {
    fn default() -> Self {
        Self {
            allow_print: true,
            allow_modify_contents: true,
            allow_copy_and_extract: true,
            allow_annotations_and_forms: true,
            allow_assemble_pages: true,
            allow_high_quality_print: true,
        }
    }
}

impl PdfPermissions {
    /// Encodes permissions into the standard 32-bit `/P` integer bitmask.
    pub fn to_bitmask(self) -> u32 {
        let mut mask: u32 = 0xFFFF_F0C0; // Reserved upper bits set per ISO 32000
        if self.allow_print {
            mask |= 1 << 2;
        }
        if self.allow_modify_contents {
            mask |= 1 << 3;
        }
        if self.allow_copy_and_extract {
            mask |= 1 << 4;
            mask |= 1 << 9;
        }
        if self.allow_annotations_and_forms {
            mask |= 1 << 5;
            mask |= 1 << 8;
        }
        if self.allow_assemble_pages {
            mask |= 1 << 10;
        }
        if self.allow_high_quality_print {
            mask |= 1 << 11;
        }
        mask
    }

    pub fn from_bitmask(mask: u32) -> Self {
        Self {
            allow_print: (mask & (1 << 2)) != 0,
            allow_modify_contents: (mask & (1 << 3)) != 0,
            allow_copy_and_extract: (mask & (1 << 4)) != 0,
            allow_annotations_and_forms: (mask & (1 << 5)) != 0,
            allow_assemble_pages: (mask & (1 << 10)) != 0,
            allow_high_quality_print: (mask & (1 << 11)) != 0,
        }
    }
}

/// ISO 32000-2 `/Encrypt` dictionary header (Revision 5/6, AES-256, `V=5`, `R=6`, `Length=256`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptionHeader {
    pub version: u8,
    pub revision: u8,
    pub key_length_bits: u16,
    pub permissions_mask: u32,
    pub user_validation_salt: [u8; 8],
    pub user_key_salt: [u8; 8],
    pub owner_validation_salt: [u8; 8],
    pub owner_key_salt: [u8; 8],
    /// `/U` (48 bytes: 32-byte hash + 8-byte validation salt + 8-byte key salt)
    pub u_entry: Vec<u8>,
    /// `/UE` (32 bytes: file encryption key encrypted with User key via AES-256-CBC)
    pub ue_entry: [u8; 32],
    /// `/O` (48 bytes: 32-byte hash + 8-byte validation salt + 8-byte key salt)
    pub o_entry: Vec<u8>,
    /// `/OE` (32 bytes: file encryption key encrypted with Owner key via AES-256-CBC)
    pub oe_entry: [u8; 32],
    /// `/Perms` (16 bytes: AES-256-ECB encrypted permission block with `"adb"` marker)
    pub perms_entry: [u8; 16],
}

/// Encrypted binary envelope storing AES-256-CBC encrypted PDF streams and objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPdfPackage {
    pub header: EncryptionHeader,
    pub iv: [u8; 16],
    pub ciphertext: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnlockedRole {
    User,
    Owner,
}

pub struct PdfAes256Security;

impl PdfAes256Security {
    /// Encrypts a `PdfSecurityDocument` using ISO 32000-2 AES-256 (`V=5, R=6`) with distinct
    /// User Password (required to open) and Owner Password (required to override permissions).
    pub fn encrypt_document(
        doc: &PdfSecurityDocument,
        user_password: &str,
        owner_password: &str,
        permissions: PdfPermissions,
        deterministic_seed: &[u8; 32],
    ) -> Result<EncryptedPdfPackage, String> {
        // Derive deterministic cryptographic salts and 256-bit File Encryption Key (FEK) from seed
        let fek = Sha256::hmac(deterministic_seed, b"OPENWPS-AES256-FILE-ENCRYPTION-KEY");
        let salt_material = Sha256::hmac(deterministic_seed, b"OPENWPS-AES256-SALTS");
        let iv_material = Sha256::hmac(deterministic_seed, b"OPENWPS-AES256-CBC-IV");

        let mut u_val_salt = [0u8; 8];
        let mut u_key_salt = [0u8; 8];
        let mut o_val_salt = [0u8; 8];
        let mut o_key_salt = [0u8; 8];
        u_val_salt.copy_from_slice(&salt_material[0..8]);
        u_key_salt.copy_from_slice(&salt_material[8..16]);
        o_val_salt.copy_from_slice(&salt_material[16..24]);
        o_key_salt.copy_from_slice(&salt_material[24..32]);

        let mut iv = [0u8; 16];
        iv.copy_from_slice(&iv_material[0..16]);

        // Compute `/U` and `/UE`
        let user_val_hash = Self::kdf_sha256(user_password.as_bytes(), &u_val_salt, &[]);
        let mut u_entry = Vec::with_capacity(48);
        u_entry.extend_from_slice(&user_val_hash);
        u_entry.extend_from_slice(&u_val_salt);
        u_entry.extend_from_slice(&u_key_salt);

        let user_fek_key = Self::kdf_sha256(user_password.as_bytes(), &u_key_salt, &[]);
        let ue_entry = Aes256Cipher::encrypt_32_zero_iv(&user_fek_key, &fek);

        // Compute `/O` and `/OE`
        let eff_owner_pw = if owner_password.is_empty() {
            user_password
        } else {
            owner_password
        };
        let owner_val_hash = Self::kdf_sha256(eff_owner_pw.as_bytes(), &o_val_salt, &u_entry);
        let mut o_entry = Vec::with_capacity(48);
        o_entry.extend_from_slice(&owner_val_hash);
        o_entry.extend_from_slice(&o_val_salt);
        o_entry.extend_from_slice(&o_key_salt);

        let owner_fek_key = Self::kdf_sha256(eff_owner_pw.as_bytes(), &o_key_salt, &u_entry);
        let oe_entry = Aes256Cipher::encrypt_32_zero_iv(&owner_fek_key, &fek);

        // Compute `/Perms` (16-byte block: 4-byte permissions + 4-byte 0xFFFFFFFF + 'T' + "adb" + "open")
        let p_mask = permissions.to_bitmask();
        let mut perms_plain = [0u8; 16];
        perms_plain[0..4].copy_from_slice(&p_mask.to_le_bytes());
        perms_plain[4..8].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
        perms_plain[8] = b'T'; // EncryptMetadata = true
        perms_plain[9..12].copy_from_slice(b"adb");
        perms_plain[12..16].copy_from_slice(b"wps!");
        let perms_entry = Aes256Cipher::encrypt_block(&fek, &perms_plain);

        let header = EncryptionHeader {
            version: 5,
            revision: 6,
            key_length_bits: 256,
            permissions_mask: p_mask,
            user_validation_salt: u_val_salt,
            user_key_salt: u_key_salt,
            owner_validation_salt: o_val_salt,
            owner_key_salt: o_key_salt,
            u_entry,
            ue_entry,
            o_entry,
            oe_entry,
            perms_entry,
        };

        let mut doc_with_header = doc.clone();
        doc_with_header.encryption = Some(header.clone());

        let plaintext = serde_json::to_vec(&doc_with_header)
            .map_err(|e| format!("Failed to serialize PDF security state: {}", e))?;
        let ciphertext = Aes256Cipher::encrypt_cbc_pkcs7(&fek, &iv, &plaintext);

        Ok(EncryptedPdfPackage {
            header,
            iv,
            ciphertext,
        })
    }

    /// Attempts to unlock and decrypt an `EncryptedPdfPackage` with `password`.
    /// Returns the decrypted `PdfSecurityDocument`, the authenticated `UnlockedRole` (`User` or `Owner`),
    /// and the verified `PdfPermissions`.
    pub fn decrypt_document(
        package: &EncryptedPdfPackage,
        password: &str,
    ) -> Result<(PdfSecurityDocument, UnlockedRole, PdfPermissions), String> {
        let h = &package.header;

        // 1. Check Owner Password first (grants full Owner role)
        let candidate_o_hash = Self::kdf_sha256(
            password.as_bytes(),
            &h.owner_validation_salt,
            &h.u_entry,
        );

        let (fek, role) = if h.o_entry.len() >= 32 && candidate_o_hash[..] == h.o_entry[0..32] {
            let owner_fek_key =
                Self::kdf_sha256(password.as_bytes(), &h.owner_key_salt, &h.u_entry);
            let fek = Aes256Cipher::decrypt_32_zero_iv(&owner_fek_key, &h.oe_entry);
            (fek, UnlockedRole::Owner)
        } else {
            // 2. Check User Password
            let candidate_u_hash =
                Self::kdf_sha256(password.as_bytes(), &h.user_validation_salt, &[]);
            if h.u_entry.len() >= 32 && candidate_u_hash[..] == h.u_entry[0..32] {
                let user_fek_key = Self::kdf_sha256(password.as_bytes(), &h.user_key_salt, &[]);
                let fek = Aes256Cipher::decrypt_32_zero_iv(&user_fek_key, &h.ue_entry);
                (fek, UnlockedRole::User)
            } else {
                return Err("Invalid User or Owner password for AES-256 encrypted PDF".to_string());
            }
        };

        // 3. Verify `/Perms` integrity marker (`"adb"`) using recovered FEK
        let perms_plain = Aes256Cipher::decrypt_block(&fek, &h.perms_entry);
        if &perms_plain[9..12] != b"adb" {
            return Err("Corrupted /Perms block or invalid AES-256 File Encryption Key".to_string());
        }
        let verified_p_mask = u32::from_le_bytes([
            perms_plain[0],
            perms_plain[1],
            perms_plain[2],
            perms_plain[3],
        ]);
        let permissions = PdfPermissions::from_bitmask(verified_p_mask);

        // 4. Decrypt payload via AES-256-CBC
        let plaintext = Aes256Cipher::decrypt_cbc_pkcs7(&fek, &package.iv, &package.ciphertext)
            .ok_or_else(|| "AES-256-CBC PKCS#7 padding verification failed".to_string())?;

        let doc: PdfSecurityDocument = serde_json::from_slice(&plaintext)
            .map_err(|e| format!("Failed to deserialize decrypted PDF: {}", e))?;

        Ok((doc, role, permissions))
    }

    /// Multi-round hardened SHA-256 key derivation matching ISO 32000-2 Revision 5/6 structure.
    fn kdf_sha256(password: &[u8], salt: &[u8; 8], u_vector: &[u8]) -> [u8; 32] {
        let mut input = Vec::with_capacity(password.len() + 8 + u_vector.len());
        let truncated_pw = if password.len() > 127 {
            &password[..127]
        } else {
            password
        };
        input.extend_from_slice(truncated_pw);
        input.extend_from_slice(salt);
        input.extend_from_slice(u_vector);

        let mut digest = Sha256::digest(&input);
        // Perform 64 rounds of iterative key stretching
        for round in 0u8..64 {
            let mut round_buf = Vec::with_capacity(digest.len() + truncated_pw.len() + 1);
            round_buf.extend_from_slice(&digest);
            round_buf.extend_from_slice(truncated_pw);
            round_buf.push(round);
            digest = Sha256::digest(&round_buf);
        }
        digest
    }
}

// ============================================================================
// 5. PURE-RUST FIPS-180-4 SHA-256 & FIPS-197 AES-256-CBC PRIMITIVES
//    (100% Self-Contained for Native & wasm32-unknown-unknown)
// ============================================================================

pub struct Sha256;

impl Sha256 {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    pub fn digest(data: &[u8]) -> [u8; 32] {
        let mut h: [u32; 8] = [
            0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
            0x5be0cd19,
        ];

        let bit_len = (data.len() as u64) * 8;
        let mut padded = data.to_vec();
        padded.push(0x80);
        while (padded.len() % 64) != 56 {
            padded.push(0x00);
        }
        padded.extend_from_slice(&bit_len.to_be_bytes());

        for chunk in padded.chunks_exact(64) {
            let mut w = [0u32; 64];
            for i in 0..16 {
                w[i] = u32::from_be_bytes([
                    chunk[i * 4],
                    chunk[i * 4 + 1],
                    chunk[i * 4 + 2],
                    chunk[i * 4 + 3],
                ]);
            }
            for i in 16..64 {
                let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
                let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
                w[i] = w[i - 16]
                    .wrapping_add(s0)
                    .wrapping_add(w[i - 7])
                    .wrapping_add(s1);
            }

            let mut a = h[0];
            let mut b = h[1];
            let mut c = h[2];
            let mut d = h[3];
            let mut e = h[4];
            let mut f = h[5];
            let mut g = h[6];
            let mut hh = h[7];

            for i in 0..64 {
                let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                let ch = (e & f) ^ ((!e) & g);
                let temp1 = hh
                    .wrapping_add(s1)
                    .wrapping_add(ch)
                    .wrapping_add(Self::K[i])
                    .wrapping_add(w[i]);
                let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                let maj = (a & b) ^ (a & c) ^ (b & c);
                let temp2 = s0.wrapping_add(maj);

                hh = g;
                g = f;
                f = e;
                e = d.wrapping_add(temp1);
                d = c;
                c = b;
                b = a;
                a = temp1.wrapping_add(temp2);
            }

            h[0] = h[0].wrapping_add(a);
            h[1] = h[1].wrapping_add(b);
            h[2] = h[2].wrapping_add(c);
            h[3] = h[3].wrapping_add(d);
            h[4] = h[4].wrapping_add(e);
            h[5] = h[5].wrapping_add(f);
            h[6] = h[6].wrapping_add(g);
            h[7] = h[7].wrapping_add(hh);
        }

        let mut out = [0u8; 32];
        for (i, word) in h.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
        }
        out
    }

    pub fn hmac(key: &[u8], message: &[u8]) -> [u8; 32] {
        let mut k_block = [0u8; 64];
        if key.len() > 64 {
            let hashed = Self::digest(key);
            k_block[..32].copy_from_slice(&hashed);
        } else {
            k_block[..key.len()].copy_from_slice(key);
        }

        let mut o_key_pad = [0x5cu8; 64];
        let mut i_key_pad = [0x36u8; 64];
        for i in 0..64 {
            o_key_pad[i] ^= k_block[i];
            i_key_pad[i] ^= k_block[i];
        }

        let mut inner = Vec::with_capacity(64 + message.len());
        inner.extend_from_slice(&i_key_pad);
        inner.extend_from_slice(message);
        let inner_hash = Self::digest(&inner);

        let mut outer = Vec::with_capacity(96);
        outer.extend_from_slice(&o_key_pad);
        outer.extend_from_slice(&inner_hash);
        Self::digest(&outer)
    }
}

pub struct Aes256Cipher;

impl Aes256Cipher {
    const SBOX: [u8; 256] = [
        0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab,
        0x76, 0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4,
        0x72, 0xc0, 0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71,
        0xd8, 0x31, 0x15, 0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2,
        0xeb, 0x27, 0xb2, 0x75, 0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6,
        0xb3, 0x29, 0xe3, 0x2f, 0x84, 0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb,
        0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf, 0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45,
        0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8, 0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5,
        0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2, 0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44,
        0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73, 0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a,
        0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb, 0xe0, 0x32, 0x3a, 0x0a, 0x49,
        0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79, 0xe7, 0xc8, 0x37, 0x6d,
        0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08, 0xba, 0x78, 0x25,
        0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a, 0x70, 0x3e,
        0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e, 0xe1,
        0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
        0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb,
        0x16,
    ];

    const INV_SBOX: [u8; 256] = [
        0x52, 0x09, 0x6a, 0xd5, 0x30, 0x36, 0xa5, 0x38, 0xbf, 0x40, 0xa3, 0x9e, 0x81, 0xf3, 0xd7,
        0xfb, 0x7c, 0xe3, 0x39, 0x82, 0x9b, 0x2f, 0xff, 0x87, 0x34, 0x8e, 0x43, 0x44, 0xc4, 0xde,
        0xe9, 0xcb, 0x54, 0x7b, 0x94, 0x32, 0xa6, 0xc2, 0x23, 0x3d, 0xee, 0x4c, 0x95, 0x0b, 0x42,
        0xfa, 0xc3, 0x4e, 0x08, 0x2e, 0xa1, 0x66, 0x28, 0xd9, 0x24, 0xb2, 0x76, 0x5b, 0xa2, 0x49,
        0x6d, 0x8b, 0xd1, 0x25, 0x72, 0xf8, 0xf6, 0x64, 0x86, 0x68, 0x98, 0x16, 0xd4, 0xa4, 0x5c,
        0xcc, 0x5d, 0x65, 0xb6, 0x92, 0x6c, 0x70, 0x48, 0x50, 0xfd, 0xed, 0xb9, 0xda, 0x5e, 0x15,
        0x46, 0x57, 0xa7, 0x8d, 0x9d, 0x84, 0x90, 0xd8, 0xab, 0x00, 0x8c, 0xbc, 0xd3, 0x0a, 0xf7,
        0xe4, 0x58, 0x05, 0xb8, 0xb3, 0x45, 0x06, 0xd0, 0x2c, 0x1e, 0x8f, 0xca, 0x3f, 0x0f, 0x02,
        0xc1, 0xaf, 0xbd, 0x03, 0x01, 0x13, 0x8a, 0x6b, 0x3a, 0x91, 0x11, 0x41, 0x4f, 0x67, 0xdc,
        0xea, 0x97, 0xf2, 0xcf, 0xce, 0xf0, 0xb4, 0xe6, 0x73, 0x96, 0xac, 0x74, 0x22, 0xe7, 0xad,
        0x35, 0x85, 0xe2, 0xf9, 0x37, 0xe8, 0x1c, 0x75, 0xdf, 0x6e, 0x47, 0xf1, 0x1a, 0x71, 0x1d,
        0x29, 0xc5, 0x89, 0x6f, 0xb7, 0x62, 0x0e, 0xaa, 0x18, 0xbe, 0x1b, 0xfc, 0x56, 0x3e, 0x4b,
        0xc6, 0xd2, 0x79, 0x20, 0x9a, 0xdb, 0xc0, 0xfe, 0x78, 0xcd, 0x5a, 0xf4, 0x1f, 0xdd, 0xa8,
        0x33, 0x88, 0x07, 0xc7, 0x31, 0xb1, 0x12, 0x10, 0x59, 0x27, 0x80, 0xec, 0x5f, 0x60, 0x51,
        0x7f, 0xa9, 0x19, 0xb5, 0x4a, 0x0d, 0x2d, 0xe5, 0x7a, 0x9f, 0x93, 0xc9, 0x9c, 0xef, 0xa0,
        0xe0, 0x3b, 0x4d, 0xae, 0x2a, 0xf5, 0xb0, 0xc8, 0xeb, 0xbb, 0x3c, 0x83, 0x53, 0x99, 0x61,
        0x17, 0x2b, 0x04, 0x7e, 0xba, 0x77, 0xd6, 0x26, 0xe1, 0x69, 0x14, 0x63, 0x55, 0x21, 0x0c,
        0x7d,
    ];

    const RCON: [u8; 15] = [
        0x00, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36, 0x6c, 0xd8, 0xab, 0x4d,
    ];

    fn expand_key(key: &[u8; 32]) -> [[u8; 16]; 15] {
        let mut w = [[0u8; 4]; 60];
        for i in 0..8 {
            w[i] = [key[4 * i], key[4 * i + 1], key[4 * i + 2], key[4 * i + 3]];
        }
        for i in 8..60 {
            let mut temp = w[i - 1];
            if i % 8 == 0 {
                temp = [
                    Self::SBOX[temp[1] as usize] ^ Self::RCON[i / 8],
                    Self::SBOX[temp[2] as usize],
                    Self::SBOX[temp[3] as usize],
                    Self::SBOX[temp[0] as usize],
                ];
            } else if i % 8 == 4 {
                temp = [
                    Self::SBOX[temp[0] as usize],
                    Self::SBOX[temp[1] as usize],
                    Self::SBOX[temp[2] as usize],
                    Self::SBOX[temp[3] as usize],
                ];
            }
            let prev = w[i - 8];
            w[i] = [
                prev[0] ^ temp[0],
                prev[1] ^ temp[1],
                prev[2] ^ temp[2],
                prev[3] ^ temp[3],
            ];
        }

        let mut round_keys = [[0u8; 16]; 15];
        for r in 0..15 {
            for c in 0..4 {
                round_keys[r][c * 4..c * 4 + 4].copy_from_slice(&w[r * 4 + c]);
            }
        }
        round_keys
    }

    #[inline]
    fn gmul(mut a: u8, mut b: u8) -> u8 {
        let mut p = 0u8;
        for _ in 0..8 {
            if (b & 1) != 0 {
                p ^= a;
            }
            let hi = (a & 0x80) != 0;
            a <<= 1;
            if hi {
                a ^= 0x1b;
            }
            b >>= 1;
        }
        p
    }

    pub fn encrypt_block(key: &[u8; 32], input: &[u8; 16]) -> [u8; 16] {
        let rk = Self::expand_key(key);
        let mut s = *input;
        for i in 0..16 {
            s[i] ^= rk[0][i];
        }

        for r in 1..14 {
            for i in 0..16 {
                s[i] = Self::SBOX[s[i] as usize];
            }
            // ShiftRows
            s = [
                s[0], s[5], s[10], s[15], s[4], s[9], s[14], s[3], s[8], s[13], s[2], s[7], s[12],
                s[1], s[6], s[11],
            ];
            // MixColumns
            for c in 0..4 {
                let i = c * 4;
                let (s0, s1, s2, s3) = (s[i], s[i + 1], s[i + 2], s[i + 3]);
                s[i] = Self::gmul(s0, 2) ^ Self::gmul(s1, 3) ^ s2 ^ s3;
                s[i + 1] = s0 ^ Self::gmul(s1, 2) ^ Self::gmul(s2, 3) ^ s3;
                s[i + 2] = s0 ^ s1 ^ Self::gmul(s2, 2) ^ Self::gmul(s3, 3);
                s[i + 3] = Self::gmul(s0, 3) ^ s1 ^ s2 ^ Self::gmul(s3, 2);
            }
            for i in 0..16 {
                s[i] ^= rk[r][i];
            }
        }

        for i in 0..16 {
            s[i] = Self::SBOX[s[i] as usize];
        }
        s = [
            s[0], s[5], s[10], s[15], s[4], s[9], s[14], s[3], s[8], s[13], s[2], s[7], s[12],
            s[1], s[6], s[11],
        ];
        for i in 0..16 {
            s[i] ^= rk[14][i];
        }
        s
    }

    pub fn decrypt_block(key: &[u8; 32], input: &[u8; 16]) -> [u8; 16] {
        let rk = Self::expand_key(key);
        let mut s = *input;
        for i in 0..16 {
            s[i] ^= rk[14][i];
        }

        for r in (1..14).rev() {
            // InvShiftRows
            s = [
                s[0], s[13], s[10], s[7], s[4], s[1], s[14], s[11], s[8], s[5], s[2], s[15], s[12],
                s[9], s[6], s[3],
            ];
            for i in 0..16 {
                s[i] = Self::INV_SBOX[s[i] as usize];
            }
            for i in 0..16 {
                s[i] ^= rk[r][i];
            }
            // InvMixColumns
            for c in 0..4 {
                let i = c * 4;
                let (s0, s1, s2, s3) = (s[i], s[i + 1], s[i + 2], s[i + 3]);
                s[i] = Self::gmul(s0, 14)
                    ^ Self::gmul(s1, 11)
                    ^ Self::gmul(s2, 13)
                    ^ Self::gmul(s3, 9);
                s[i + 1] = Self::gmul(s0, 9)
                    ^ Self::gmul(s1, 14)
                    ^ Self::gmul(s2, 11)
                    ^ Self::gmul(s3, 13);
                s[i + 2] = Self::gmul(s0, 13)
                    ^ Self::gmul(s1, 9)
                    ^ Self::gmul(s2, 14)
                    ^ Self::gmul(s3, 11);
                s[i + 3] = Self::gmul(s0, 11)
                    ^ Self::gmul(s1, 13)
                    ^ Self::gmul(s2, 9)
                    ^ Self::gmul(s3, 14);
            }
        }

        s = [
            s[0], s[13], s[10], s[7], s[4], s[1], s[14], s[11], s[8], s[5], s[2], s[15], s[12],
            s[9], s[6], s[3],
        ];
        for i in 0..16 {
            s[i] = Self::INV_SBOX[s[i] as usize];
            s[i] ^= rk[0][i];
        }
        s
    }

    pub fn encrypt_32_zero_iv(key: &[u8; 32], data: &[u8; 32]) -> [u8; 32] {
        let mut b0 = [0u8; 16];
        b0.copy_from_slice(&data[0..16]);
        let c0 = Self::encrypt_block(key, &b0);

        let mut b1 = [0u8; 16];
        for i in 0..16 {
            b1[i] = data[16 + i] ^ c0[i];
        }
        let c1 = Self::encrypt_block(key, &b1);

        let mut out = [0u8; 32];
        out[0..16].copy_from_slice(&c0);
        out[16..32].copy_from_slice(&c1);
        out
    }

    pub fn decrypt_32_zero_iv(key: &[u8; 32], cipher: &[u8; 32]) -> [u8; 32] {
        let mut c0 = [0u8; 16];
        let mut c1 = [0u8; 16];
        c0.copy_from_slice(&cipher[0..16]);
        c1.copy_from_slice(&cipher[16..32]);

        let p0 = Self::decrypt_block(key, &c0);
        let mut p1 = Self::decrypt_block(key, &c1);
        for i in 0..16 {
            p1[i] ^= c0[i];
        }

        let mut out = [0u8; 32];
        out[0..16].copy_from_slice(&p0);
        out[16..32].copy_from_slice(&p1);
        out
    }

    pub fn encrypt_cbc_pkcs7(key: &[u8; 32], iv: &[u8; 16], plaintext: &[u8]) -> Vec<u8> {
        let pad_len = 16 - (plaintext.len() % 16);
        let mut padded = Vec::with_capacity(plaintext.len() + pad_len);
        padded.extend_from_slice(plaintext);
        padded.resize(plaintext.len() + pad_len, pad_len as u8);

        let mut out = Vec::with_capacity(padded.len());
        let mut prev = *iv;

        for chunk in padded.chunks_exact(16) {
            let mut block = [0u8; 16];
            for i in 0..16 {
                block[i] = chunk[i] ^ prev[i];
            }
            let enc = Self::encrypt_block(key, &block);
            out.extend_from_slice(&enc);
            prev = enc;
        }
        out
    }

    pub fn decrypt_cbc_pkcs7(
        key: &[u8; 32],
        iv: &[u8; 16],
        ciphertext: &[u8],
    ) -> Option<Vec<u8>> {
        if ciphertext.is_empty() || ciphertext.len() % 16 != 0 {
            return None;
        }

        let mut out = Vec::with_capacity(ciphertext.len());
        let mut prev = *iv;

        for chunk in ciphertext.chunks_exact(16) {
            let mut c_block = [0u8; 16];
            c_block.copy_from_slice(chunk);
            let mut dec = Self::decrypt_block(key, &c_block);
            for i in 0..16 {
                dec[i] ^= prev[i];
            }
            out.extend_from_slice(&dec);
            prev = c_block;
        }

        let pad_len = *out.last()? as usize;
        if pad_len == 0 || pad_len > 16 || pad_len > out.len() {
            return None;
        }
        for &b in &out[out.len() - pad_len..] {
            if (b as usize) != pad_len {
                return None;
            }
        }
        out.truncate(out.len() - pad_len);
        Some(out)
    }
}

// ============================================================================
// 6. UNIT TESTS (GATE 5 VERIFICATION SUITE)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_sample_confidential_pdf() -> PdfSecurityDocument {
        let raw_stream =
            b"BT /Helvetica 12.00 Tf 1 0 0 1 50.00 100.00 Tm (CONFIDENTIAL SSN: 999-00-1234 DO NOT SHARE) Tj ET\n"
                .to_vec();

        let run = PositionedGlyphRun {
            id: 1,
            text: "CONFIDENTIAL SSN: 999-00-1234 DO NOT SHARE".to_string(),
            bbox: PdfRect::new(50.0, 88.0, 470.0, 102.0), // 42 chars * 10pt width = 420pt
            baseline_y: 100.0,
            font_name: "Helvetica".to_string(),
            font_size_pt: 12.0,
            is_bold: false,
            is_italic: false,
            color: RgbaColor::default(),
            stream_byte_range: Some((0, raw_stream.len())),
        };

        let page = PageScene {
            page_index: 0,
            width_pt: 612.0,
            height_pt: 792.0,
            glyph_runs: vec![run],
            stroke_segments: vec![],
            filled_rects: vec![],
        };

        // 10x10 white raster image sitting behind the SSN region
        let image = PdfRasterImage {
            object_id: 10,
            page_index: 0,
            bbox: PdfRect::new(200.0, 80.0, 350.0, 110.0),
            width_px: 10,
            height_px: 10,
            rgba_pixels: vec![255u8; 10 * 10 * 4],
        };

        let mut metadata = BTreeMap::new();
        metadata.insert(
            "Keywords".to_string(),
            "Tax Record for 999-00-1234".to_string(),
        );

        PdfSecurityDocument {
            pages: vec![page],
            raw_content_streams: vec![raw_stream],
            images: vec![image],
            annotations: vec![],
            form_fields: vec![AcroFormField {
                id: 1,
                name: "ApprovalSig".to_string(),
                page_index: 0,
                rect: PdfRect::new(50.0, 500.0, 250.0, 550.0),
                required: true,
                read_only: false,
                field_type: AcroFieldType::SignatureBox {
                    signed_by: None,
                    signature_id: None,
                },
            }],
            signatures: vec![],
            metadata,
            bookmarks: vec![("SSN 999-00-1234 Filing".to_string(), 0, 100.0)],
            encryption: None,
        }
    }

    #[test]
    fn test_true_redaction_purges_ssn_from_streams_glyphs_pixels_and_metadata() {
        let mut doc = create_sample_confidential_pdf();
        let mut next_id = 1;

        // 1. Automatically find SSN pattern and propose redaction
        let proposals = RedactionEngine::search_and_propose_redactions(
            &doc,
            &RedactionPattern::UsSsn,
            Some("[REDACTED-SSN]".to_string()),
            &mut next_id,
        );
        assert_eq!(proposals.len(), 1, "Must locate exact SSN bounding box");

        // 2. Apply permanent byte-stripping redaction
        let report = RedactionEngine::apply_redactions_permanently(&mut doc, &proposals);
        assert_eq!(report.redacted_characters, 11); // "999-00-1234"
        assert!(report.scrubbed_stream_bytes > 0);
        assert!(report.scrubbed_image_pixels > 0);
        assert_eq!(report.scrubbed_metadata_entries, 1);

        // 3. Verify "999-00-1234" is completely absent from glyph runs, raw content stream bytes,
        //    metadata, and bookmarks, while surrounding non-redacted text survives!
        let combined_glyph_text: String = doc.pages[0]
            .glyph_runs
            .iter()
            .map(|r| r.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        assert!(!combined_glyph_text.contains("999-00-1234"));
        assert!(combined_glyph_text.contains("CONFIDENTIAL SSN:"));
        assert!(combined_glyph_text.contains("DO NOT SHARE"));
        assert!(combined_glyph_text.contains("[REDACTED-SSN]"));

        let raw_stream_str = String::from_utf8_lossy(&doc.raw_content_streams[0]);
        assert!(
            !raw_stream_str.contains("999-00-1234"),
            "Raw PDF stream must not contain redacted SSN bytes"
        );

        assert!(!doc.metadata["Keywords"].contains("999-00-1234"));
        assert!(!doc.bookmarks[0].0.contains("999-00-1234"));
    }

    #[test]
    fn test_aes256_encryption_user_owner_passwords_and_permissions() {
        let doc = create_sample_confidential_pdf();
        let perms = PdfPermissions {
            allow_print: true,
            allow_modify_contents: false,
            allow_copy_and_extract: false,
            allow_annotations_and_forms: true,
            allow_assemble_pages: false,
            allow_high_quality_print: true,
        };
        let seed = [0x42u8; 32];

        let pkg = PdfAes256Security::encrypt_document(
            &doc,
            "user-secret-2026",
            "owner-master-2026",
            perms,
            &seed,
        )
        .expect("AES-256 encryption should succeed");

        // Raw ciphertext must not contain plaintext strings
        let cipher_lossy = String::from_utf8_lossy(&pkg.ciphertext);
        assert!(!cipher_lossy.contains("CONFIDENTIAL SSN"));

        // Wrong password must fail
        assert!(PdfAes256Security::decrypt_document(&pkg, "wrong-password").is_err());

        // User password unlocks as `UnlockedRole::User` with exact permissions preserved
        let (unlocked_user_doc, role_u, unlocked_perms) =
            PdfAes256Security::decrypt_document(&pkg, "user-secret-2026")
                .expect("User password must unlock document");
        assert_eq!(role_u, UnlockedRole::User);
        assert_eq!(unlocked_perms, perms);
        assert_eq!(
            unlocked_user_doc.pages[0].glyph_runs[0].text,
            "CONFIDENTIAL SSN: 999-00-1234 DO NOT SHARE"
        );

        // Owner password unlocks as `UnlockedRole::Owner`
        let (_, role_o, _) = PdfAes256Security::decrypt_document(&pkg, "owner-master-2026")
            .expect("Owner password must unlock document");
        assert_eq!(role_o, UnlockedRole::Owner);
    }

    #[test]
    fn test_digital_signature_signing_and_tamper_detection() {
        let mut doc = create_sample_confidential_pdf();
        let key = b"enterprise-pkcs7-signing-key-32b";

        AcroFormAndSignatureManager::sign_document(
            &mut doc,
            "ApprovalSig",
            "Veronica Medusa",
            "veronica@openwps.dev",
            "Executive Approval",
            "Bengaluru, IN",
            "2026-10-30T18:30:00Z",
            SignatureVisualAppearance::TypedCalligraphic {
                text: "Veronica Medusa".to_string(),
                font_name: "SignPainter".to_string(),
            },
            key,
        )
        .expect("Signing should succeed");

        // Verify immediately after signing -> valid & unmodified
        let status_initial = AcroFormAndSignatureManager::verify_signatures(&doc, key);
        assert_eq!(status_initial.len(), 1);
        assert!(status_initial[0].is_valid);
        assert!(!status_initial[0].document_modified_since_signing);

        // Tamper with a glyph on page 0 -> verification must detect post-signing modification!
        doc.pages[0].glyph_runs[0].text = "TAMPERED AMOUNT: $9,000,000".to_string();
        let status_tampered = AcroFormAndSignatureManager::verify_signatures(&doc, key);
        assert!(!status_tampered[0].is_valid);
        assert!(status_tampered[0].document_modified_since_signing);
    }
}
