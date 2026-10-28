use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use wps_core::color::Color;
use crate::buffer::RunStyle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StyleType {
    Paragraph,
    Character,
    Table,
    Numbering,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ParagraphProperties {
    pub alignment: Option<TextAlignment>,
    pub indent_left_twips: Option<i32>,
    pub indent_right_twips: Option<i32>,
    pub indent_first_line_twips: Option<i32>, // Negative represents hanging indent
    pub space_before_twips: Option<u32>,
    pub space_after_twips: Option<u32>,
    pub line_spacing_multiple: Option<f64>, // e.g. 1.0, 1.15, 1.5, 2.0
    pub keep_with_next: Option<bool>,
    pub keep_lines_together: Option<bool>,
    pub page_break_before: Option<bool>,
    pub widow_control: Option<bool>,
    pub outline_level: Option<u8>, // 1..=9 for TOC
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TextAlignment {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleDefinition {
    pub id: String,
    pub name: String,
    pub style_type: StyleType,
    pub based_on: Option<String>,
    pub next_style: Option<String>,
    pub run_style: RunStyle,
    pub paragraph_props: Option<ParagraphProperties>,
    pub outline_level: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleSheet {
    pub styles: HashMap<String, StyleDefinition>,
}

impl Default for StyleSheet {
    fn default() -> Self {
        let mut sheet = Self {
            styles: HashMap::new(),
        };
        sheet.load_builtin_defaults();
        sheet
    }
}

impl StyleSheet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_builtin_defaults(&mut self) {
        // Normal (Body text)
        self.add_style(StyleDefinition {
            id: "Normal".to_string(),
            name: "Normal".to_string(),
            style_type: StyleType::Paragraph,
            based_on: None,
            next_style: Some("Normal".to_string()),
            run_style: RunStyle {
                font_family: Some("Inter".to_string()),
                font_size_half_pts: Some(22), // 11pt
                color: Some(Color::rgb(23, 23, 33)), // Mercury Night
                ..Default::default()
            },
            paragraph_props: Some(ParagraphProperties {
                alignment: Some(TextAlignment::Left),
                line_spacing_multiple: Some(1.15),
                space_after_twips: Some(160), // 8pt
                widow_control: Some(true),
                ..Default::default()
            }),
            outline_level: None,
        });

        // Heading 1
        self.add_style(StyleDefinition {
            id: "Heading 1".to_string(),
            name: "Heading 1".to_string(),
            style_type: StyleType::Paragraph,
            based_on: Some("Normal".to_string()),
            next_style: Some("Normal".to_string()),
            run_style: RunStyle {
                font_size_half_pts: Some(36), // 18pt
                bold: Some(true),
                color: Some(Color::rgb(59, 91, 219)), // Mercury Indigo
                ..Default::default()
            },
            paragraph_props: Some(ParagraphProperties {
                space_before_twips: Some(240), // 12pt
                space_after_twips: Some(120),  // 6pt
                keep_with_next: Some(true),
                outline_level: Some(1),
                ..Default::default()
            }),
            outline_level: Some(1),
        });

        // Heading 2
        self.add_style(StyleDefinition {
            id: "Heading 2".to_string(),
            name: "Heading 2".to_string(),
            style_type: StyleType::Paragraph,
            based_on: Some("Normal".to_string()),
            next_style: Some("Normal".to_string()),
            run_style: RunStyle {
                font_size_half_pts: Some(28), // 14pt
                bold: Some(true),
                color: Some(Color::rgb(59, 91, 219)),
                ..Default::default()
            },
            paragraph_props: Some(ParagraphProperties {
                space_before_twips: Some(200), // 10pt
                space_after_twips: Some(80),   // 4pt
                keep_with_next: Some(true),
                outline_level: Some(2),
                ..Default::default()
            }),
            outline_level: Some(2),
        });

        // Heading 3
        self.add_style(StyleDefinition {
            id: "Heading 3".to_string(),
            name: "Heading 3".to_string(),
            style_type: StyleType::Paragraph,
            based_on: Some("Normal".to_string()),
            next_style: Some("Normal".to_string()),
            run_style: RunStyle {
                font_size_half_pts: Some(24), // 12pt
                bold: Some(true),
                ..Default::default()
            },
            paragraph_props: Some(ParagraphProperties {
                space_before_twips: Some(160),
                space_after_twips: Some(40),
                keep_with_next: Some(true),
                outline_level: Some(3),
                ..Default::default()
            }),
            outline_level: Some(3),
        });

        // Title
        self.add_style(StyleDefinition {
            id: "Title".to_string(),
            name: "Title".to_string(),
            style_type: StyleType::Paragraph,
            based_on: Some("Normal".to_string()),
            next_style: Some("Subtitle".to_string()),
            run_style: RunStyle {
                font_size_half_pts: Some(48), // 24pt
                bold: Some(true),
                color: Some(Color::rgb(23, 23, 33)),
                ..Default::default()
            },
            paragraph_props: Some(ParagraphProperties {
                space_before_twips: Some(240),
                space_after_twips: Some(160),
                keep_with_next: Some(true),
                ..Default::default()
            }),
            outline_level: None,
        });

        // Subtitle
        self.add_style(StyleDefinition {
            id: "Subtitle".to_string(),
            name: "Subtitle".to_string(),
            style_type: StyleType::Paragraph,
            based_on: Some("Normal".to_string()),
            next_style: Some("Normal".to_string()),
            run_style: RunStyle {
                font_size_half_pts: Some(28), // 14pt
                italic: Some(true),
                color: Some(Color::rgb(85, 78, 123)),
                ..Default::default()
            },
            paragraph_props: Some(ParagraphProperties {
                space_after_twips: Some(120),
                ..Default::default()
            }),
            outline_level: None,
        });

        // Quote
        self.add_style(StyleDefinition {
            id: "Quote".to_string(),
            name: "Quote".to_string(),
            style_type: StyleType::Paragraph,
            based_on: Some("Normal".to_string()),
            next_style: Some("Normal".to_string()),
            run_style: RunStyle {
                italic: Some(true),
                ..Default::default()
            },
            paragraph_props: Some(ParagraphProperties {
                indent_left_twips: Some(720),  // 0.5 in
                indent_right_twips: Some(720),
                space_before_twips: Some(120),
                space_after_twips: Some(120),
                ..Default::default()
            }),
            outline_level: None,
        });
    }

    pub fn add_style(&mut self, style: StyleDefinition) {
        self.styles.insert(style.id.clone(), style);
    }

    pub fn get_style(&self, id: &str) -> Option<&StyleDefinition> {
        self.styles.get(id)
    }

    /// Resolves RunStyle through the `based_on` inheritance chain with cycle protection
    pub fn resolve_run_style(
        &self,
        direct: &RunStyle,
        char_style_id: Option<&str>,
        para_style_id: Option<&str>,
    ) -> RunStyle {
        let mut resolved = RunStyle::default();

        // 1. Inherit from base paragraph style chain
        if let Some(pid) = para_style_id {
            let mut visited = HashSet::new();
            let mut chain = Vec::new();
            let mut curr = Some(pid);

            while let Some(id) = curr {
                if !visited.insert(id) {
                    break; // Cycle detected
                }
                if let Some(style) = self.get_style(id) {
                    chain.push(style.run_style.clone());
                    curr = style.based_on.as_deref();
                } else {
                    break;
                }
            }

            // Apply from root of inheritance down to leaf
            for s in chain.into_iter().rev() {
                resolved = resolved.merge_with(&s);
            }
        }

        // 2. Inherit from character style chain
        if let Some(cid) = char_style_id {
            let mut visited = HashSet::new();
            let mut chain = Vec::new();
            let mut curr = Some(cid);

            while let Some(id) = curr {
                if !visited.insert(id) {
                    break;
                }
                if let Some(style) = self.get_style(id) {
                    chain.push(style.run_style.clone());
                    curr = style.based_on.as_deref();
                } else {
                    break;
                }
            }

            for s in chain.into_iter().rev() {
                resolved = resolved.merge_with(&s);
            }
        }

        // 3. Direct overrides have highest priority
        resolved = resolved.merge_with(direct);
        resolved
    }

    /// Resolves ParagraphProperties through inheritance chain
    pub fn resolve_paragraph_props(
        &self,
        direct: &ParagraphProperties,
        para_style_id: Option<&str>,
    ) -> ParagraphProperties {
        let mut resolved = ParagraphProperties::default();

        if let Some(pid) = para_style_id {
            let mut visited = HashSet::new();
            let mut chain = Vec::new();
            let mut curr = Some(pid);

            while let Some(id) = curr {
                if !visited.insert(id) {
                    break;
                }
                if let Some(style) = self.get_style(id) {
                    if let Some(ref p) = style.paragraph_props {
                        chain.push(p.clone());
                    }
                    curr = style.based_on.as_deref();
                } else {
                    break;
                }
            }

            for p in chain.into_iter().rev() {
                resolved = Self::merge_paragraph_props(&resolved, &p);
            }
        }

        Self::merge_paragraph_props(&resolved, direct)
    }

    fn merge_paragraph_props(base: &ParagraphProperties, over: &ParagraphProperties) -> ParagraphProperties {
        ParagraphProperties {
            alignment: over.alignment.or(base.alignment),
            indent_left_twips: over.indent_left_twips.or(base.indent_left_twips),
            indent_right_twips: over.indent_right_twips.or(base.indent_right_twips),
            indent_first_line_twips: over.indent_first_line_twips.or(base.indent_first_line_twips),
            space_before_twips: over.space_before_twips.or(base.space_before_twips),
            space_after_twips: over.space_after_twips.or(base.space_after_twips),
            line_spacing_multiple: over.line_spacing_multiple.or(base.line_spacing_multiple),
            keep_with_next: over.keep_with_next.or(base.keep_with_next),
            keep_lines_together: over.keep_lines_together.or(base.keep_lines_together),
            page_break_before: over.page_break_before.or(base.page_break_before),
            widow_control: over.widow_control.or(base.widow_control),
            outline_level: over.outline_level.or(base.outline_level),
        }
    }
}
