use wps_core::{Color, Point, Rect};
use wps_render::DisplayList;
use wps_slides::{
    animator::{AnimationCategory, AnimationEffect, KeyframeAnimation, SlideTimeline},
    connector::{ConnectionPort, ConnectorRouting, SmartConnector},
    morph::{MorphInterpolator, MorphType, SlideTransition},
    pptx::PptxCodec,
    scene_graph::{PresentationDeck, Slide, SlideNode},
    shape::{ParametricShape, ShapeFill, ShapeGeometryType},
    theme::SlideTheme,
};

#[test]
fn test_master_layout_slide_inheritance() {
    let mut deck = PresentationDeck::new("Enterprise Architecture Deck");
    deck.master.theme = SlideTheme::coral_sunset();

    let mut slide = Slide::new("slide-1", "Architecture Blueprint", "layout-title");
    let mut shape = ParametricShape::new("node-core", ShapeGeometryType::RoundedRectangle, Rect::new(100.0, 100.0, 200.0, 80.0));
    shape.fill = ShapeFill::Solid(deck.master.theme.color_scheme.accent1);
    slide.add_shape(shape);

    let mut test_node = ParametricShape::new("test-node", ShapeGeometryType::Rectangle, Rect::new(50.0, 50.0, 100.0, 100.0));
    test_node.fill = ShapeFill::None;
    test_node.stroke = None;

    let (fill, stroke, width, font) = deck.resolve_node_properties(&test_node, 0);

    // Verify cascading property resolution: unstyled node inherits master theme accent1
    assert_eq!(fill, deck.master.theme.color_scheme.accent1);
    assert_eq!(stroke, deck.master.theme.color_scheme.dark1);
    assert_eq!(width, 1.0);
    assert_eq!(font, deck.master.theme.font_scheme.minor_font);

    // Verify scene graph display list composition
    let mut dl = DisplayList::new();
    slide.render_to_display_list(&deck, &mut dl);
    assert!(!dl.is_empty());
}

#[test]
fn test_parametric_shapes_and_yellow_handles() {
    let mut rounded = ParametricShape::new("r1", ShapeGeometryType::RoundedRectangle, Rect::new(50.0, 50.0, 200.0, 100.0));
    let handles = rounded.get_adjustment_handles();
    assert_eq!(handles.len(), 1);
    assert_eq!(handles[0].index, 0);

    // Drag yellow diamond handle to adjust corner radius
    rounded.set_adjustment_handle(0, Point::new(80.0, 50.0)); // 30px / 100px = 0.3
    assert!((rounded.adjustment_values[0] - 0.3).abs() < 1e-4);

    // Verify Star5 handles and geometry
    let star = ParametricShape::new("star", ShapeGeometryType::Star5, Rect::new(100.0, 100.0, 100.0, 100.0));
    let star_handles = star.get_adjustment_handles();
    assert_eq!(star_handles.len(), 1);
    let pts = star.generate_outline_points();
    assert_eq!(pts.len(), 10); // 5 outer + 5 inner vertices

    // Verify RightArrow handles
    let arrow = ParametricShape::new("arr", ShapeGeometryType::RightArrow, Rect::new(0.0, 0.0, 200.0, 100.0));
    let arrow_handles = arrow.get_adjustment_handles();
    assert_eq!(arrow_handles.len(), 2); // Head width and shaft thickness

    // Verify Ray-Casting Hit-Testing
    let rect_shape = ParametricShape::new("rect", ShapeGeometryType::Rectangle, Rect::new(10.0, 10.0, 100.0, 100.0));
    assert!(rect_shape.hit_test(&Point::new(50.0, 50.0)));
    assert!(!rect_shape.hit_test(&Point::new(5.0, 5.0)));
    assert!(!rect_shape.hit_test(&Point::new(150.0, 50.0)));
}

#[test]
fn test_astar_orthogonal_connector_routing() {
    let r1 = Rect::new(50.0, 100.0, 100.0, 60.0);
    let r2 = Rect::new(350.0, 100.0, 100.0, 60.0);
    // Obstacle positioned directly in the line of sight between r1 and r2
    let obstacle = Rect::new(180.0, 80.0, 100.0, 100.0);

    let mut connector = SmartConnector::new("conn-1", "s1", "s2");
    connector.start_port = ConnectionPort::Right;
    connector.end_port = ConnectionPort::Left;
    connector.routing = ConnectorRouting::Orthogonal;

    let waypoints = connector.compute_waypoints_with_obstacles(&r1, &r2, &[obstacle]);

    assert!(waypoints.len() >= 4);
    assert_eq!(waypoints[0], Point::new(150.0, 130.0)); // Right port of r1
    assert_eq!(*waypoints.last().unwrap(), Point::new(350.0, 130.0)); // Left port of r2

    // Verify waypoints route around the obstacle
    let mut penetrates_obstacle = false;
    for pt in &waypoints[1..waypoints.len() - 1] {
        if obstacle.contains_point(pt) {
            penetrates_obstacle = true;
            break;
        }
    }
    assert!(!penetrates_obstacle, "Orthogonal connector must not penetrate obstacle!");
}

