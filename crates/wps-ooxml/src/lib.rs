// OpenWPS OOXML Codecs & Package Parser

pub mod error;
pub mod opc;
pub mod package;
pub mod shared_strings;
pub mod styles;

pub use error::OoxmlError;
pub use opc::{
    ContentTypes, Relationship, Relationships, CT_DOCX_MAIN, CT_PPTX_PRESENTATION,
    CT_PPTX_SLIDE, CT_XLSX_SHARED_STRINGS, CT_XLSX_STYLES, CT_XLSX_WORKBOOK, CT_XLSX_WORKSHEET,
    REL_OFFICE_DOCUMENT, REL_SHARED_STRINGS, REL_SLIDE, REL_SLIDE_LAYOUT, REL_SLIDE_MASTER,
    REL_STYLES, REL_THEME, REL_WORKSHEET,
};
pub use package::OoxmlPackage;
pub use shared_strings::SharedStringsTable;
pub use styles::{BorderStyle, CellFormat, FontStyle, StylesSheet};
