use std::collections::BTreeMap;
use crate::ast::*;
use crate::layout::LaidOutPage;

pub struct DocumentAutomation;

impl DocumentAutomation {
    /// Scans paragraphs with outline levels, finds their page numbers in layout, and generates/updates the TOC block
    pub fn update_table_of_contents(doc: &mut Document, pages: &[LaidOutPage]) {
        let mut entries = Vec::new();

        // 1. Collect all headings with their target page numbers
        for sec in &doc.sections {
            for elem in &sec.elements {
                if let BlockNode::Paragraph(p) = elem {
                    let level = doc.styles.get_style(&p.style_id)
                        .and_then(|s| s.outline_level)
                        .or_else(|| {
                            if p.style_id == "Heading 1" { Some(1) }
                            else if p.style_id == "Heading 2" { Some(2) }
                            else if p.style_id == "Heading 3" { Some(3) }
                            else { None }
                        });

                    if let Some(lvl) = level {
                        let text = p.plain_text();
                        // Find which page this heading landed on
                        let mut target_page = 1;
                        for page in pages {
                            let contains = page.items.iter().any(|item| {
                                match item {
                                    crate::layout::LaidOutItem::Line(l) => {
                                        l.runs.iter().any(|r| r.text.contains(&text))
                                    }
                                    _ => false,
                                }
                            });
                            if contains {
                                target_page = page.page_number;
                                break;
                            }
                        }

                        entries.push((lvl, text, target_page));
                    }
                }
            }
        }

        // 2. Generate TOC Paragraphs with dot leaders
        let mut toc_paragraphs = Vec::new();

        let mut toc_title = ParagraphNode::new("toc-title");
        toc_title.style_id = "Heading 1".to_string();
        toc_title.add_run(TextRun::new("Table of Contents").bold().with_size(18.0));
        toc_paragraphs.push(BlockNode::Paragraph(toc_title));

        for (idx, (lvl, title, page_num)) in entries.iter().enumerate() {
            let mut toc_p = ParagraphNode::new(&format!("toc-item-{}", idx + 1));
            toc_p.style_id = format!("TOC {}", lvl);
            toc_p.indent_left_twips = wps_core::units::Twips((*lvl as i64 - 1) * 360);

            let dots_count = (60usize).saturating_sub(title.chars().count() + page_num.to_string().len());
            let dots: String = std::iter::repeat('.').take(dots_count).collect();

            let entry_line = format!("{} {} {}", title, dots, page_num);
            toc_p.add_run(TextRun::new(&entry_line).with_size(10.0));
            toc_paragraphs.push(BlockNode::Paragraph(toc_p));
        }

        // Insert at beginning of first section
        if !doc.sections.is_empty() {
            let mut new_elements = toc_paragraphs;
            new_elements.extend(doc.sections[0].elements.clone());
            doc.sections[0].elements = new_elements;
        }
    }

    /// Executes Mail Merge substituting template merge fields with data records
    pub fn execute_mail_merge(
        template_doc: &Document,
        records: &[BTreeMap<String, String>],
    ) -> Result<Vec<Document>, String> {
        let mut merged_docs = Vec::new();

        for (rec_idx, record) in records.iter().enumerate() {
            let mut doc_copy = template_doc.clone();
            doc_copy.title = format!("{} - Record {}", template_doc.title, rec_idx + 1);

            // Replace merge fields across all sections and blocks
            for sec in &mut doc_copy.sections {
                for elem in &mut sec.elements {
                    Self::replace_in_block(elem, record);
                }
                for elem in &mut sec.header_elements {
                    Self::replace_in_block(elem, record);
                }
                for elem in &mut sec.footer_elements {
                    Self::replace_in_block(elem, record);
                }
            }

            // Sync merged text into rope buffer
            let full_text = doc_copy.sections.iter()
                .flat_map(|s| &s.elements)
                .map(|e| e.plain_text())
                .collect::<Vec<_>>()
                .join("\n");

            doc_copy.buffer = RopeBuffer::from_str(&full_text);

            merged_docs.push(doc_copy);
        }

        Ok(merged_docs)
    }

    fn replace_in_block(block: &mut BlockNode, record: &BTreeMap<String, String>) {
        match block {
            BlockNode::Paragraph(p) => {
                for inline in &mut p.inlines {
                    match inline {
                        InlineElement::Run(run) => {
                            for (key, val) in record {
                                let pattern_curly = format!("{{{}}}", key);
                                let pattern_chevron = format!("«{}»", key);
                                if run.text.contains(&pattern_curly) {
                                    run.text = run.text.replace(&pattern_curly, val);
                                }
                                if run.text.contains(&pattern_chevron) {
                                    run.text = run.text.replace(&pattern_chevron, val);
                                }
                            }
                        }
                        InlineElement::Field(f) => {
                            if let DynamicField::MergeField(ref name) = f.field_type {
                                if let Some(val) = record.get(name) {
                                    f.cached_value = val.clone();
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            BlockNode::Table(table) => {
                for row in &mut table.rows {
                    for cell in &mut row.cells {
                        for sub_elem in &mut cell.elements {
                            Self::replace_in_block(sub_elem, record);
                        }
                    }
                }
            }
            BlockNode::FloatingObject(_) => {}
        }
    }
}
