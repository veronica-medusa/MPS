use crate::scene_graph::{PresentationDeck, Slide};
use crate::shape::{ParametricShape, ShapeGeometryType};
use quick_xml::events::Event;
use quick_xml::Reader;
use wps_core::Rect;
use wps_ooxml::{
    ContentTypes, OoxmlError, OoxmlPackage, Relationships, CT_PPTX_PRESENTATION, CT_PPTX_SLIDE,
    REL_OFFICE_DOCUMENT, REL_SLIDE,
};

pub struct PptxCodec;

impl PptxCodec {
    pub fn export_pptx(deck: &PresentationDeck) -> Result<Vec<u8>, OoxmlError> {
        let mut pkg = OoxmlPackage::new();

        // 1. [Content_Types].xml
        let mut ct = ContentTypes::new();
        ct.overrides.insert("/ppt/presentation.xml".to_string(), CT_PPTX_PRESENTATION.to_string());
        for i in 1..=deck.slides.len() {
            ct.overrides.insert(format!("/ppt/slides/slide{}.xml", i), CT_PPTX_SLIDE.to_string());
        }
        pkg.set_part_string("[Content_Types].xml", &ct.to_xml());

        // 2. _rels/.rels
        let mut root_rels = Relationships::new();
        root_rels.add("rId1", REL_OFFICE_DOCUMENT, "ppt/presentation.xml");
        pkg.set_part_string("_rels/.rels", &root_rels.to_xml());

        // 3. ppt/_rels/presentation.xml.rels
        let mut pres_rels = Relationships::new();
        for (idx, _) in deck.slides.iter().enumerate() {
            pres_rels.add(&format!("rId{}", idx + 1), REL_SLIDE, &format!("slides/slide{}.xml", idx + 1));
        }
        pkg.set_part_string("ppt/_rels/presentation.xml.rels", &pres_rels.to_xml());

        // 4. ppt/presentation.xml
        let mut pres_xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<p:presentation xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><p:sldIdLst>");
        for (idx, _) in deck.slides.iter().enumerate() {
            pres_xml.push_str(&format!("<p:sldId id=\"{}\" r:id=\"rId{}\"/>", 256 + idx, idx + 1));
        }
        pres_xml.push_str("</p:sldIdLst></p:presentation>");
        pkg.set_part_string("ppt/presentation.xml", &pres_xml);

        // 5. ppt/slides/slideX.xml
        for (idx, slide) in deck.slides.iter().enumerate() {
            let mut sld_xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<p:sld xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\" xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\"><p:cSld><p:spTree>");

            for shape in &slide.shapes {
                sld_xml.push_str(&format!(
                    "<p:sp><p:nvSpPr><p:cNvPr id=\"{}\" name=\"{}\"/><p:cNvSpPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/></a:xfrm></p:spPr></p:sp>",
                    quick_xml::escape::escape(&shape.id),
                    quick_xml::escape::escape(&shape.name),
                    (shape.bounds.x * 12700.0).round() as i64,
                    (shape.bounds.y * 12700.0).round() as i64,
                    (shape.bounds.width * 12700.0).round() as i64,
                    (shape.bounds.height * 12700.0).round() as i64,
                ));
            }

            sld_xml.push_str("</p:spTree></p:cSld></p:sld>");
            pkg.set_part_string(&format!("ppt/slides/slide{}.xml", idx + 1), &sld_xml);
        }

        pkg.to_bytes()
    }

    pub fn import_pptx(bytes: &[u8]) -> Result<PresentationDeck, OoxmlError> {
        let pkg = OoxmlPackage::from_bytes(bytes)?;
        let mut deck = PresentationDeck::new("Imported Presentation");
        deck.slides.clear();

        let mut slide_idx = 1;
        while pkg.has_part(&format!("ppt/slides/slide{}.xml", slide_idx)) {
            let xml_str = pkg.get_part_string(&format!("ppt/slides/slide{}.xml", slide_idx))?;
            let mut slide = Slide::new(&format!("slide-{}", slide_idx), &format!("Slide {}", slide_idx), "layout-content");

            let mut reader = Reader::from_str(&xml_str);
            reader.config_mut().trim_text(true);

            let mut current_id = String::new();
            let mut current_name = String::new();
            let mut current_rect = Rect::ZERO;

            let mut buf = Vec::new();
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => match e.local_name().as_ref() {
                        b"cNvPr" => {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"id" {
                                    current_id = String::from_utf8_lossy(&attr.value).to_string();
                                } else if attr.key.as_ref() == b"name" {
                                    current_name = String::from_utf8_lossy(&attr.value).to_string();
                                }
                            }
                        }
                        b"off" => {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"x" {
                                    if let Ok(v) = String::from_utf8_lossy(&attr.value).parse::<f64>() {
                                        current_rect.x = v / 12700.0;
                                    }
                                } else if attr.key.as_ref() == b"y" {
                                    if let Ok(v) = String::from_utf8_lossy(&attr.value).parse::<f64>() {
                                        current_rect.y = v / 12700.0;
                                    }
                                }
                            }
                        }
                        b"ext" => {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"cx" {
                                    if let Ok(v) = String::from_utf8_lossy(&attr.value).parse::<f64>() {
                                        current_rect.width = v / 12700.0;
                                    }
                                } else if attr.key.as_ref() == b"cy" {
                                    if let Ok(v) = String::from_utf8_lossy(&attr.value).parse::<f64>() {
                                        current_rect.height = v / 12700.0;
                                    }
                                }
                            }
                            if !current_id.is_empty() {
                                let mut shape = ParametricShape::new(&current_id, ShapeGeometryType::Rectangle, current_rect);
                                shape.name = current_name.clone();
                                slide.shapes.push(shape);
                                current_id.clear();
                            }
                        }
                        _ => {}
                    },
                    Ok(Event::Eof) => break,
                    Err(e) => return Err(OoxmlError::Xml(e.to_string())),
                    _ => {}
                }
                buf.clear();
            }

            deck.add_slide(slide);
            slide_idx += 1;
        }

        if deck.slides.is_empty() {
            deck.add_slide(Slide::new("slide-1", "Default Slide", "layout-title"));
        }

        Ok(deck)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pptx_roundtrip() {
        let mut deck = PresentationDeck::new("My Presentation");
        let mut s1 = ParametricShape::new("sp1", ShapeGeometryType::Rectangle, Rect::new(50.0, 50.0, 200.0, 100.0));
        s1.name = "Box1".to_string();
        deck.slides[0].shapes.push(s1);

        let bytes = PptxCodec::export_pptx(&deck).unwrap();
        assert!(!bytes.is_empty());

        let imported = PptxCodec::import_pptx(&bytes).unwrap();
        assert_eq!(imported.slides.len(), 1);
        assert_eq!(imported.slides[0].shapes.len(), 1);
        assert_eq!(imported.slides[0].shapes[0].name, "Box1");
    }
}
