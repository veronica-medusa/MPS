// OpenWPS Writer Core Engine

pub mod ast;
pub mod docx;
pub mod mail_merge;
pub mod paginator;
pub mod track_changes;

pub use ast::{
    BlockElement, Document, FieldElement, FieldType, FloatingObject, InlineElement, PageDimensions,
    PageOrientation, Paragraph, Section, SectionProperties, Table, TableCell, TableRow, TextAlignment,
    TextRun, WrapMode,
};
pub use docx::DocxCodec;
pub use mail_merge::MailMergeEngine;
pub use paginator::{Paginator, RenderedPage};
pub use track_changes::{CommentThread, Revision, RevisionType, TrackChangesManager};
