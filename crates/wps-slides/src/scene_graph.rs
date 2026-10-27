use crate::shape::ParametricShape;
use crate::theme::SlideTheme;
use serde::{Deserialize, Serialize};
use wps_core::{Color, Rect};
use wps_render::{DisplayList, DrawCommand};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutType {
    Title,
    TitleAndContent,
    SectionHeader,
    TwoContent,
    Comparison,
    TitleOnly,
    Blank,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlideLayout {
    pub id: String,
    pub name: String,
    pub layout_type: LayoutType,
    pub placeholder_shapes: Vec<ParametricShape>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlideMaster {
    pub id: String,
    pub theme: SlideTheme,
    pub background_color: Color,
    pub background_shapes: Vec<ParametricShape>,
    pub layouts: Vec<SlideLayout>,
}

impl Default for SlideMaster {
    fn default() -> Self {
        Self {
            id: "master-1".to_string(),
            theme: SlideTheme::mercury_modern(),
            background_color: Color::WHITE,
            background_shapes: Vec::new(),
            layouts: vec![
                SlideLayout {
                    id: "layout-title".to_string(),
                    name: "Title Slide".to_string(),
                    layout_type: LayoutType::Title,
                    placeholder_shapes: Vec::new(),
                },
                SlideLayout {
                    id: "layout-content".to_string(),
                    name: "Title and Content".to_string(),
                    layout_type: LayoutType::TitleAndContent,
                    placeholder_shapes: Vec::new(),
                },
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Slide {
    pub id: String,
    pub title: String,
    pub layout_id: String,
    pub shapes: Vec<ParametricShape>,
    pub speaker_notes: String,
    pub transition_name: Option<String>,
}

impl Slide {
    pub fn new(id: &str, title: &str, layout_id: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            layout_id: layout_id.to_string(),
            shapes: Vec::new(),
            speaker_notes: String::new(),
            transition_name: None,
        }
    }

    pub fn render_to_display_list(&self, master: &SlideMaster, dl: &mut DisplayList) {
        // 1. Render Master background
        dl.push(DrawCommand::FillRect {
            rect: Rect::new(0.0, 0.0, 960.0, 540.0), // Standard 16:9 canvas
            color: master.background_color,
        });

        // 2. Render Master decorative shapes
        for s in &master.background_shapes {
            s.render_to_display_list(dl);
        }

        // 3. Render Slide shapes
        for s in &self.shapes {
            s.render_to_display_list(dl);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PresentationDeck {
    pub title: String,
    pub master: SlideMaster,
    pub slides: Vec<Slide>,
}

impl PresentationDeck {
    pub fn new(title: &str) -> Self {
        let master = SlideMaster::default();
        let first_slide = Slide::new("slide-1", title, "layout-title");
        Self {
            title: title.to_string(),
            master,
            slides: vec![first_slide],
        }
    }

    pub fn add_slide(&mut self, slide: Slide) {
        self.slides.push(slide);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slide_master_inheritance() {
        let deck = PresentationDeck::new("Q1 Deck");
        let mut dl = DisplayList::new();
        deck.slides[0].render_to_display_list(&deck.master, &mut dl);

        assert!(!dl.is_empty());
        assert_eq!(deck.slides.len(), 1);
    }
}
