use std::collections::HashMap;
use wps_core::color::Color;
use wps_core::geometry::{Point, Rect};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PdfError {
    #[error("Invalid PDF syntax: {0}")]
    SyntaxError(String),
    #[error("Object not found: {0}:{1}")]
    ObjectNotFound(u32, u16),
    #[error("Stream decompression error: {0}")]
    DecompressionError(String),
    #[error("Unsupported PDF feature: {0}")]
    Unsupported(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PdfObject {
    Null,
    Boolean(bool),
    Integer(i64),
    Real(f64),
    String(Vec<u8>),
    Name(String),
    Array(Vec<PdfObject>),
    Dictionary(HashMap<String, PdfObject>),
    Stream {
        dict: HashMap<String, PdfObject>,
        data: Vec<u8>,
    },
    Reference {
        obj_num: u32,
        gen_num: u16,
    },
}

impl PdfObject {
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            PdfObject::Integer(i) => Some(*i),
            PdfObject::Real(r) => Some(*r as i64),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            PdfObject::Real(r) => Some(*r),
            PdfObject::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_name(&self) -> Option<&str> {
        match self {
            PdfObject::Name(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_string_lossy(&self) -> Option<String> {
        match self {
            PdfObject::String(bytes) => Some(String::from_utf8_lossy(bytes).to_string()),
            PdfObject::Name(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn as_dict(&self) -> Option<&HashMap<String, PdfObject>> {
        match self {
            PdfObject::Dictionary(map) => Some(map),
            PdfObject::Stream { dict, .. } => Some(dict),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<PdfObject>> {
        match self {
            PdfObject::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn get<'a>(&'a self, key: &str) -> Option<&'a PdfObject> {
        self.as_dict().and_then(|d| d.get(key))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PdfTextElement {
    pub text: String,
    pub bounds: Rect,
    pub font_name: String,
    pub font_size: f64,
    pub color: Color,
    pub baseline: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathOp {
    MoveTo(Point),
    LineTo(Point),
    CurveTo(Point, Point, Point),
    Rectangle(Rect),
    Close,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PdfVectorPath {
    pub operations: Vec<PathOp>,
    pub bounds: Rect,
    pub stroke_color: Option<Color>,
    pub fill_color: Option<Color>,
    pub line_width: f64,
}

#[derive(Debug, Clone)]
pub struct PdfPage {
    pub page_index: usize,
    pub width: f64,
    pub height: f64,
    pub rotation: i32,
    pub crop_box: Rect,
    pub text_elements: Vec<PdfTextElement>,
    pub vector_paths: Vec<PdfVectorPath>,
    pub raw_stream: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct PdfMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub creation_date: Option<String>,
}

pub struct PdfParser<'a> {
    data: &'a [u8],
    pos: usize,
    pub objects: HashMap<(u32, u16), PdfObject>,
    pub trailer: HashMap<String, PdfObject>,
}

impl<'a> PdfParser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            objects: HashMap::new(),
            trailer: HashMap::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<PdfPage>, PdfError> {
        self.parse_objects()?;
        self.extract_pages()
    }

    fn skip_whitespace_and_comments(&mut self) {
        while self.pos < self.data.len() {
            let b = self.data[self.pos];
            if b == b'%' {
                // Comment until end of line
                while self.pos < self.data.len() && self.data[self.pos] != b'\n' && self.data[self.pos] != b'\r' {
                    self.pos += 1;
                }
            } else if b.is_ascii_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        if self.pos < self.data.len() {
            Some(self.data[self.pos])
        } else {
            None
        }
    }

    fn parse_objects(&mut self) -> Result<(), PdfError> {
        self.pos = 0;
        let len = self.data.len();

        while self.pos < len {
            self.skip_whitespace_and_comments();
            if self.pos >= len {
                break;
            }

            // Check for trailer
            if self.starts_with(b"trailer") {
                self.pos += 7;
                self.skip_whitespace_and_comments();
                if let Ok(obj) = self.parse_object() {
                    if let PdfObject::Dictionary(dict) = obj {
                        self.trailer = dict;
                    }
                }
                continue;
            }

            // Check for indirect object: `<number> <number> obj`
            let start = self.pos;
            if let Some(num1) = self.try_parse_integer() {
                self.skip_whitespace_and_comments();
                if let Some(num2) = self.try_parse_integer() {
                    self.skip_whitespace_and_comments();
                    if self.starts_with(b"obj") {
                        self.pos += 3;
                        let obj = self.parse_object()?;
                        self.skip_whitespace_and_comments();
                        if self.starts_with(b"endobj") {
                            self.pos += 6;
                        }
                        self.objects.insert((num1 as u32, num2 as u16), obj);
                        continue;
                    }
                }
            }

            // If not an object, advance by 1
            self.pos = start + 1;
        }

        Ok(())
    }

    fn starts_with(&self, prefix: &[u8]) -> bool {
        if self.pos + prefix.len() <= self.data.len() {
            &self.data[self.pos..self.pos + prefix.len()] == prefix
        } else {
            false
        }
    }

    fn try_parse_integer(&mut self) -> Option<i64> {
        self.skip_whitespace_and_comments();
        let start = self.pos;
        let mut negative = false;
        if self.peek() == Some(b'-') {
            negative = true;
            self.pos += 1;
        } else if self.peek() == Some(b'+') {
            self.pos += 1;
        }

        let num_start = self.pos;
        while self.pos < self.data.len() && self.data[self.pos].is_ascii_digit() {
            self.pos += 1;
        }

        if self.pos == num_start {
            self.pos = start;
            return None;
        }

        // Make sure next char isn't a decimal point (which would make it a float)
        if self.peek() == Some(b'.') {
            self.pos = start;
            return None;
        }

        let slice = std::str::from_utf8(&self.data[num_start..self.pos]).ok()?;
        let val: i64 = slice.parse().ok()?;
        Some(if negative { -val } else { val })
    }

    pub fn parse_object(&mut self) -> Result<PdfObject, PdfError> {
        self.skip_whitespace_and_comments();
        if self.pos >= self.data.len() {
            return Ok(PdfObject::Null);
        }

        let b = self.data[self.pos];
        match b {
            b'n' if self.starts_with(b"null") => {
                self.pos += 4;
                Ok(PdfObject::Null)
            }
            b't' if self.starts_with(b"true") => {
                self.pos += 4;
                Ok(PdfObject::Boolean(true))
            }
            b'f' if self.starts_with(b"false") => {
                self.pos += 5;
                Ok(PdfObject::Boolean(false))
            }
            b'/' => {
                self.pos += 1;
                let start = self.pos;
                while self.pos < self.data.len() {
                    let ch = self.data[self.pos];
                    if ch.is_ascii_whitespace() || b"/()<>[]{}/%".contains(&ch) {
                        break;
                    }
                    self.pos += 1;
                }
                let name = String::from_utf8_lossy(&self.data[start..self.pos]).to_string();
                Ok(PdfObject::Name(name))
            }
            b'(' => self.parse_literal_string(),
            b'<' => {
                if self.pos + 1 < self.data.len() && self.data[self.pos + 1] == b'<' {
                    self.parse_dictionary()
                } else {
                    self.parse_hex_string()
                }
            }
            b'[' => self.parse_array(),
            b'+' | b'-' | b'.' | b'0'..=b'9' => self.parse_number_or_reference(),
            _ => {
                self.pos += 1;
                Ok(PdfObject::Null)
            }
        }
    }

    fn parse_literal_string(&mut self) -> Result<PdfObject, PdfError> {
        self.pos += 1; // skip '('
        let mut result = Vec::new();
        let mut depth = 1;

        while self.pos < self.data.len() {
            let b = self.data[self.pos];
            self.pos += 1;
            match b {
                b'\\' => {
                    if self.pos < self.data.len() {
                        let esc = self.data[self.pos];
                        self.pos += 1;
                        match esc {
                            b'n' => result.push(b'\n'),
                            b'r' => result.push(b'\r'),
                            b't' => result.push(b'\t'),
                            b'b' => result.push(0x08),
                            b'f' => result.push(0x0C),
                            b'(' => result.push(b'('),
                            b')' => result.push(b')'),
                            b'\\' => result.push(b'\\'),
                            _ => result.push(esc),
                        }
                    }
                }
                b'(' => {
                    depth += 1;
                    result.push(b'(');
                }
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    result.push(b')');
                }
                _ => result.push(b),
            }
        }
        Ok(PdfObject::String(result))
    }

    fn parse_hex_string(&mut self) -> Result<PdfObject, PdfError> {
        self.pos += 1; // skip '<'
        let mut hex = Vec::new();
        while self.pos < self.data.len() {
            let b = self.data[self.pos];
            self.pos += 1;
            if b == b'>' {
                break;
            }
            if !b.is_ascii_whitespace() {
                hex.push(b);
            }
        }
        let mut bytes = Vec::new();
        for chunk in hex.chunks(2) {
            let s = std::str::from_utf8(chunk).unwrap_or("00");
            if let Ok(v) = u8::from_str_radix(s, 16) {
                bytes.push(v);
            }
        }
        Ok(PdfObject::String(bytes))
    }

    fn parse_array(&mut self) -> Result<PdfObject, PdfError> {
        self.pos += 1; // skip '['
        let mut elements = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.pos >= self.data.len() || self.data[self.pos] == b']' {
                if self.pos < self.data.len() {
                    self.pos += 1; // skip ']'
                }
                break;
            }
            elements.push(self.parse_object()?);
        }
        Ok(PdfObject::Array(elements))
    }

    fn parse_dictionary(&mut self) -> Result<PdfObject, PdfError> {
        self.pos += 2; // skip '<<'
        let mut dict = HashMap::new();

        loop {
            self.skip_whitespace_and_comments();
            if self.pos >= self.data.len() {
                break;
            }
            if self.starts_with(b">>") {
                self.pos += 2;
                break;
            }

            let key_obj = self.parse_object()?;
            let key = match key_obj {
                PdfObject::Name(k) => k,
                _ => continue,
            };

            let val = self.parse_object()?;
            dict.insert(key, val);
        }

        // Check if a stream follows this dictionary
        self.skip_whitespace_and_comments();
        if self.starts_with(b"stream") {
            self.pos += 6;
            if self.peek() == Some(b'\r') {
                self.pos += 1;
            }
            if self.peek() == Some(b'\n') {
                self.pos += 1;
            }

            let mut stream_data = Vec::new();
            while self.pos < self.data.len() {
                if self.starts_with(b"endstream") {
                    self.pos += 9;
                    break;
                }
                stream_data.push(self.data[self.pos]);
                self.pos += 1;
            }

            // Strip trailing newline if any
            if stream_data.ends_with(b"\n") {
                stream_data.pop();
            }
            if stream_data.ends_with(b"\r") {
                stream_data.pop();
            }

            // Decompress FlateDecode if specified
            if let Some(PdfObject::Name(filter)) = dict.get("Filter") {
                if filter == "FlateDecode" {
                    if let Ok(decompressed) = miniz_oxide::inflate::decompress_to_vec_zlib(&stream_data) {
                        stream_data = decompressed;
                    }
                }
            }

            return Ok(PdfObject::Stream {
                dict,
                data: stream_data,
            });
        }

        Ok(PdfObject::Dictionary(dict))
    }

    fn parse_number_or_reference(&mut self) -> Result<PdfObject, PdfError> {
        let start = self.pos;
        let mut has_dot = false;
        let mut is_first = true;

        while self.pos < self.data.len() {
            let b = self.data[self.pos];
            if (b == b'+' || b == b'-') && is_first {
                self.pos += 1;
            } else if b == b'.' && !has_dot {
                has_dot = true;
                self.pos += 1;
            } else if b.is_ascii_digit() {
                self.pos += 1;
            } else {
                break;
            }
            is_first = false;
        }

        let num_str = std::str::from_utf8(&self.data[start..self.pos]).unwrap_or("0");

        if has_dot {
            let r: f64 = num_str.parse().unwrap_or(0.0);
            return Ok(PdfObject::Real(r));
        }

        let i: i64 = num_str.parse().unwrap_or(0);

        // Check if this integer is followed by another integer and 'R' (indirect reference)
        let saved_pos = self.pos;
        self.skip_whitespace_and_comments();
        if let Some(gen_num) = self.try_parse_integer() {
            self.skip_whitespace_and_comments();
            if self.peek() == Some(b'R') {
                self.pos += 1;
                // Verify R is a standalone delimiter
                if self.pos >= self.data.len() || self.data[self.pos].is_ascii_whitespace() || b"/()<>[]{}/%".contains(&self.data[self.pos]) {
                    return Ok(PdfObject::Reference {
                        obj_num: i as u32,
                        gen_num: gen_num as u16,
                    });
                }
            }
        }

        self.pos = saved_pos;
        Ok(PdfObject::Integer(i))
    }

    pub fn resolve<'b>(&'b self, obj: &'b PdfObject) -> &'b PdfObject {
        match obj {
            PdfObject::Reference { obj_num, gen_num } => {
                if let Some(target) = self.objects.get(&(*obj_num, *gen_num)) {
                    self.resolve(target)
                } else {
                    obj
                }
            }
            _ => obj,
        }
    }

    fn extract_pages(&self) -> Result<Vec<PdfPage>, PdfError> {
        let mut pages = Vec::new();

        // 1. Find Root Catalog
        let root = self.trailer.get("Root")
            .or_else(|| {
                // Search objects for /Type /Catalog
                self.objects.values().find(|o| {
                    if let Some(dict) = o.as_dict() {
                        dict.get("Type").and_then(|t| t.as_name()) == Some("Catalog")
                    } else {
                        false
                    }
                })
            });

        let catalog = match root {
            Some(r) => self.resolve(r),
            None => return Ok(self.fallback_pages()),
        };

        // 2. Traverse Page Tree
        if let Some(pages_ref) = catalog.get("Pages") {
            let pages_node = self.resolve(pages_ref);
            self.collect_pages_from_node(pages_node, &mut pages)?;
        }

        if pages.is_empty() {
            pages = self.fallback_pages();
        }

        Ok(pages)
    }

    fn collect_pages_from_node(&self, node: &PdfObject, pages: &mut Vec<PdfPage>) -> Result<(), PdfError> {
        let dict = match node.as_dict() {
            Some(d) => d,
            None => return Ok(()),
        };

        let node_type = dict.get("Type").and_then(|t| t.as_name()).unwrap_or("");
        if node_type == "Pages" {
            if let Some(kids) = dict.get("Kids").and_then(|k| self.resolve(k).as_array()) {
                for kid in kids {
                    let resolved_kid = self.resolve(kid);
                    self.collect_pages_from_node(resolved_kid, pages)?;
                }
            }
        } else if node_type == "Page" {
            let page_idx = pages.len();
            let (w, h) = self.extract_media_box(dict);
            let rotation = dict.get("Rotate").and_then(|r| self.resolve(r).as_i64()).unwrap_or(0) as i32;
            let crop_box = Rect::new(0.0, 0.0, w, h);

            let content_stream = self.extract_page_content_stream(dict);
            let (text_elements, vector_paths) = self.interpret_content_stream(&content_stream, w, h);

            pages.push(PdfPage {
                page_index: page_idx,
                width: w,
                height: h,
                rotation,
                crop_box,
                text_elements,
                vector_paths,
                raw_stream: content_stream,
            });
        }

        Ok(())
    }

    fn extract_media_box(&self, dict: &HashMap<String, PdfObject>) -> (f64, f64) {
        if let Some(mb) = dict.get("MediaBox").and_then(|m| self.resolve(m).as_array()) {
            if mb.len() >= 4 {
                let w = self.resolve(&mb[2]).as_f64().unwrap_or(595.0) - self.resolve(&mb[0]).as_f64().unwrap_or(0.0);
                let h = self.resolve(&mb[3]).as_f64().unwrap_or(842.0) - self.resolve(&mb[1]).as_f64().unwrap_or(0.0);
                return (w.abs(), h.abs());
            }
        }
        (595.0, 842.0) // Default A4
    }

    fn extract_page_content_stream(&self, dict: &HashMap<String, PdfObject>) -> Vec<u8> {
        let mut stream_bytes = Vec::new();
        if let Some(contents) = dict.get("Contents") {
            let res = self.resolve(contents);
            match res {
                PdfObject::Stream { data, .. } => {
                    stream_bytes.extend_from_slice(data);
                }
                PdfObject::Array(arr) => {
                    for item in arr {
                        if let PdfObject::Stream { data, .. } = self.resolve(item) {
                            stream_bytes.extend_from_slice(data);
                            stream_bytes.push(b' ');
                        }
                    }
                }
                _ => {}
            }
        }
        stream_bytes
    }

    fn fallback_pages(&self) -> Vec<PdfPage> {
        let mut pages = Vec::new();
        for (_i, obj) in self.objects.values().enumerate() {
            if let Some(dict) = obj.as_dict() {
                if dict.get("Type").and_then(|t| t.as_name()) == Some("Page") {
                    let (w, h) = self.extract_media_box(dict);
                    let stream = self.extract_page_content_stream(dict);
                    let (text, vectors) = self.interpret_content_stream(&stream, w, h);
                    pages.push(PdfPage {
                        page_index: pages.len(),
                        width: w,
                        height: h,
                        rotation: 0,
                        crop_box: Rect::new(0.0, 0.0, w, h),
                        text_elements: text,
                        vector_paths: vectors,
                        raw_stream: stream,
                    });
                }
            }
        }

        if pages.is_empty() {
            // Default 1 blank A4 page
            pages.push(PdfPage {
                page_index: 0,
                width: 595.0,
                height: 842.0,
                rotation: 0,
                crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
                text_elements: Vec::new(),
                vector_paths: Vec::new(),
                raw_stream: Vec::new(),
            });
        }
        pages
    }

    pub fn interpret_content_stream(&self, stream: &[u8], _page_width: f64, page_height: f64) -> (Vec<PdfTextElement>, Vec<PdfVectorPath>) {
        let mut text_elements = Vec::new();
        let mut vector_paths = Vec::new();

        let mut pos = 0;
        let mut stack: Vec<PdfObject> = Vec::new();

        // State trackers
        let mut current_font = "Helvetica".to_string();
        let mut current_font_size = 12.0;
        let mut current_fill_color = Color::rgb(0, 0, 0);
        let mut current_stroke_color = Color::rgb(0, 0, 0);
        let mut current_line_width = 1.0;

        let mut text_x = 0.0;
        let mut text_y = 0.0;
        let mut line_x = 0.0;
        let mut line_y = 0.0;

        let mut current_path_ops: Vec<PathOp> = Vec::new();

        while pos < stream.len() {
            // Skip whitespace
            while pos < stream.len() && stream[pos].is_ascii_whitespace() {
                pos += 1;
            }
            if pos >= stream.len() {
                break;
            }

            let b = stream[pos];
            if b == b'/' {
                // Name
                pos += 1;
                let start = pos;
                while pos < stream.len() && !stream[pos].is_ascii_whitespace() && !b"/()<>[]{}/%".contains(&stream[pos]) {
                    pos += 1;
                }
                let name = String::from_utf8_lossy(&stream[start..pos]).to_string();
                stack.push(PdfObject::Name(name));
            } else if b == b'(' {
                // Literal string
                let mut temp_parser = PdfParser::new(&stream[pos..]);
                if let Ok(obj) = temp_parser.parse_literal_string() {
                    stack.push(obj);
                    pos += temp_parser.pos;
                } else {
                    pos += 1;
                }
            } else if b == b'<' && pos + 1 < stream.len() && stream[pos + 1] != b'<' {
                // Hex string
                let mut temp_parser = PdfParser::new(&stream[pos..]);
                if let Ok(obj) = temp_parser.parse_hex_string() {
                    stack.push(obj);
                    pos += temp_parser.pos;
                } else {
                    pos += 1;
                }
            } else if b == b'[' {
                // Array
                let mut temp_parser = PdfParser::new(&stream[pos..]);
                if let Ok(obj) = temp_parser.parse_array() {
                    stack.push(obj);
                    pos += temp_parser.pos;
                } else {
                    pos += 1;
                }
            } else if b == b'+' || b == b'-' || b == b'.' || b.is_ascii_digit() {
                // Number
                let start = pos;
                while pos < stream.len() && (stream[pos] == b'.' || stream[pos] == b'-' || stream[pos] == b'+' || stream[pos].is_ascii_digit()) {
                    pos += 1;
                }
                let s = std::str::from_utf8(&stream[start..pos]).unwrap_or("0");
                if s.contains('.') {
                    stack.push(PdfObject::Real(s.parse().unwrap_or(0.0)));
                } else {
                    stack.push(PdfObject::Integer(s.parse().unwrap_or(0)));
                }
            } else {
                // Operator keyword
                let start = pos;
                while pos < stream.len() && !stream[pos].is_ascii_whitespace() && !b"/()<>[]{}/%".contains(&stream[pos]) {
                    pos += 1;
                }
                let op = std::str::from_utf8(&stream[start..pos]).unwrap_or("");

                match op {
                    "BT" => {
                        text_x = 0.0;
                        text_y = 0.0;
                        line_x = 0.0;
                        line_y = 0.0;
                    }
                    "ET" => {}
                    "Tf" => {
                        if stack.len() >= 2 {
                            let size = stack.pop().and_then(|o| o.as_f64()).unwrap_or(12.0);
                            let font = stack.pop().and_then(|o| o.as_string_lossy()).unwrap_or("Helvetica".to_string());
                            current_font = font;
                            current_font_size = size;
                        }
                    }
                    "Tm" => {
                        if stack.len() >= 6 {
                            let f = stack.pop().and_then(|o| o.as_f64()).unwrap_or(0.0);
                            let e = stack.pop().and_then(|o| o.as_f64()).unwrap_or(0.0);
                            let _d = stack.pop();
                            let _c = stack.pop();
                            let _b = stack.pop();
                            let _a = stack.pop();
                            text_x = e;
                            text_y = f;
                            line_x = e;
                            line_y = f;
                        }
                    }
                    "Td" | "TD" => {
                        if stack.len() >= 2 {
                            let ty = stack.pop().and_then(|o| o.as_f64()).unwrap_or(0.0);
                            let tx = stack.pop().and_then(|o| o.as_f64()).unwrap_or(0.0);
                            line_x += tx;
                            line_y += ty;
                            text_x = line_x;
                            text_y = line_y;
                        }
                    }
                    "T*" => {
                        line_y -= current_font_size * 1.2;
                        text_x = line_x;
                        text_y = line_y;
                    }
                    "Tj" => {
                        if let Some(obj) = stack.pop() {
                            let text = obj.as_string_lossy().unwrap_or_default();
                            let char_w = current_font_size * 0.55;
                            let width = text.chars().count() as f64 * char_w;
                            let height = current_font_size;
                            // Convert PDF bottom-left origin to OpenWPS top-left origin
                            let screen_y = (page_height - text_y - height).max(0.0);

                            text_elements.push(PdfTextElement {
                                text: text.clone(),
                                bounds: Rect::new(text_x, screen_y, width, height),
                                font_name: current_font.clone(),
                                font_size: current_font_size,
                                color: current_fill_color,
                                baseline: screen_y + height * 0.8,
                            });
                            text_x += width;
                        }
                    }
                    "TJ" => {
                        if let Some(PdfObject::Array(items)) = stack.pop() {
                            let mut full_text = String::new();
                            for item in items {
                                match item {
                                    PdfObject::String(bytes) => {
                                        full_text.push_str(&String::from_utf8_lossy(&bytes));
                                    }
                                    PdfObject::Integer(adj) => {
                                        if adj < -150 {
                                            full_text.push(' ');
                                        }
                                    }
                                    PdfObject::Real(adj) => {
                                        if adj < -150.0 {
                                            full_text.push(' ');
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            let char_w = current_font_size * 0.55;
                            let width = full_text.chars().count() as f64 * char_w;
                            let height = current_font_size;
                            let screen_y = (page_height - text_y - height).max(0.0);

                            text_elements.push(PdfTextElement {
                                text: full_text,
                                bounds: Rect::new(text_x, screen_y, width, height),
                                font_name: current_font.clone(),
                                font_size: current_font_size,
                                color: current_fill_color,
                                baseline: screen_y + height * 0.8,
                            });
                            text_x += width;
                        }
                    }
                    "rg" => {
                        if stack.len() >= 3 {
                            let b = (stack.pop().and_then(|o| o.as_f64()).unwrap_or(0.0) * 255.0) as u8;
                            let g = (stack.pop().and_then(|o| o.as_f64()).unwrap_or(0.0) * 255.0) as u8;
                            let r = (stack.pop().and_then(|o| o.as_f64()).unwrap_or(0.0) * 255.0) as u8;
                            current_fill_color = Color::rgb(r, g, b);
                        }
                    }
                    "RG" => {
                        if stack.len() >= 3 {
                            let b = (stack.pop().and_then(|o| o.as_f64()).unwrap_or(0.0) * 255.0) as u8;
                            let g = (stack.pop().and_then(|o| o.as_f64()).unwrap_or(0.0) * 255.0) as u8;
                            let r = (stack.pop().and_then(|o| o.as_f64()).unwrap_or(0.0) * 255.0) as u8;
                            current_stroke_color = Color::rgb(r, g, b);
                        }
                    }
                    "g" => {
                        if let Some(val) = stack.pop().and_then(|o| o.as_f64()) {
                            let gray = (val * 255.0) as u8;
                            current_fill_color = Color::rgb(gray, gray, gray);
                        }
                    }
                    "G" => {
                        if let Some(val) = stack.pop().and_then(|o| o.as_f64()) {
                            let gray = (val * 255.0) as u8;
                            current_stroke_color = Color::rgb(gray, gray, gray);
                        }
                    }
                    "w" => {
                        if let Some(val) = stack.pop().and_then(|o| o.as_f64()) {
                            current_line_width = val;
                        }
                    }
                    "m" => {
                        if stack.len() >= 2 {
                            let y = stack.pop().and_then(|o| o.as_f64()).unwrap_or(0.0);
                            let x = stack.pop().and_then(|o| o.as_f64()).unwrap_or(0.0);
                            let sy = (page_height - y).max(0.0);
                            current_path_ops.push(PathOp::MoveTo(Point::new(x, sy)));
                        }
                    }
                    "l" => {
                        if stack.len() >= 2 {
                            let y = stack.pop().and_then(|o| o.as_f64()).unwrap_or(0.0);
                            let x = stack.pop().and_then(|o| o.as_f64()).unwrap_or(0.0);
                            let sy = (page_height - y).max(0.0);
                            current_path_ops.push(PathOp::LineTo(Point::new(x, sy)));
                        }
                    }
                    "re" => {
                        if stack.len() >= 4 {
                            let h = stack.pop().and_then(|o| o.as_f64()).unwrap_or(0.0);
                            let w = stack.pop().and_then(|o| o.as_f64()).unwrap_or(0.0);
                            let y = stack.pop().and_then(|o| o.as_f64()).unwrap_or(0.0);
                            let x = stack.pop().and_then(|o| o.as_f64()).unwrap_or(0.0);
                            let sy = (page_height - y - h).max(0.0);
                            current_path_ops.push(PathOp::Rectangle(Rect::new(x, sy, w.abs(), h.abs())));
                        }
                    }
                    "h" => {
                        current_path_ops.push(PathOp::Close);
                    }
                    "S" | "s" => {
                        if !current_path_ops.is_empty() {
                            let bounds = Self::calculate_path_bounds(&current_path_ops);
                            vector_paths.push(PdfVectorPath {
                                operations: std::mem::take(&mut current_path_ops),
                                bounds,
                                stroke_color: Some(current_stroke_color),
                                fill_color: None,
                                line_width: current_line_width,
                            });
                        }
                    }
                    "f" | "F" | "f*" => {
                        if !current_path_ops.is_empty() {
                            let bounds = Self::calculate_path_bounds(&current_path_ops);
                            vector_paths.push(PdfVectorPath {
                                operations: std::mem::take(&mut current_path_ops),
                                bounds,
                                stroke_color: None,
                                fill_color: Some(current_fill_color),
                                line_width: 0.0,
                            });
                        }
                    }
                    "B" | "b" | "B*" | "b*" => {
                        if !current_path_ops.is_empty() {
                            let bounds = Self::calculate_path_bounds(&current_path_ops);
                            vector_paths.push(PdfVectorPath {
                                operations: std::mem::take(&mut current_path_ops),
                                bounds,
                                stroke_color: Some(current_stroke_color),
                                fill_color: Some(current_fill_color),
                                line_width: current_line_width,
                            });
                        }
                    }
                    _ => {
                        stack.clear();
                    }
                }
            }
        }

        (text_elements, vector_paths)
    }

    fn calculate_path_bounds(ops: &[PathOp]) -> Rect {
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for op in ops {
            match op {
                PathOp::MoveTo(p) | PathOp::LineTo(p) => {
                    min_x = min_x.min(p.x);
                    min_y = min_y.min(p.y);
                    max_x = max_x.max(p.x);
                    max_y = max_y.max(p.y);
                }
                PathOp::CurveTo(p1, p2, p3) => {
                    min_x = min_x.min(p1.x).min(p2.x).min(p3.x);
                    min_y = min_y.min(p1.y).min(p2.y).min(p3.y);
                    max_x = max_x.max(p1.x).max(p2.x).max(p3.x);
                    max_y = max_y.max(p1.y).max(p2.y).max(p3.y);
                }
                PathOp::Rectangle(r) => {
                    min_x = min_x.min(r.x);
                    min_y = min_y.min(r.y);
                    max_x = max_x.max(r.x + r.width);
                    max_y = max_y.max(r.y + r.height);
                }
                PathOp::Close => {}
            }
        }

        if min_x <= max_x && min_y <= max_y {
            Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
        } else {
            Rect::new(0.0, 0.0, 0.0, 0.0)
        }
    }
}
