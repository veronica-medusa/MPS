use crate::connector::SmartConnector;
use crate::morph::{MorphType, SlideTransition};
use crate::scene_graph::{PresentationDeck, Slide, SlideNode, EMU_PER_PT};
use crate::shape::{ParametricShape, ShapeFill, ShapeGeometryType, TextFrame};
use quick_xml::events::Event;
use quick_xml::Reader;
use wps_core::{Color, Rect};
use wps_ooxml::{
    ContentTypes, OoxmlError, OoxmlPackage, Relationships, CT_PPTX_PRESENTATION, CT_PPTX_SLIDE,
    CT_PPTX_SLIDE_LAYOUT, CT_PPTX_SLIDE_MASTER, CT_PPTX_THEME, REL_OFFICE_DOCUMENT, REL_SLIDE,
    REL_SLIDE_LAYOUT, REL_SLIDE_MASTER, REL_THEME,
};

pub struct PptxCodec;

impl PptxCodec {
    pub fn export_pptx(deck: &PresentationDeck) -> Result<Vec<u8>, OoxmlError> {
        let mut pkg = OoxmlPackage::new();

        // 1. [Content_Types].xml
        let mut ct = ContentTypes::new();
        ct.overrides.insert("/ppt/presentation.xml".to_string(), CT_PPTX_PRESENTATION.to_string());
        ct.overrides.insert("/ppt/slideMasters/slideMaster1.xml".to_string(), CT_PPTX_SLIDE_MASTER.to_string());
        ct.overrides.insert("/ppt/slideLayouts/slideLayout1.xml".to_string(), CT_PPTX_SLIDE_LAYOUT.to_string());
        ct.overrides.insert("/ppt/theme/theme1.xml".to_string(), CT_PPTX_THEME.to_string());

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
        pres_rels.add("rId1", REL_SLIDE_MASTER, "slideMasters/slideMaster1.xml");
        for (idx, _) in deck.slides.iter().enumerate() {
            pres_rels.add(&format!("rId{}", idx + 2), REL_SLIDE, &format!("slides/slide{}.xml", idx + 1));
        }
        pkg.set_part_string("ppt/_rels/presentation.xml.rels", &pres_rels.to_xml());

        // 4. ppt/presentation.xml
        let (slide_cx, slide_cy) = deck.slide_size.dimensions_emu();
        let mut pres_xml = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
            <p:presentation xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\" \
            xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" \
            xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">\n\
            <p:sldMasterIdLst><p:sldMasterId id=\"2147483648\" r:id=\"rId1\"/></p:sldMasterIdLst>\n\
            <p:sldIdLst>".to_string();
        for (idx, _) in deck.slides.iter().enumerate() {
            pres_xml.push_str(&format!("<p:sldId id=\"{}\" r:id=\"rId{}\"/>", 256 + idx, idx + 2));
        }
        pres_xml.push_str(&format!(
            "</p:sldIdLst><p:sldSz cx=\"{}\" cy=\"{}\"/></p:presentation>",
            slide_cx, slide_cy
        ));
        pkg.set_part_string("ppt/presentation.xml", &pres_xml);

        // 5. ppt/theme/theme1.xml
        let theme_color = &deck.master.theme.color_scheme;
        let theme_xml = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
            <a:theme xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" name=\"{}\">\n\
            <a:themeElements><a:clrScheme name=\"Mercury\">\n\
            <a:dk1><a:srgbClr val=\"{:02X}{:02X}{:02X}\"/></a:dk1>\n\
            <a:lt1><a:srgbClr val=\"{:02X}{:02X}{:02X}\"/></a:lt1>\n\
            <a:dk2><a:srgbClr val=\"{:02X}{:02X}{:02X}\"/></a:dk2>\n\
            <a:lt2><a:srgbClr val=\"{:02X}{:02X}{:02X}\"/></a:lt2>\n\
            <a:accent1><a:srgbClr val=\"{:02X}{:02X}{:02X}\"/></a:accent1>\n\
            <a:accent2><a:srgbClr val=\"{:02X}{:02X}{:02X}\"/></a:accent2>\n\
            <a:accent3><a:srgbClr val=\"{:02X}{:02X}{:02X}\"/></a:accent3>\n\
            <a:accent4><a:srgbClr val=\"{:02X}{:02X}{:02X}\"/></a:accent4>\n\
            <a:accent5><a:srgbClr val=\"{:02X}{:02X}{:02X}\"/></a:accent5>\n\
            <a:accent6><a:srgbClr val=\"{:02X}{:02X}{:02X}\"/></a:accent6>\n\
            <a:hlink><a:srgbClr val=\"{:02X}{:02X}{:02X}\"/></a:hlink>\n\
            <a:folHlink><a:srgbClr val=\"{:02X}{:02X}{:02X}\"/></a:folHlink>\n\
            </a:clrScheme></a:themeElements></a:theme>",
            deck.master.theme.name,
            theme_color.dark1.r, theme_color.dark1.g, theme_color.dark1.b,
            theme_color.light1.r, theme_color.light1.g, theme_color.light1.b,
            theme_color.dark2.r, theme_color.dark2.g, theme_color.dark2.b,
            theme_color.light2.r, theme_color.light2.g, theme_color.light2.b,
            theme_color.accent1.r, theme_color.accent1.g, theme_color.accent1.b,
            theme_color.accent2.r, theme_color.accent2.g, theme_color.accent2.b,
            theme_color.accent3.r, theme_color.accent3.g, theme_color.accent3.b,
            theme_color.accent4.r, theme_color.accent4.g, theme_color.accent4.b,
            theme_color.accent5.r, theme_color.accent5.g, theme_color.accent5.b,
            theme_color.accent6.r, theme_color.accent6.g, theme_color.accent6.b,
            theme_color.hyperlink.r, theme_color.hyperlink.g, theme_color.hyperlink.b,
            theme_color.followed_hyperlink.r, theme_color.followed_hyperlink.g, theme_color.followed_hyperlink.b,
        );
        pkg.set_part_string("ppt/theme/theme1.xml", &theme_xml);