#[test]
fn test_morph_transition_interpolation() {
    let mut s1 = ParametricShape::new("logo_a", ShapeGeometryType::Rectangle, Rect::new(50.0, 50.0, 100.0, 100.0));
    s1.name = "!!BrandLogo".to_string();
    s1.fill = ShapeFill::Solid(Color::from_hex("#3b5bdb").unwrap());

    let mut s2 = ParametricShape::new("logo_b", ShapeGeometryType::Star5, Rect::new(250.0, 150.0, 200.0, 200.0));
    s2.name = "!!BrandLogo".to_string(); // Matching !! identifier across different geometries!
    s2.fill = ShapeFill::Solid(Color::from_hex("#e8590c").unwrap());

    let shapes_a = vec![s1];
    let shapes_b = vec![s2];

    // Morph midpoint evaluation at t = 0.5
    let morphed = MorphInterpolator::interpolate_shapes(&shapes_a, &shapes_b, 0.5);

    assert_eq!(morphed.len(), 1);
    let m = &morphed[0];
    assert_eq!(m.name, "!!BrandLogo");
    assert_eq!(m.bounds, Rect::new(150.0, 100.0, 150.0, 150.0)); // Midpoint bounds

    // Cross-geometry morphing generates 64-vertex smooth resampled outline
    assert!(m.freeform_points.is_some());
    assert_eq!(m.freeform_points.as_ref().unwrap().len(), 64);
}

#[test]
fn test_animation_timeline_state_machine() {
    let mut timeline = SlideTimeline::new();
    let mut anim1 = KeyframeAnimation::new("shape-1", AnimationCategory::Entrance, AnimationEffect::Fade);
    anim1.duration_ms = 500;
    timeline.add_animation(anim1);

    let mut anim2 = KeyframeAnimation::new("shape-2", AnimationCategory::Entrance, AnimationEffect::FlyIn);
    anim2.duration_ms = 800;
    timeline.add_animation(anim2);

    let stage = Rect::new(0.0, 0.0, 960.0, 540.0);

    // Initial state before step 0: future entrance shapes are hidden (opacity 0)
    let state_future = timeline.evaluate_timeline_at(0, 0, &stage);
    let s2_state = state_future.get("shape-2");
    assert!(s2_state.is_some());

    // Midpoint of step 0 (elapsed 250ms of 500ms duration)
    let state_mid = timeline.evaluate_timeline_at(0, 250, &stage);
    let s1_state = state_mid.get("shape-1").unwrap();
    assert!(s1_state.opacity > 0.0 && s1_state.opacity <= 1.0);

    // Fully completed step 0 (elapsed 600ms)
    let state_done = timeline.evaluate_timeline_at(0, 600, &stage);
    let s1_done = state_done.get("shape-1").unwrap();
    assert_eq!(s1_done.opacity, 1.0);
}

#[test]
fn test_pptx_roundtrip_fidelity() {
    let mut deck = PresentationDeck::new("Quarterly Strategy");
    let mut slide1 = Slide::new("slide-1", "Executive Summary", "layout-title");
    slide1.transition = Some(SlideTransition {
        name: "Morph".to_string(),
        duration_ms: 1500,
        morph_type: MorphType::ByObject,
    });

    let mut s1 = ParametricShape::new("box-1", ShapeGeometryType::RoundedRectangle, Rect::new(80.0, 120.0, 300.0, 150.0));
    s1.name = "!!StrategyBox".to_string();
    s1.fill = ShapeFill::Solid(Color::from_hex("#5266eb").unwrap());
    slide1.add_shape(s1);

    deck.slides[0] = slide1;

    // Export to .pptx byte stream
    let pptx_bytes = PptxCodec::export_pptx(&deck).expect("Exporting PPTX should succeed");
    assert!(!pptx_bytes.is_empty());

    // Import from .pptx byte stream
    let imported_deck = PptxCodec::import_pptx(&pptx_bytes).expect("Importing PPTX should succeed");
    assert_eq!(imported_deck.slides.len(), 1);

    let imp_slide = &imported_deck.slides[0];
    assert_eq!(imp_slide.nodes.len(), 1);

    if let SlideNode::AutoShape(imp_shape) = &imp_slide.nodes[0] {
        assert_eq!(imp_shape.name, "!!StrategyBox");
        assert_eq!(imp_shape.bounds.width.round(), 300.0);
        assert_eq!(imp_shape.bounds.height.round(), 150.0);
    } else {
        panic!("Imported node should be AutoShape");
    }

    assert_eq!(imp_slide.transition.as_ref().map(|t| t.name.as_str()), Some("Morph"));
}
