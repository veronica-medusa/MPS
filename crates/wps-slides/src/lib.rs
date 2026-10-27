// OpenWPS Presentation Core Engine

pub mod animator;
pub mod connector;
pub mod pptx;
pub mod scene_graph;
pub mod shape;
pub mod theme;

pub use animator::{AnimationEffect, KeyframeAnimation, MorphInterpolator, TriggerType};
pub use connector::{ConnectionPort, ConnectorRouting, SmartConnector};
pub use pptx::PptxCodec;
pub use scene_graph::{LayoutType, PresentationDeck, Slide, SlideLayout, SlideMaster};
pub use shape::{ParametricShape, ShapeFill, ShapeGeometryType, ShapeStroke};
pub use theme::{SlideTheme, ThemeColorScheme, ThemeFontScheme};
