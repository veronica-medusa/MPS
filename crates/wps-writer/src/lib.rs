pub mod ast;
pub mod automation;
pub mod buffer;
pub mod comments;
pub mod docx;
pub mod layout;
pub mod numbering;
pub mod revisions;
pub mod styles;

pub use ast::*;
pub use automation::DocumentAutomation;
pub use buffer::{AttributeMap, AttributeSpan, RopeBuffer, RunStyle, UnderlineStyle, VerticalAlignScript};
pub use comments::{Comment, CommentReply, CommentStore};
pub use docx::DocxCodec;
pub use layout::{LaidOutItem, LaidOutLine, LaidOutPage, LaidOutTableCell, LaidOutTableRow, Paginator};
pub use numbering::{AbstractNumbering, NumberFormat, NumberingInstance, NumberingLevel, NumberingRegistry};
pub use revisions::{MarkupMode, Revision, RevisionManager, RevisionType};
pub use styles::{ParagraphProperties, StyleDefinition, StyleSheet, StyleType, TextAlignment};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use wps_core::color::Color;

    #[test]
    fn test_rope_and_attribute_interval_splits() {
        let mut buffer = RopeBuffer::from_str("Hello World");
        let mut attrs = AttributeMap::new();

        // Style "World" (chars 6..11) as Bold
        attrs.apply_style(6..11, RunStyle {
            bold: Some(true),
            ..Default::default()
        });
        assert_eq!(attrs.spans.len(), 1);
        assert_eq!(attrs.spans[0].range, 6..11);

        // Insert "Beautiful " in the middle of "World" (at char 8)
        buffer.insert(8, "Beautiful ");
        attrs.shift_on_insert(8, "Beautiful ".chars().count());

        // The bold span should now expand from 6 to 21
        assert_eq!(attrs.spans[0].range, 6..21);
        assert_eq!(buffer.slice(6..21), "WoBeautiful rld");

        // Delete "Beautiful "
        buffer.delete(8..18);
        attrs.shift_on_delete(8..18);
        assert_eq!(attrs.spans[0].range, 6..11);
        assert_eq!(buffer.slice(6..11), "World");
    }

    #[test]
    fn test_style_cascade_inheritance() {
        let styles = StyleSheet::default();

        // Resolve Heading 1 without direct overrides
        let resolved = styles.resolve_run_style(&RunStyle::default(), None, Some("Heading 1"));
        assert_eq!(resolved.bold, Some(true));
        assert_eq!(resolved.font_size_half_pts, Some(36)); // 18pt
        assert_eq!(resolved.font_family, Some("Inter".to_string())); // Inherited from Normal!

        // Direct override takes precedence
        let direct_override = RunStyle {
            bold: Some(false),
            font_size_half_pts: Some(40),
            ..Default::default()
        };
        let overridden = styles.resolve_run_style(&direct_override, None, Some("Heading 1"));
        assert_eq!(overridden.bold, Some(false));
        assert_eq!(overridden.font_size_half_pts, Some(40));
    }

    #[test]
    fn test_pagination_and_table_header_repeat() {
        let mut doc = Document::new("Pagination Test", "Architect");
        let mut paginator = Paginator::new();

        // Add 60 paragraphs to ensure multi-page spill
        for i in 1..=60 {
            let mut p = ParagraphNode::new(&format!("p-{}", i));
            p.add_run(TextRun::new(&format!("Paragraph {} line content with adequate length for testing layout.", i)));
            doc.add_paragraph(p);
        }

        // Add a 30-row table with is_header_repeat = true
        let mut table = TableNode::default();
        table.grid_columns_twips = vec![3000, 3000];

        let mut header_row = TableRow::default();
        header_row.is_header_repeat = true;
        header_row.is_header = true;
        header_row.cells = vec![
            TableCell {
                elements: vec![BlockNode::Paragraph(ParagraphNode::heading("h1", 3, "Col A"))],
                ..Default::default()
            },
            TableCell {
                elements: vec![BlockNode::Paragraph(ParagraphNode::heading("h2", 3, "Col B"))],
                ..Default::default()
            },
        ];
        table.rows.push(header_row);

        for r in 1..=30 {
            let row = TableRow {
                is_header_repeat: false,
                is_header: false,
                cant_split: false,
                allow_row_split_across_pages: true,
                height_twips: Some(400),
                cells: vec![
                    TableCell {
                        elements: vec![BlockNode::Paragraph(ParagraphNode::new(&format!("r{}-c1", r)))],
                        ..Default::default()
                    },
                    TableCell {
                        elements: vec![BlockNode::Paragraph(ParagraphNode::new(&format!("r{}-c2", r)))],
                        ..Default::default()
                    },
                ],
            };
            table.rows.push(row);
        }

        doc.sections[0].elements.push(BlockNode::Table(table));

        let pages = paginator.paginate_document(&doc);
        assert!(pages.len() >= 2, "Document must paginate across at least 2 pages");

        // Verify that on continuation page, the table header row was repeated
        let page2 = &pages[1];
        let has_table = page2.items.iter().any(|item| matches!(item, LaidOutItem::TableRows(_)));
        if has_table {
            for item in &page2.items {
                if let LaidOutItem::TableRows(rows) = item {
                    assert!(rows[0].is_header, "Table on continuation page must repeat header row!");
                }
            }
        }
    }

    #[test]
    fn test_track_changes_accept_reject_invariants() {
        let mut buffer = RopeBuffer::from_str("Initial text for document.");
        let mut attrs = AttributeMap::new();
        let mut rev_mgr = RevisionManager::new();
        rev_mgr.tracking_enabled = true;

        // 1. Insert "NEW " at index 8
        buffer.insert(8, "NEW ");
        let ins_id = rev_mgr.record_insert(8..12);
        assert_eq!(rev_mgr.revisions.len(), 1);

        // 2. Reject insertion -> should revert buffer to original
        rev_mgr.reject_revision(&ins_id, &mut buffer, &mut attrs);
        assert_eq!(buffer.to_string(), "Initial text for document.");
        assert_eq!(rev_mgr.revisions.len(), 0);

        // 3. Insert and Accept
        buffer.insert(8, "AWESOME ");
        let ins_id2 = rev_mgr.record_insert(8..16);
        rev_mgr.accept_revision(&ins_id2, &mut buffer, &mut attrs);
        assert_eq!(buffer.to_string(), "Initial AWESOME text for document.");
    }

    #[test]
    fn test_mail_merge_batch_generation() {
        let mut template = Document::new("Invoice Template", "Finance");
        let mut p = ParagraphNode::new("p1");
        p.add_run(TextRun::new("Hello «Name», your role is «Role»."));
        template.add_paragraph(p);

        let mut rec1 = BTreeMap::new();
        rec1.insert("Name".to_string(), "Alice".to_string());
        rec1.insert("Role".to_string(), "Staff Engineer".to_string());

        let mut rec2 = BTreeMap::new();
        rec2.insert("Name".to_string(), "Bob".to_string());
        rec2.insert("Role".to_string(), "Principal Architect".to_string());

        let mut rec3 = BTreeMap::new();
        rec3.insert("Name".to_string(), "Carol".to_string());
        rec3.insert("Role".to_string(), "Lead Designer".to_string());

        let dataset = vec![rec1, rec2, rec3];
        let merged = DocumentAutomation::execute_mail_merge(&template, &dataset).expect("Mail merge succeeds");

        assert_eq!(merged.len(), 3);
        assert!(merged[0].sections[0].elements[0].plain_text().contains("Hello Alice, your role is Staff Engineer."));
        assert!(merged[1].sections[0].elements[0].plain_text().contains("Hello Bob, your role is Principal Architect."));
        assert!(merged[2].sections[0].elements[0].plain_text().contains("Hello Carol, your role is Lead Designer."));
    }

    #[test]
    fn test_docx_roundtrip_fidelity() {
        let mut original = Document::new("Quarterly Report", "Executive Team");

        let p1 = ParagraphNode::heading("p1", 1, "Quarterly Financial Overview");
        original.add_paragraph(p1);

        let mut p2 = ParagraphNode::new("p2");
        p2.add_run(TextRun::new("Revenue increased by 28% with ").with_size(11.0));
        p2.add_run(TextRun::new("high operational efficiency.").bold().with_color(Color::rgb(12, 166, 120)));
        original.add_paragraph(p2);

        // 2x2 table with header
        let mut tbl = TableNode::default();
        tbl.grid_columns_twips = vec![4000, 4000];

        let mut r1 = TableRow::default();
        r1.is_header = true;
        r1.cells = vec![
            TableCell { elements: vec![BlockNode::Paragraph(ParagraphNode::heading("th1", 3, "Metric"))], ..Default::default() },
            TableCell { elements: vec![BlockNode::Paragraph(ParagraphNode::heading("th2", 3, "Value"))], ..Default::default() },
        ];
        tbl.rows.push(r1);

        let mut r2 = TableRow::default();
        r2.cells = vec![
            TableCell { elements: vec![BlockNode::Paragraph(ParagraphNode::new("td1"))], ..Default::default() },
            TableCell { elements: vec![BlockNode::Paragraph(ParagraphNode::new("td2"))], ..Default::default() },
        ];
        tbl.rows.push(r2);

        original.sections[0].elements.push(BlockNode::Table(tbl));

        // Export to .docx byte stream
        let bytes = DocxCodec::export_docx(&original).expect("Export DOCX succeeds");
        assert!(!bytes.is_empty());

        // Re-import and assert fidelity
        let imported = DocxCodec::import_docx(&bytes).expect("Import DOCX succeeds");
        assert_eq!(imported.sections[0].elements.len(), 3); // 2 paragraphs + 1 table
        assert_eq!(imported.sections[0].elements[0].plain_text(), "Quarterly Financial Overview");
        assert!(imported.sections[0].elements[1].plain_text().contains("Revenue increased by 28%"));
    }
}
