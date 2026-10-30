//! AST-Grounded `@SlidesAI` Co-Pilot & Presentation Compiler (`wps-ai`)
//!
//! Implements:
//! 1. Deck Outline Generation from Document Text / Topics.
//! 2. Multi-Slide Presentation Compilation:
//!    - Master Scene Graph layouts (`layout-title`, `layout-content`).
//!    - Modern card / metric / timeline shapes (`ParametricShape`).
//!    - Smart Connectors linking multi-stage timeline items.
//!    - Morph transitions between slides.

use crate::provider::{BuiltInLocalEngine, LlmClient, LlmPromptRequest};
use serde::{Deserialize, Serialize};
use wps_core::{Color, Rect};
use wps_slides::connector::SmartConnector;
use wps_slides::morph::{MorphType, SlideTransition};
use wps_slides::scene_graph::{PresentationDeck, Slide};
use wps_slides::shape::{
    ParametricShape, ShapeFill, ShapeGeometryType, ShapeStroke, TextAlignment, TextFrame,
    TextParagraph, TextRun, VerticalAlign,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlideOutlineItem {
    pub title: String,
    pub layout: String, // "TitleHero", "TwoColumnComparison", "KpiMetricsGrid", "ProcessTimeline", "BulletList"
    pub bullets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeckOutline {
    pub title: String,
    pub slides: Vec<SlideOutlineItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PresentationThemeConfig {
    pub primary_color: Color,
    pub secondary_color: Color,
    pub accent_color: Color,
    pub background_color: Color,
    pub font_family: String,
}

impl Default for PresentationThemeConfig {
    fn default() -> Self {
        Self {
            primary_color: Color::from_hex("#1E293B").unwrap_or(Color::MERCURY_INK),
            secondary_color: Color::from_hex("#3B82F6").unwrap_or(Color::MERCURY_PRIMARY),
            accent_color: Color::from_hex("#7048E8").unwrap_or(Color::MERCURY_PRIMARY), // Mercury Iris
            background_color: Color::WHITE,
            font_family: "Inter".to_string(),
        }
    }
}

pub struct SlidesAi;

impl SlidesAi {
    /// Generates structured presentation outline from source text or prompt.
    pub fn generate_presentation_outline(
        topic_or_doc_text: &str,
        slide_count: usize,
        audience_style: &str,
    ) -> DeckOutline {
        let engine = BuiltInLocalEngine::new();
        let req = LlmPromptRequest {
            system_prompt: "You are an executive presentation designer. Return a JSON outline.".to_string(),
            user_prompt: format!(
                "Generate a {}-slide {} presentation on topic: {}",
                slide_count, audience_style, topic_or_doc_text
            ),
            temperature: 0.3,
            max_tokens: 512,
        };

        if let Ok(res) = engine.generate_completion(&req) {
            if let Ok(outline) = serde_json::from_str::<DeckOutline>(&res.content) {
                return outline;
            }
        }

        // Deterministic fallback outline
        let title = if topic_or_doc_text.is_empty() {
            "Executive Overview".to_string()
        } else {
            topic_or_doc_text.lines().next().unwrap_or("Strategic Overview").trim().to_string()
        };

        let mut slides = Vec::new();
        slides.push(SlideOutlineItem {
            title: "Executive Summary".to_string(),
            layout: "TitleHero".to_string(),
            bullets: vec![
                "Comprehensive overview of primary strategic objectives".to_string(),
                "Operational metrics validated across all operating units".to_string(),
            ],
        });

        slides.push(SlideOutlineItem {
            title: "Market Analysis & Opportunity".to_string(),
            layout: "TwoColumnComparison".to_string(),
            bullets: vec![
                "Total Addressable Market expanding at 24% CAGR".to_string(),
                "High customer retention driving recurring revenue expansion".to_string(),
            ],
        });

        slides.push(SlideOutlineItem {
            title: "Financial & Operating Architecture".to_string(),
            layout: "KpiMetricsGrid".to_string(),
            bullets: vec![
                "Gross Margin: 76%".to_string(),
                "Net Retention: 124%".to_string(),
                "Operating Efficiency: 3.2x".to_string(),
            ],
        });

        slides.push(SlideOutlineItem {
            title: "Execution Roadmap".to_string(),
            layout: "ProcessTimeline".to_string(),
            bullets: vec![
                "Phase 1: Architecture & Prototyping".to_string(),
                "Phase 2: Enterprise Deployment".to_string(),
                "Phase 3: Global Scale".to_string(),
            ],
        });

        slides.truncate(slide_count.max(2));

        DeckOutline { title, slides }
    }

    /// Compiles a `DeckOutline` into a full `PresentationDeck` AST.
    pub fn compile_outline_to_presentation(
        outline: &DeckOutline,
        theme: PresentationThemeConfig,
    ) -> PresentationDeck {
        let mut deck = PresentationDeck::new(&outline.title);

        // Customize the initial title slide
        if let Some(first_slide) = deck.slides.first_mut() {
            first_slide.title = outline.title.clone();
            first_slide.layout_id = "layout-title".to_string();

            let mut title_shape = ParametricShape::new(
                "title-hero-card",
                ShapeGeometryType::RoundedRectangle,
                Rect::new(80.0, 140.0, 800.0, 220.0),
            );
            title_shape.fill = ShapeFill::Solid(theme.accent_color);
            title_shape.stroke = Some(ShapeStroke {
                color: theme.primary_color,
                width_px: 1.5,
                dash_style: "solid".to_string(),
            });

            let mut tf = TextFrame::default();
            tf.vertical_align = VerticalAlign::Middle;
            tf.paragraphs.push(TextParagraph {
                runs: vec![TextRun {
                    text: outline.title.clone(),
                    font_family: theme.font_family.clone(),
                    font_size_pt: 32.0,
                    bold: true,
                    italic: false,
                    color: Color::WHITE,
                }],
                alignment: TextAlignment::Center,
                line_spacing: 1.2,
            });
            tf.paragraphs.push(TextParagraph {
                runs: vec![TextRun {
                    text: "Generated by WPS AI Co-Pilot".to_string(),
                    font_family: theme.font_family.clone(),
                    font_size_pt: 16.0,
                    bold: false,
                    italic: true,
                    color: Color::WHITE,
                }],
                alignment: TextAlignment::Center,
                line_spacing: 1.4,
            });
            title_shape.text_frame = Some(tf);
            first_slide.add_shape(title_shape);

            first_slide.transition = Some(SlideTransition {
                name: "Morph".to_string(),
                duration_ms: 1200,
                morph_type: MorphType::ByObject,
            });
        }

        // Add additional slides from outline
        for (idx, item) in outline.slides.iter().enumerate() {
            let slide_id = format!("slide-{}", idx + 2);
            let mut slide = Slide::new(&slide_id, &item.title, "layout-content");
            slide.transition = Some(SlideTransition {
                name: "Morph".to_string(),
                duration_ms: 1000,
                morph_type: MorphType::ByObject,
            });

            match item.layout.as_str() {
                "TwoColumnComparison" => {
                    let col_w = 380.0;
                    let col_h = 320.0;
                    let y = 140.0;

                    for (c_idx, bullet) in item.bullets.iter().take(2).enumerate() {
                        let x = 60.0 + c_idx as f64 * (col_w + 40.0);
                        let mut card = ParametricShape::new(
                            &format!("{}-col-{}", slide_id, c_idx + 1),
                            ShapeGeometryType::RoundedRectangle,
                            Rect::new(x, y, col_w, col_h),
                        );
                        card.fill = ShapeFill::Solid(Color::from_hex("#F8FAFC").unwrap_or(Color::WHITE));
                        card.stroke = Some(ShapeStroke {
                            color: theme.secondary_color,
                            width_px: 1.0,
                            dash_style: "solid".to_string(),
                        });
                        card.text_frame = Some(TextFrame::simple(bullet, 18.0, theme.primary_color));
                        slide.add_shape(card);
                    }
                }
                "KpiMetricsGrid" => {
                    let card_w = 240.0;
                    let card_h = 180.0;
                    let y = 180.0;

                    for (m_idx, metric) in item.bullets.iter().take(3).enumerate() {
                        let x = 60.0 + m_idx as f64 * (card_w + 30.0);
                        let mut card = ParametricShape::new(
                            &format!("{}-kpi-{}", slide_id, m_idx + 1),
                            ShapeGeometryType::RoundedRectangle,
                            Rect::new(x, y, card_w, card_h),
                        );
                        card.fill = ShapeFill::Solid(theme.accent_color);
                        card.text_frame = Some(TextFrame::simple(metric, 20.0, Color::WHITE));
                        slide.add_shape(card);
                    }
                }
                "ProcessTimeline" => {
                    let step_w = 220.0;
                    let step_h = 120.0;
                    let y = 200.0;
                    let mut prev_id: Option<String> = None;

                    for (s_idx, step) in item.bullets.iter().take(3).enumerate() {
                        let x = 60.0 + s_idx as f64 * (step_w + 70.0);
                        let shape_id = format!("{}-step-{}", slide_id, s_idx + 1);
                        let mut card = ParametricShape::new(
                            &shape_id,
                            ShapeGeometryType::RoundedRectangle,
                            Rect::new(x, y, step_w, step_h),
                        );
                        card.fill = ShapeFill::Solid(Color::from_hex("#F1F5F9").unwrap_or(Color::WHITE));
                        card.stroke = Some(ShapeStroke {
                            color: theme.accent_color,
                            width_px: 2.0,
                            dash_style: "solid".to_string(),
                        });
                        card.text_frame = Some(TextFrame::simple(step, 14.0, theme.primary_color));
                        slide.add_shape(card);

                        if let Some(prev) = &prev_id {
                            let connector = SmartConnector::new(
                                &format!("{}-conn-{}", slide_id, s_idx),
                                prev,
                                &shape_id,
                            );
                            slide.add_connector(connector);
                        }
                        prev_id = Some(shape_id);
                    }
                }
                _ => {
                    // Default Bullet List Card
                    let mut card = ParametricShape::new(
                        &format!("{}-body-card", slide_id),
                        ShapeGeometryType::RoundedRectangle,
                        Rect::new(60.0, 130.0, 840.0, 340.0),
                    );
                    card.fill = ShapeFill::Solid(Color::WHITE);
                    card.stroke = Some(ShapeStroke {
                        color: Color::from_hex("#E2E8F0").unwrap_or(Color::MERCURY_INK),
                        width_px: 1.0,
                        dash_style: "solid".to_string(),
                    });

                    let mut tf = TextFrame::default();
                    tf.vertical_align = VerticalAlign::Top;
                    for bullet in &item.bullets {
                        tf.paragraphs.push(TextParagraph {
                            runs: vec![TextRun {
                                text: format!("• {}", bullet),
                                font_family: theme.font_family.clone(),
                                font_size_pt: 16.0,
                                bold: false,
                                italic: false,
                                color: theme.primary_color,
                            }],
                            alignment: TextAlignment::Left,
                            line_spacing: 1.5,
                        });
                    }
                    card.text_frame = Some(tf);
                    slide.add_shape(card);
                }
            }

            deck.add_slide(slide);
        }

        deck
    }
}
