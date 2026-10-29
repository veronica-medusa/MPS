// OpenWPS Presentation Core Engine

pub mod animator;
pub mod boolean_ops;
pub mod connector;
pub mod morph;
pub mod pptx;
pub mod presenter;
pub mod scene_graph;
pub mod shape;
pub mod theme;

pub use animator::{
    AnimatedTransform, AnimationCategory, AnimationEffect, Easing, KeyframeAnimation, SlideTimeline,
    TriggerType,
};
pub use boolean_ops::{BooleanOp, BooleanShapeEngine};
pub use connector::{ArrowheadType, ConnectionPort, ConnectorRouting, SmartConnector};
pub use morph::{MorphInterpolator, MorphType, SlideTransition};
pub use pptx::PptxCodec;
pub use presenter::{DisplayMode, InkStroke, LaserPointer, SlideshowSession};
pub use scene_graph::{
    LayoutType, PlaceholderNode, PlaceholderType, PresentationDeck, Slide, SlideComment,
    SlideLayout, SlideMaster, SlideNode, SlideSizeEmu, SlideTableNode, EMU_PER_INCH, EMU_PER_PT,
};
pub use shape::{
    AdjustmentHandle, ParametricShape, ShapeFill, ShapeGeometryType, ShapeStroke, TextAlignment,
    TextAutoFit, TextFrame, TextParagraph, TextRun, VerticalAlign,
};
pub use theme::{SlideTheme, ThemeColorScheme, ThemeFontScheme};