        // 6. ppt/slideMasters/slideMaster1.xml & rels
        let master_xml = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
            <p:sldMaster xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\" \
            xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" \
            xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">\n\
            <p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:grpSpPr/></p:nvGrpSpPr></p:spTree></p:cSld>\n\
            <p:sldLayoutIdLst><p:sldLayoutId id=\"2147483649\" r:id=\"rId1\"/></p:sldLayoutIdLst>\n\
            </p:sldMaster>".to_string();
        pkg.set_part_string("ppt/slideMasters/slideMaster1.xml", &master_xml);

        let mut master_rels = Relationships::new();
        master_rels.add("rId1", REL_SLIDE_LAYOUT, "../slideLayouts/slideLayout1.xml");
        master_rels.add("rId2", REL_THEME, "../theme/theme1.xml");
        pkg.set_part_string("ppt/slideMasters/_rels/slideMaster1.xml.rels", &master_rels.to_xml());

        // 7. ppt/slideLayouts/slideLayout1.xml & rels
        let layout_xml = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
            <p:sldLayout xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\" \
            xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" \
            xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" type=\"blank\">\n\
            <p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:grpSpPr/></p:nvGrpSpPr></p:spTree></p:cSld>\n\
            </p:sldLayout>".to_string();
        pkg.set_part_string("ppt/slideLayouts/slideLayout1.xml", &layout_xml);

        let mut layout_rels = Relationships::new();
        layout_rels.add("rId1", REL_SLIDE_MASTER, "../slideMasters/slideMaster1.xml");
        pkg.set_part_string("ppt/slideLayouts/_rels/slideLayout1.xml.rels", &layout_rels.to_xml());

