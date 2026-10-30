pub mod mutations;
pub mod pdf_ai;
pub mod provider;
pub mod router;
pub mod sheets_ai;
pub mod slides_ai;
pub mod writer_ai;

pub use mutations::{AiMutation, MutationApplicator};
pub use pdf_ai::{PdfAi, PdfCitationChip, PdfIndexedBlock, PdfSpatialIndex};
pub use provider::{BuiltInLocalEngine, LlmClient, LlmPromptRequest, LlmPromptResponse, LlmProviderConfig};
pub use router::{AgentRouter, AiDomain, AiResponse, CitationBadge};
pub use sheets_ai::{
    ChartRecommendation, ColumnSchema, ColumnType, CorrelationPair, PivotRecommendation,
    SheetAnomaly, SheetInsightsReport, SheetSchema, SheetsAi,
};
pub use slides_ai::{
    DeckOutline, PresentationThemeConfig, SlideOutlineItem, SlidesAi,
};
pub use writer_ai::{RewriteTone, WriterAi, WriterAiResult};

#[cfg(test)]
mod tests {
    use super::*;
    use wps_core::geometry::Rect;
    use wps_sheets::cell::CellCoordinate;
    use wps_sheets::sparse_matrix::SparseMatrix;
    use wps_writer::ast::{Document as WriterDocument, Paragraph, TextRun};
    use wps_slides::scene_graph::PresentationDeck as SlidesDeck;
    use wps_pdf::{PdfDocument, PdfPage, PdfTextElement};
    use wps_core::color::Color;

    #[test]
    fn test_writer_ai_outline_and_rewrite() {
        let mut doc = WriterDocument::new("Strategic Report", "Author");
        let mut p = Paragraph::new("p-1");
        p.add_run(TextRun::new("We made some good progress this quarter."));
        doc.add_paragraph(p);

        // 1. Rewrite formal
        let resp = AgentRouter::dispatch(
            AiDomain::Writer,
            "rewrite in formal tone",
            Some("We made some good progress this quarter."),
            None,
        );

        assert_eq!(resp.mutations.len(), 1);
        let applied = MutationApplicator::apply_to_writer(&mut doc, &resp.mutations[0]);
        assert!(applied);
        match &doc.sections[0].elements[0] {
            wps_writer::ast::BlockElement::Paragraph(p) => {
                assert!(p.plain_text().contains("substantial operational progress"));
            }
            _ => panic!("Expected paragraph"),
        }

        // 2. Outline generation
        let outline_resp = AgentRouter::dispatch(
            AiDomain::Writer,
            "generate outline for expansion plan",
            Some("Expansion Plan"),
            None,
        );
        assert_eq!(outline_resp.mutations.len(), 4);
        for m in &outline_resp.mutations {
            MutationApplicator::apply_to_writer(&mut doc, m);
        }
        assert!(doc.sections[0].elements.len() >= 5);
    }

    #[test]
    fn test_sheets_ai_formula_and_table_generation() {
        let mut matrix = SparseMatrix::new();

        // 1. Formula synthesis
        let resp = AgentRouter::dispatch(
            AiDomain::Sheets,
            "generate an xlookup for product code",
            None,
            None,
        );
        assert_eq!(resp.mutations.len(), 1);
        MutationApplicator::apply_to_sheets(&mut matrix, &resp.mutations[0]);

        let b2 = matrix.get_cell(CellCoordinate::new(1, 1)).unwrap();
        assert!(b2.formula.as_ref().unwrap().contains("XLOOKUP"));

        // 2. Table synthesis
        let table_resp = AgentRouter::dispatch(
            AiDomain::Sheets,
            "create financial overview table",
            None,
            None,
        );
        assert_eq!(table_resp.mutations.len(), 1);
        MutationApplicator::apply_to_sheets(&mut matrix, &table_resp.mutations[0]);

        let metric_cell = matrix.get_cell(CellCoordinate::new(0, 0)).unwrap();
        assert_eq!(metric_cell.value, wps_sheets::cell::CellValue::Text("Metric".to_string()));
    }

    #[test]
    fn test_slides_ai_presentation_generation() {
        let mut deck = SlidesDeck::new("Initial Deck");

        let resp = AgentRouter::dispatch(
            AiDomain::Slides,
            "Engineering Architecture",
            None,
            None,
        );

        assert_eq!(resp.mutations.len(), 2);
        for m in &resp.mutations {
            MutationApplicator::apply_to_slides(&mut deck, m);
        }

        assert_eq!(deck.slides.len(), 3); // 1 initial + 2 generated
        assert!(deck.slides[1].title.contains("Architecture: Overview"));
    }

    #[test]
    fn test_pdf_ai_rag_citations() {
        let mut pdf = PdfDocument::new("Annual Review");
        let page = PdfPage {
            page_index: 0,
            width: 595.0,
            height: 842.0,
            rotation: 0,
            crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
            text_elements: vec![
                PdfTextElement {
                    text: "Net profit margin expanded to 31.4% in 2026.".to_string(),
                    bounds: Rect::new(50.0, 100.0, 240.0, 14.0),
                    font_name: "Helvetica".to_string(),
                    font_size: 14.0,
                    color: Color::rgb(0, 0, 0),
                    baseline: 112.0,
                },
            ],
            vector_paths: Vec::new(),
            raw_stream: Vec::new(),
        };
        pdf.add_page(page);

        let resp = AgentRouter::dispatch(
            AiDomain::Pdf,
            "profit margin",
            None,
            Some(&pdf),
        );

        assert_eq!(resp.citations.len(), 1);
        assert_eq!(resp.citations[0].page_index, 0);
        assert_eq!(resp.citations[0].bbox, Rect::new(50.0, 100.0, 240.0, 14.0));

        // Apply citation highlights
        for m in &resp.mutations {
            MutationApplicator::apply_to_pdf(&mut pdf, m);
        }

        assert!(pdf.annotations.contains_key(&0));
        assert_eq!(pdf.annotations.get(&0).unwrap().len(), 1);
    }
}
