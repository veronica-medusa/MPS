//! `wps-ai` — AST-Native WPS AI Co-Pilot Crate Root (`@WriterAI`, `@SheetsAI`, `@SlidesAI`, `@PdfAI`)

pub mod pdf_ai;
pub mod provider;
pub mod sheets_ai;
pub mod slides_ai;
pub mod writer_ai;

pub use pdf_ai::{PdfBBox, PdfChunkKind, PdfCitationChip, PdfGroundedAnswer, PdfSpatialRagIndex};
pub use provider::{AiProviderKind, LlmProviderConfig, OutboundLlmHttpRequest};
pub use sheets_ai::{
    CellAnomalyCard, ColumnCorrelationInsight, ColumnSemanticType, FormulaGenerationResult,
    PivotAndChartRecommendation, SheetColumnDescriptor, SheetDataInsightsReport, SheetTableSchema,
    SheetsAiAgent,
};
pub use slides_ai::{
    CompiledPresentationDeck, CompiledShapeNode, CompiledSlideScene, CompiledSmartConnector,
    DeckOutline, SlideLayoutKind, SlideOutlineCard, SlidesAiAgent,
};
pub use writer_ai::{
    ExecutiveSummaryCard, GrammarIssueSuggestion, RewriteTone, StyleAssignmentMutation,
    WriterAiAgent, WriterParagraphInput, WriterRedlineSpan, WriterRewriteResult,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writer_ai_rewrite_emits_track_changes_and_auto_formats() {
        let para = WriterParagraphInput {
            paragraph_id: 42,
            text: "We utilize this module in order to process a large number of documents.".to_string(),
            current_style_id: "Normal".to_string(),
        };

        let rewrite =
            WriterAiAgent::rewrite_with_track_changes(&para, RewriteTone::Concise, "2026-10-30T12:00:00Z");

        assert_eq!(
            rewrite.rewritten_text,
            "We use this module to process many documents."
        );

        // Must contain both Delete and Insert redline spans attributed to "WPS AI"
        let has_delete = rewrite.redline_spans.iter().any(|s| {
            matches!(s, WriterRedlineSpan::Delete { author, .. } if author == "WPS AI")
        });
        let has_insert = rewrite.redline_spans.iter().any(|s| {
            matches!(s, WriterRedlineSpan::Insert { author, .. } if author == "WPS AI")
        });
        assert!(has_delete, "Expected Track Changes deletion span");
        assert!(has_insert, "Expected Track Changes insertion span");

        // Test Auto-Format Document
        let raw_doc = vec![
            WriterParagraphInput {
                paragraph_id: 1,
                text: "OpenWPS Next-Gen Architecture".to_string(),
                current_style_id: "Normal".to_string(),
            },
            WriterParagraphInput {
                paragraph_id: 2,
                text: "1.2 Core Layout Engine".to_string(),
                current_style_id: "Normal".to_string(),
            },
            WriterParagraphInput {
                paragraph_id: 3,
                text: "• Sub-pixel glyph shaping via cosmic-text".to_string(),
                current_style_id: "Normal".to_string(),
            },
        ];
        let style_mutations = WriterAiAgent::auto_format_document(&raw_doc);
        assert_eq!(style_mutations[0].new_style_id, "Title");
        assert_eq!(style_mutations[1].new_style_id, "Heading 2");
        assert_eq!(style_mutations[2].new_style_id, "List Paragraph");
    }

    #[test]
    fn test_sheets_ai_nl_to_formula_and_zscore_anomaly_detection() {
        let headers = vec![
            "Region".to_string(),
            "Quarter".to_string(),
            "Revenue".to_string(),
        ];
        let rows = vec![
            vec!["APAC".to_string(), "Q1".to_string(), "$120,000".to_string()],
            vec!["EMEA".to_string(), "Q1".to_string(), "$115,000".to_string()],
            vec!["APAC".to_string(), "Q2".to_string(), "$125,000".to_string()],
            vec!["AMER".to_string(), "Q2".to_string(), "$118,000".to_string()],
            vec!["APAC".to_string(), "Q3".to_string(), "$122,000".to_string()],
            vec!["EMEA".to_string(), "Q3".to_string(), "$119,000".to_string()],
            vec!["AMER".to_string(), "Q4".to_string(), "$121,000".to_string()],
            // Inject 50x anomalous spike at Row 9 (1-based) -> C9
            vec!["APAC".to_string(), "Q4".to_string(), "$6,500,000".to_string()],
        ];

        let report =
            SheetsAiAgent::analyze_sheet_insights_and_anomalies("Sales2026", &headers, &rows);

        // Verify anomaly detector catches C9 ($6,500,000)
        assert_eq!(report.anomalies.len(), 1);
        assert_eq!(report.anomalies[0].cell_coord, "C9");
        assert!(report.anomalies[0].z_score > 2.2);

        // Verify NL-to-Formula generation
        let formula_res = SheetsAiAgent::natural_language_to_formula(
            "Sum Revenue where Region is 'APAC'",
            &report.schema,
        );
        assert_eq!(
            formula_res.generated_formula,
            "=SUMIFS(C2:C9, A2:A9, \"APAC\")"
        );
        assert!(formula_res.python_alternative.contains("=PY("));
    }

    #[test]
    fn test_slides_ai_prompt_to_deck_scene_graph_compiler() {
        let outline =
            SlidesAiAgent::generate_outline_from_prompt("Quantum Rust Office Suite", 5);
        assert_eq!(outline.slides.len(), 5);

        let compiled = SlidesAiAgent::compile_outline_to_deck(&outline);
        assert_eq!(compiled.width_emu, 12_192_000);
        assert_eq!(compiled.height_emu, 6_858_000);
        assert_eq!(compiled.slides.len(), 5);
        assert_eq!(compiled.slides[0].transition_effect, "Morph");

        // Verify the ProcessTimeline slide has A* smart connectors linking its chevron cards
        let timeline_slide = compiled
            .slides
            .iter()
            .find(|s| s.layout == SlideLayoutKind::ProcessTimeline)
            .expect("Expected a ProcessTimeline slide");
        assert!(!timeline_slide.connectors.is_empty());
    }

    #[test]
    fn test_pdf_ai_spatial_bbox_rag_citations() {
        let mut rag = PdfSpatialRagIndex::new();

        rag.add_spatial_chunk(
            101,
            0,
            PdfBBox::new(50.0, 100.0, 520.0, 160.0),
            PdfChunkKind::BodyParagraph,
            "Section 1. General Software Architecture and multi-tab workspace overview.".to_string(),
        );

        rag.add_spatial_chunk(
            205,
            2, // Page 3 (0-based index 2)
            PdfBBox::new(72.0, 310.0, 540.0, 395.0),
            PdfChunkKind::TableGrid,
            "Section 8.4 Early Termination Penalty: Customer shall pay a termination fee of $45,000 within 30 days.".to_string(),
        );

        let answer = rag.query_with_bbox_citations("What is the early termination penalty fee?", 2);
        assert!(!answer.citations.is_empty());

        let top_citation = &answer.citations[0];
        assert_eq!(top_citation.block_id, 205);
        assert_eq!(top_citation.page_index, 2);
        assert_eq!(top_citation.bbox, PdfBBox::new(72.0, 310.0, 540.0, 395.0));
        assert!(answer.synthesized_markdown.contains("[1]"));
        assert!(answer.synthesized_markdown.contains("Page 3"));
    }
}