        // 8. Slides: ppt/slides/slideX.xml & rels
        for (idx, slide) in deck.slides.iter().enumerate() {
            let mut sld_xml = String::from(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
                <p:sld xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\" \
                xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" \
                xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" \
                xmlns:p15=\"http://schemas.microsoft.com/office/powerpoint/2012/roamingmstheme\">\n\
                <p:cSld><p:spTree>\n\
                <p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:grpSpPr/></p:nvGrpSpPr>"
            );

            for (node_idx, node) in slide.nodes.iter().enumerate() {
                let shape_counter = node_idx + 2;
                match node {
                    SlideNode::AutoShape(shape) => {
                        let emu_x = (shape.bounds.x * EMU_PER_PT as f64).round() as i64;
                        let emu_y = (shape.bounds.y * EMU_PER_PT as f64).round() as i64;
                        let emu_w = (shape.bounds.width * EMU_PER_PT as f64).round() as i64;
                        let emu_h = (shape.bounds.height * EMU_PER_PT as f64).round() as i64;
                        let rot_val = (shape.rotation_deg * 60000.0).round() as i64;

                        let fill_xml = match &shape.fill {
                            ShapeFill::Solid(c) => format!("<a:solidFill><a:srgbClr val=\"{:02X}{:02X}{:02X}\"/></a:solidFill>", c.r, c.g, c.b),
                            _ => "<a:noFill/>".to_string(),
                        };

                        let stroke_xml = if let Some(stroke) = &shape.stroke {
                            let stroke_emu = (stroke.width_px * 12700.0).round() as i64;
                            format!(
                                "<a:ln w=\"{}\"><a:solidFill><a:srgbClr val=\"{:02X}{:02X}{:02X}\"/></a:solidFill></a:ln>",
                                stroke_emu, stroke.color.r, stroke.color.g, stroke.color.b
                            )
                        } else {
                            "<a:ln><a:noFill/></a:ln>".to_string()
                        };

                        let geom_name = Self::shape_geometry_to_preset(shape.geometry);

                        let mut text_xml = String::new();
                        if let Some(frame) = &shape.text_frame {
                            text_xml.push_str("<p:txBody><a:bodyPr/><a:lstStyle/>");
                            for p in &frame.paragraphs {
                                text_xml.push_str("<a:p>");
                                for r in &p.runs {
                                    text_xml.push_str(&format!(
                                        "<a:r><a:rPr b=\"{}\" i=\"{}\" sz=\"{}\"><a:solidFill><a:srgbClr val=\"{:02X}{:02X}{:02X}\"/></a:solidFill></a:rPr><a:t>{}</a:t></a:r>",
                                        if r.bold { 1 } else { 0 },
                                        if r.italic { 1 } else { 0 },
                                        (r.font_size_pt * 100.0).round() as i64,
                                        r.color.r, r.color.g, r.color.b,
                                        quick_xml::escape::escape(&r.text)
                                    ));
                                }
                                text_xml.push_str("</a:p>");
                            }
                            text_xml.push_str("</p:txBody>");
                        }

                        sld_xml.push_str(&format!(
                            "<p:sp><p:nvSpPr><p:cNvPr id=\"{}\" name=\"{}\"/><p:cNvSpPr/></p:nvSpPr>\
                            <p:spPr><a:xfrm rot=\"{}\"><a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/></a:xfrm>\
                            <a:prstGeom prst=\"{}\"><a:avLst/></a:prstGeom>{}{}</p:spPr>{}</p:sp>",
                            shape_counter,
                            quick_xml::escape::escape(&shape.name),
                            rot_val, emu_x, emu_y, emu_w, emu_h,
                            geom_name, fill_xml, stroke_xml, text_xml
                        ));
                    }
                    SlideNode::Connector(c) => {
                        sld_xml.push_str(&format!(
                            "<p:cxnSp><p:nvCxnSpPr><p:cNvPr id=\"{}\" name=\"Connector {}\"/><p:cNvCxnSpPr/></p:nvCxnSpPr>\
                            <p:spPr><a:prstGeom prst=\"bentConnector3\"><a:avLst/></a:prstGeom>\
                            <a:ln w=\"{}\"><a:solidFill><a:srgbClr val=\"{:02X}{:02X}{:02X}\"/></a:solidFill></a:ln></p:spPr></p:cxnSp>",
                            shape_counter, shape_counter,
                            (c.stroke_width * 12700.0).round() as i64,
                            c.color.r, c.color.g, c.color.b
                        ));
                    }
                    SlideNode::Table(t) => {
                        let emu_x = (t.bounds.x * EMU_PER_PT as f64).round() as i64;
                        let emu_y = (t.bounds.y * EMU_PER_PT as f64).round() as i64;
                        let emu_w = (t.bounds.width * EMU_PER_PT as f64).round() as i64;
                        let emu_h = (t.bounds.height * EMU_PER_PT as f64).round() as i64;

                        sld_xml.push_str(&format!(
                            "<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id=\"{}\" name=\"Table {}\"/><p:cNvGraphicFramePr/></p:nvGraphicFramePr>\
                            <p:xfrm><a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/></p:xfrm>\
                            <a:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/table\">\
                            <a:tbl><a:tblPr/>",
                            shape_counter, shape_counter, emu_x, emu_y, emu_w, emu_h
                        ));

                        for r in 0..t.rows {
                            sld_xml.push_str("<a:tr h=\"300000\">");
                            for c in 0..t.cols {
                                let val = t.cells.get(r).and_then(|row| row.get(c)).map(|s| s.as_str()).unwrap_or("");
                                sld_xml.push_str(&format!(
                                    "<a:tc><a:txBody><a:bodyPr/><a:p><a:r><a:t>{}</a:t></a:r></a:p></a:txBody></a:tc>",
                                    quick_xml::escape::escape(val)
                                ));
                            }
                            sld_xml.push_str("</a:tr>");
                        }
                        sld_xml.push_str("</a:tbl></a:graphicData></a:graphic></p:graphicFrame>");
                    }
                    _ => {}
                }
            }

            sld_xml.push_str("</p:spTree></p:cSld>");

            // Slide Transition (including Morph)
            if let Some(trans) = &slide.transition {
                if trans.name == "Morph" {
                    sld_xml.push_str("<p:transition><p15:morph option=\"byObject\"/></p:transition>");
                } else if trans.name == "Fade" {
                    sld_xml.push_str("<p:transition><p:fade/></p:transition>");
                } else if trans.name == "Push" {
                    sld_xml.push_str("<p:transition><p:push/></p:transition>");
                } else if trans.name == "Wipe" {
                    sld_xml.push_str("<p:transition><p:wipe/></p:transition>");
                }
            }

            sld_xml.push_str("</p:sld>");
            pkg.set_part_string(&format!("ppt/slides/slide{}.xml", idx + 1), &sld_xml);

            // Slide relationships
            let mut sld_rels = Relationships::new();
            sld_rels.add("rId1", REL_SLIDE_LAYOUT, "../slideLayouts/slideLayout1.xml");
            pkg.set_part_string(&format!("ppt/slides/_rels/slide{}.xml.rels", idx + 1), &sld_rels.to_xml());
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
            let mut slide = Slide::new(
                &format!("slide-{}", slide_idx),
                &format!("Slide {}", slide_idx),
                "layout-content",
            );

            let mut reader = Reader::from_str(&xml_str);
            reader.config_mut().trim_text(true);

            let mut current_id = String::new();
            let mut current_name = String::new();
            let mut current_geom = ShapeGeometryType::Rectangle;
            let mut current_rect = Rect::ZERO;
            let mut current_rot = 0.0;
            let mut current_fill_color = Color::MERCURY_PRIMARY;
            let mut current_text = String::new();
            let mut in_shape = false;

            let mut buf = Vec::new();
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                        match e.local_name().as_ref() {
                            b"sp" => {
                                in_shape = true;
                                current_id.clear();
                                current_name.clear();
                                current_geom = ShapeGeometryType::Rectangle;
                                current_rect = Rect::ZERO;
                                current_rot = 0.0;
                                current_fill_color = Color::MERCURY_PRIMARY;
                                current_text.clear();
                            }
                            b"cxnSp" => {
                                in_shape = true;
                                current_id.clear();
                                current_name.clear();
                            }
                            b"cNvPr" => {
                                for attr in e.attributes().flatten() {
                                    if attr.key.as_ref() == b"id" {
                                        current_id = String::from_utf8_lossy(&attr.value).to_string();
                                    } else if attr.key.as_ref() == b"name" {
                                        current_name = String::from_utf8_lossy(&attr.value).to_string();
                                    }
                                }
                            }
                            b"xfrm" => {
                                for attr in e.attributes().flatten() {
                                    if attr.key.as_ref() == b"rot" {
                                        if let Ok(r) = String::from_utf8_lossy(&attr.value).parse::<f64>() {
                                            current_rot = r / 60000.0;
                                        }
                                    }
                                }
                            }
                            b"off" => {
                                for attr in e.attributes().flatten() {
                                    if attr.key.as_ref() == b"x" {
                                        if let Ok(v) = String::from_utf8_lossy(&attr.value).parse::<f64>() {
                                            current_rect.x = v / EMU_PER_PT as f64;
                                        }
                                    } else if attr.key.as_ref() == b"y" {
                                        if let Ok(v) = String::from_utf8_lossy(&attr.value).parse::<f64>() {
                                            current_rect.y = v / EMU_PER_PT as f64;
                                        }
                                    }
                                }
                            }
                            b"ext" => {
                                for attr in e.attributes().flatten() {
                                    if attr.key.as_ref() == b"cx" {
                                        if let Ok(v) = String::from_utf8_lossy(&attr.value).parse::<f64>() {
                                            current_rect.width = v / EMU_PER_PT as f64;
                                        }
                                    } else if attr.key.as_ref() == b"cy" {
                                        if let Ok(v) = String::from_utf8_lossy(&attr.value).parse::<f64>() {
                                            current_rect.height = v / EMU_PER_PT as f64;
                                        }
                                    }
                                }
                            }
                            b"prstGeom" => {
                                for attr in e.attributes().flatten() {
                                    if attr.key.as_ref() == b"prst" {
                                        current_geom = Self::preset_to_shape_geometry(&String::from_utf8_lossy(&attr.value));
                                    }
                                }
                            }
                            b"srgbClr" => {
                                for attr in e.attributes().flatten() {
                                    if attr.key.as_ref() == b"val" {
                                        let hex = String::from_utf8_lossy(&attr.value);
                                        current_fill_color = Color::from_hex(&format!("#{}", hex)).unwrap_or(Color::MERCURY_PRIMARY);
                                    }
                                }
                            }
                            b"morph" => {
                                slide.transition = Some(SlideTransition {
                                    name: "Morph".to_string(),
                                    duration_ms: 1500,
                                    morph_type: MorphType::ByObject,
                                });
                            }
                            _ => {}
                        }
                    }
                    Ok(Event::Text(ref e)) => {
                        if in_shape {
                            let text = e.unescape().unwrap_or_default().to_string();
                            current_text.push_str(&text);
                        }
                    }
                    Ok(Event::End(ref e)) => match e.local_name().as_ref() {
                        b"sp" => {
                            if !current_id.is_empty() {
                                let mut shape = ParametricShape::new(&current_id, current_geom, current_rect);
                                shape.name = current_name.clone();
                                shape.rotation_deg = current_rot;
                                shape.fill = ShapeFill::Solid(current_fill_color);
                                if !current_text.is_empty() {
                                    shape.text_frame = Some(TextFrame::simple(&current_text, 14.0, Color::WHITE));
                                }
                                slide.add_shape(shape);
                            }
                            in_shape = false;
                        }
                        b"cxnSp" => {
                            if !current_id.is_empty() {
                                let connector = SmartConnector::new(&current_id, "start", "end");
                                slide.add_connector(connector);
                            }
                            in_shape = false;
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

    fn shape_geometry_to_preset(geom: ShapeGeometryType) -> &'static str {
        match geom {
            ShapeGeometryType::Rectangle => "rect",
            ShapeGeometryType::RoundedRectangle => "roundRect",
            ShapeGeometryType::Ellipse => "ellipse",
            ShapeGeometryType::Triangle => "triangle",
            ShapeGeometryType::RightTriangle => "rtTriangle",
            ShapeGeometryType::Diamond => "diamond",
            ShapeGeometryType::Parallelogram => "parallelogram",
            ShapeGeometryType::Trapezoid => "trapezoid",
            ShapeGeometryType::Hexagon => "hexagon",
            ShapeGeometryType::Octagon => "octagon",
            ShapeGeometryType::Decagon => "decagon",
            ShapeGeometryType::Dodecagon => "dodecagon",
            ShapeGeometryType::Can => "can",
            ShapeGeometryType::Cube => "cube",
            ShapeGeometryType::Donut => "donut",
            ShapeGeometryType::Heart => "heart",
            ShapeGeometryType::LightningBolt => "lightningBolt",
            ShapeGeometryType::Sun => "sun",
            ShapeGeometryType::Moon => "moon",
            ShapeGeometryType::Cloud => "cloud",
            ShapeGeometryType::Cross => "mathPlus",
            ShapeGeometryType::Star4 => "star4",
            ShapeGeometryType::Star5 => "star5",
            ShapeGeometryType::Star6 => "star6",
            ShapeGeometryType::Star8 => "star8",
            ShapeGeometryType::RightArrow => "rightArrow",
            ShapeGeometryType::LeftArrow => "leftArrow",
            ShapeGeometryType::UpArrow => "upArrow",
            ShapeGeometryType::DownArrow => "downArrow",
            ShapeGeometryType::LeftRightArrow => "leftRightArrow",
            ShapeGeometryType::UpDownArrow => "upDownArrow",
            ShapeGeometryType::QuadArrow => "quadArrow",
            ShapeGeometryType::Chevron => "chevron",
            ShapeGeometryType::RectCallout => "wedgeRectCallout",
            ShapeGeometryType::FlowchartProcess => "flowChartProcess",
            ShapeGeometryType::FlowchartDecision => "flowChartDecision",
            ShapeGeometryType::FlowchartDocument => "flowChartDocument",
            _ => "rect",
        }
    }

    fn preset_to_shape_geometry(preset: &str) -> ShapeGeometryType {
        match preset {
            "rect" => ShapeGeometryType::Rectangle,
            "roundRect" => ShapeGeometryType::RoundedRectangle,
            "ellipse" => ShapeGeometryType::Ellipse,
            "triangle" => ShapeGeometryType::Triangle,
            "rtTriangle" => ShapeGeometryType::RightTriangle,
            "diamond" => ShapeGeometryType::Diamond,
            "parallelogram" => ShapeGeometryType::Parallelogram,
            "trapezoid" => ShapeGeometryType::Trapezoid,
            "hexagon" => ShapeGeometryType::Hexagon,
            "octagon" => ShapeGeometryType::Octagon,
            "decagon" => ShapeGeometryType::Decagon,
            "dodecagon" => ShapeGeometryType::Dodecagon,
            "can" => ShapeGeometryType::Can,
            "cube" => ShapeGeometryType::Cube,
            "donut" => ShapeGeometryType::Donut,
            "heart" => ShapeGeometryType::Heart,
            "lightningBolt" => ShapeGeometryType::LightningBolt,
            "sun" => ShapeGeometryType::Sun,
            "moon" => ShapeGeometryType::Moon,
            "cloud" => ShapeGeometryType::Cloud,
            "star4" => ShapeGeometryType::Star4,
            "star5" => ShapeGeometryType::Star5,
            "star6" => ShapeGeometryType::Star6,
            "star8" => ShapeGeometryType::Star8,
            "rightArrow" => ShapeGeometryType::RightArrow,
            "leftArrow" => ShapeGeometryType::LeftArrow,
            "upArrow" => ShapeGeometryType::UpArrow,
            "downArrow" => ShapeGeometryType::DownArrow,
            "chevron" => ShapeGeometryType::Chevron,
            "wedgeRectCallout" => ShapeGeometryType::RectCallout,
            _ => ShapeGeometryType::Rectangle,
        }
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
        deck.slides[0].add_shape(s1);

        let bytes = PptxCodec::export_pptx(&deck).unwrap();
        assert!(!bytes.is_empty());

        let imported = PptxCodec::import_pptx(&bytes).unwrap();
        assert_eq!(imported.slides.len(), 1);
        assert_eq!(imported.slides[0].nodes.len(), 1);
        assert_eq!(imported.slides[0].nodes[0].id(), "2");
    }
}
