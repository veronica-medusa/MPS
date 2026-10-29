use wps_core::color::Color;
use wps_core::geometry::Rect;
use crate::parser::{PdfPage, PdfTextElement};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatesPosition {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SplitMode {
    PageRanges(Vec<(usize, usize)>), // 1-based start and end inclusive
    EveryNPages(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    Standard,
    High,
    Extreme,
}

pub struct PageOrganizer;

impl PageOrganizer {
    /// Reorders pages according to the provided index array
    pub fn reorder_pages(pages: &mut Vec<PdfPage>, new_order: &[usize]) {
        let mut reordered = Vec::with_capacity(new_order.len());
        for &idx in new_order {
            if idx < pages.len() {
                let mut page = pages[idx].clone();
                page.page_index = reordered.len();
                reordered.push(page);
            }
        }
        *pages = reordered;
    }

    /// Rotates a page by 90, 180, or 270 degrees clockwise
    pub fn rotate_page(page: &mut PdfPage, degrees: i32) {
        page.rotation = (page.rotation + degrees).rem_euclid(360);
        if degrees == 90 || degrees == 270 {
            std::mem::swap(&mut page.width, &mut page.height);
            let old_crop = page.crop_box;
            page.crop_box = Rect::new(old_crop.y, old_crop.x, old_crop.height, old_crop.width);
        }
    }

    /// Crops a page to the target bounding box
    pub fn crop_page(page: &mut PdfPage, new_crop_box: Rect) {
        page.crop_box = new_crop_box;
    }

    /// Deletes pages at the specified indices
    pub fn delete_pages(pages: &mut Vec<PdfPage>, indices_to_remove: &[usize]) {
        let mut sorted = indices_to_remove.to_vec();
        sorted.sort_unstable();
        sorted.dedup();

        let mut filtered = Vec::new();
        for (i, page) in pages.iter().enumerate() {
            if !sorted.contains(&i) {
                let mut p = page.clone();
                p.page_index = filtered.len();
                filtered.push(p);
            }
        }
        *pages = filtered;
    }

    /// Extracts pages into a separate document
    pub fn extract_pages(pages: &[PdfPage], indices: &[usize]) -> Vec<PdfPage> {
        let mut extracted = Vec::new();
        for (new_idx, &orig_idx) in indices.iter().enumerate() {
            if let Some(p) = pages.get(orig_idx) {
                let mut copy = p.clone();
                copy.page_index = new_idx;
                extracted.push(copy);
            }
        }
        extracted
    }

    /// Merges other pages onto the end of the current page list
    pub fn merge_pages(pages: &mut Vec<PdfPage>, other_pages: Vec<PdfPage>) {
        for mut p in other_pages {
            p.page_index = pages.len();
            pages.push(p);
        }
    }

    /// Applies a semi-transparent diagonal watermark across all pages
    pub fn apply_watermark(pages: &mut [PdfPage], text: &str, font_size: f64, opacity: f32, angle_deg: f64, color: Color) {
        let mut wm_color = color;
        wm_color.a = (opacity * 255.0) as u8;

        for page in pages.iter_mut() {
            // Calculate center of page
            let center_x = page.width * 0.5;
            let center_y = page.height * 0.5;
            let text_w = text.chars().count() as f64 * font_size * 0.55;

            let elem = PdfTextElement {
                text: text.to_string(),
                bounds: Rect::new(center_x - text_w * 0.5, center_y - font_size * 0.5, text_w, font_size),
                font_name: "Helvetica-Bold".to_string(),
                font_size,
                color: wm_color,
                baseline: center_y + font_size * 0.3,
            };

            page.text_elements.push(elem);

            // Also append watermark command to raw stream
            let r = wm_color.r as f64 / 255.0;
            let g = wm_color.g as f64 / 255.0;
            let b = wm_color.b as f64 / 255.0;
            let rad = angle_deg.to_radians();
            let cos_a = rad.cos();
            let sin_a = rad.sin();

            let safe_text = text.replace('\\', "\\\\").replace('(', "\\(").replace(')', "\\)");
            let wm_stream = format!(
                "q\n/F1 {:.2} Tf\n{:.3} {:.3} {:.3} rg\n{:.4} {:.4} {:.4} {:.4} {:.2} {:.2} cm\nBT\n({}) Tj\nET\nQ\n",
                font_size, r, g, b, cos_a, sin_a, -sin_a, cos_a, center_x - text_w * 0.5, center_y, safe_text
            );
            page.raw_stream.extend_from_slice(wm_stream.as_bytes());
        }
    }

    /// Applies sequential Bates numbering for legal and regulatory discovery
    pub fn apply_bates_numbering(
        pages: &mut [PdfPage],
        prefix: &str,
        start_number: u32,
        num_digits: usize,
        position: BatesPosition,
    ) {
        let font_size = 10.0;
        let margin = 36.0; // 0.5 inch = 36pt

        for (idx, page) in pages.iter_mut().enumerate() {
            let number = start_number + idx as u32;
            let bates_str = format!("{}{:0width$}", prefix, number, width = num_digits);
            let text_w = bates_str.chars().count() as f64 * font_size * 0.55;

            let (x, y) = match position {
                BatesPosition::TopLeft => (margin, margin),
                BatesPosition::TopCenter => ((page.width - text_w) * 0.5, margin),
                BatesPosition::TopRight => (page.width - margin - text_w, margin),
                BatesPosition::BottomLeft => (margin, page.height - margin),
                BatesPosition::BottomCenter => ((page.width - text_w) * 0.5, page.height - margin),
                BatesPosition::BottomRight => (page.width - margin - text_w, page.height - margin),
            };

            let elem = PdfTextElement {
                text: bates_str.clone(),
                bounds: Rect::new(x, y, text_w, font_size),
                font_name: "Courier-Bold".to_string(),
                font_size,
                color: Color::rgb(0, 0, 0),
                baseline: y + font_size * 0.8,
            };

            page.text_elements.push(elem);

            let pdf_y = page.height - y - font_size;
            let chunk = format!(
                "BT\n/F2 {:.2} Tf\n0 0 0 rg\n1 0 0 1 {:.2} {:.2} Tm\n({}) Tj\nET\n",
                font_size, x, pdf_y, bates_str
            );
            page.raw_stream.extend_from_slice(chunk.as_bytes());
        }
    }

    /// Splits pages into multiple document chunks
    pub fn split_pdf(pages: &[PdfPage], mode: SplitMode) -> Vec<(String, Vec<PdfPage>)> {
        let mut results = Vec::new();
        match mode {
            SplitMode::PageRanges(ranges) => {
                for (idx, (start, end)) in ranges.into_iter().enumerate() {
                    let mut chunk_pages = Vec::new();
                    let s = start.saturating_sub(1);
                    let e = end.min(pages.len());
                    for p_idx in s..e {
                        if let Some(p) = pages.get(p_idx) {
                            let mut copy = p.clone();
                            copy.page_index = chunk_pages.len();
                            chunk_pages.push(copy);
                        }
                    }
                    results.push((format!("Part_{}_pages_{}_{}.pdf", idx + 1, start, end), chunk_pages));
                }
            }
            SplitMode::EveryNPages(n) => {
                let n = n.max(1);
                for (chunk_idx, chunk) in pages.chunks(n).enumerate() {
                    let mut chunk_pages = Vec::new();
                    for p in chunk {
                        let mut copy = p.clone();
                        copy.page_index = chunk_pages.len();
                        chunk_pages.push(copy);
                    }
                    results.push((format!("Part_{}.pdf", chunk_idx + 1), chunk_pages));
                }
            }
        }
        results
    }

    /// Compresses page streams according to the target compression level
    pub fn compress_pages(pages: &mut [PdfPage], level: CompressionLevel) -> usize {
        let mut bytes_saved = 0;
        for page in pages.iter_mut() {
            let original_len = page.raw_stream.len();
            if original_len == 0 {
                continue;
            }

            let mut compacted = Vec::with_capacity(original_len);
            let mut prev_space = false;
            for &b in &page.raw_stream {
                if b == b' ' || b == b'\t' {
                    if !prev_space {
                        compacted.push(b' ');
                        prev_space = true;
                    }
                } else {
                    compacted.push(b);
                    prev_space = false;
                }
            }

            match level {
                CompressionLevel::Standard => {
                    if compacted.len() < original_len {
                        bytes_saved += original_len - compacted.len();
                        page.raw_stream = compacted;
                    }
                }
                CompressionLevel::High | CompressionLevel::Extreme => {
                    let compressed = miniz_oxide::deflate::compress_to_vec_zlib(&compacted, 6);
                    if compressed.len() < original_len {
                        bytes_saved += original_len - compressed.len();
                        page.raw_stream = compressed;
                    } else if compacted.len() < original_len {
                        bytes_saved += original_len - compacted.len();
                        page.raw_stream = compacted;
                    }
                }
            }
        }
        bytes_saved
    }
}
