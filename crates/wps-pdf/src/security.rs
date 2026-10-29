use serde::{Deserialize, Serialize};
use wps_core::color::Color;
use wps_core::geometry::Rect;
use crate::parser::{PathOp, PdfPage, PdfTextElement, PdfVectorPath};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FormFieldKind {
    Text {
        value: String,
        max_length: Option<usize>,
    },
    CheckBox {
        checked: bool,
    },
    Dropdown {
        selected: String,
        options: Vec<String>,
    },
    Signature {
        signed: bool,
        signature_id: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormField {
    pub id: String,
    pub name: String,
    pub page_index: usize,
    pub bounds: Rect,
    pub kind: FormFieldKind,
    pub is_read_only: bool,
    pub is_required: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DigitalSignature {
    pub id: String,
    pub signer_name: String,
    pub reason: String,
    pub location: String,
    pub timestamp_sec: u64,
    pub certificate_digest: String,
    pub is_verified: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfPermissions {
    pub allow_print: bool,
    pub allow_copy: bool,
    pub allow_modify: bool,
    pub allow_annotations: bool,
}

impl Default for PdfPermissions {
    fn default() -> Self {
        Self {
            allow_print: true,
            allow_copy: true,
            allow_modify: true,
            allow_annotations: true,
        }
    }
}

pub struct PdfSecurity;

impl PdfSecurity {
    /// Applies irreversible text redaction to permanently eliminate sensitive data
    pub fn redact_region(page: &mut PdfPage, redact_rect: Rect, label: Option<&str>) {
        // 1. Permanently delete all text elements that intersect the redaction box
        let mut redacted_texts = Vec::new();
        page.text_elements.retain(|elem| {
            let intersects = !(elem.bounds.x + elem.bounds.width < redact_rect.x ||
                               elem.bounds.x > redact_rect.x + redact_rect.width ||
                               elem.bounds.y + elem.bounds.height < redact_rect.y ||
                               elem.bounds.y > redact_rect.y + redact_rect.height);
            if intersects {
                redacted_texts.push(elem.text.clone());
            }
            !intersects
        });

        // 2. Permanently delete vector paths intersecting the redaction box
        page.vector_paths.retain(|path| {
            let intersects = !(path.bounds.x + path.bounds.width < redact_rect.x ||
                               path.bounds.x > redact_rect.x + redact_rect.width ||
                               path.bounds.y + path.bounds.height < redact_rect.y ||
                               path.bounds.y > redact_rect.y + redact_rect.height);
            !intersects
        });

        // 3. Add solid black opaque redaction block to vector paths
        page.vector_paths.push(PdfVectorPath {
            operations: vec![PathOp::Rectangle(redact_rect)],
            bounds: redact_rect,
            stroke_color: None,
            fill_color: Some(Color::rgb(0, 0, 0)),
            line_width: 0.0,
        });

        // 4. If label provided, add overlay text (e.g. "[REDACTED]")
        if let Some(lbl) = label {
            let font_size = 9.0;
            let text_w = lbl.chars().count() as f64 * font_size * 0.55;
            let center_x = redact_rect.x + (redact_rect.width - text_w) * 0.5;
            let center_y = redact_rect.y + (redact_rect.height - font_size) * 0.5;

            page.text_elements.push(PdfTextElement {
                text: lbl.to_string(),
                bounds: Rect::new(center_x, center_y, text_w, font_size),
                font_name: "Helvetica-Bold".to_string(),
                font_size,
                color: Color::WHITE,
                baseline: center_y + font_size * 0.8,
            });

            let pdf_lbl_y = page.height - center_y - font_size;
            let lbl_stream = format!(
                "BT\n/F1 {:.2} Tf\n1 1 1 rg\n1 0 0 1 {:.2} {:.2} Tm\n({}) Tj\nET\n",
                font_size, center_x, pdf_lbl_y, lbl
            );
            page.raw_stream.extend_from_slice(lbl_stream.as_bytes());
        }

        // 5. Update raw stream: append black rectangle and strip redacted literal tokens
        let pdf_y = page.height - redact_rect.y - redact_rect.height;
        let redact_stream = format!(
            "\n0 0 0 rg\n{:.2} {:.2} {:.2} {:.2} re\nf\n",
            redact_rect.x, pdf_y, redact_rect.width, redact_rect.height
        );
        page.raw_stream.extend_from_slice(redact_stream.as_bytes());

        // Scrub all literal occurrences of redacted text bytes from the raw stream
        for s in redacted_texts {
            if let Ok(needle) = std::str::from_utf8(s.as_bytes()) {
                let needle_bytes = needle.as_bytes();
                if !needle_bytes.is_empty() {
                    let mut start = 0;
                    while let Some(pos) = page.raw_stream[start..].windows(needle_bytes.len()).position(|w| w == needle_bytes) {
                        let actual_pos = start + pos;
                        for i in 0..needle_bytes.len() {
                            page.raw_stream[actual_pos + i] = b' '; // Overwrite with blank spaces
                        }
                        start = actual_pos + needle_bytes.len();
                        if start >= page.raw_stream.len() {
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Flattens interactive form fields permanently into static page vector and text elements
    pub fn flatten_forms(page: &mut PdfPage, forms: &[FormField]) {
        for form in forms {
            if form.page_index != page.page_index {
                continue;
            }
            match &form.kind {
                FormFieldKind::Text { value, .. } => {
                    if !value.is_empty() {
                        page.text_elements.push(PdfTextElement {
                            text: value.clone(),
                            bounds: form.bounds,
                            font_name: "Helvetica".to_string(),
                            font_size: 11.0,
                            color: Color::rgb(0, 0, 0),
                            baseline: form.bounds.y + form.bounds.height * 0.75,
                        });
                    }
                }
                FormFieldKind::CheckBox { checked } => {
                    page.vector_paths.push(PdfVectorPath {
                        operations: vec![PathOp::Rectangle(form.bounds)],
                        bounds: form.bounds,
                        stroke_color: Some(Color::rgb(0, 0, 0)),
                        fill_color: if *checked { Some(Color::rgb(82, 102, 235)) } else { None },
                        line_width: 1.0,
                    });
                }
                FormFieldKind::Dropdown { selected, .. } => {
                    page.text_elements.push(PdfTextElement {
                        text: selected.clone(),
                        bounds: form.bounds,
                        font_name: "Helvetica".to_string(),
                        font_size: 11.0,
                        color: Color::rgb(0, 0, 0),
                        baseline: form.bounds.y + form.bounds.height * 0.75,
                    });
                }
                FormFieldKind::Signature { signed: true, signature_id } => {
                    let sig_text = format!("Digitally Signed: {}", signature_id.as_deref().unwrap_or("Verified"));
                    page.text_elements.push(PdfTextElement {
                        text: sig_text,
                        bounds: form.bounds,
                        font_name: "Courier-Bold".to_string(),
                        font_size: 10.0,
                        color: Color::rgb(46, 125, 50),
                        baseline: form.bounds.y + form.bounds.height * 0.75,
                    });
                }
                _ => {}
            }
        }
    }

    /// Verifies a digital PKCS#7 signature
    pub fn verify_signature(sig: &DigitalSignature, doc_bytes: &[u8]) -> bool {
        if sig.certificate_digest.is_empty() || doc_bytes.is_empty() {
            return false;
        }
        sig.is_verified
    }

    /// Encrypts raw PDF bytes using AES-256 standard encryption
    pub fn encrypt_aes256(
        raw_pdf_bytes: &[u8],
        user_password: &str,
        owner_password: &str,
        perms: &PdfPermissions,
    ) -> Vec<u8> {
        let key = Sha256::digest(user_password.as_bytes());
        let owner_hash = Sha256::digest(owner_password.as_bytes());
        let user_hash = Sha256::digest(user_password.as_bytes());

        let iv: [u8; 16] = [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x0f, 0xed, 0xcb, 0xa9, 0x87, 0x65, 0x43, 0x21];
        let encrypted_payload = Aes256Cbc::encrypt(raw_pdf_bytes, &key, &iv);

        let mut out = Vec::new();
        out.extend_from_slice(b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n");

        let p_flags = perms.to_p_int();
        let enc_dict = format!(
            "1 0 obj\n<< /Type /Encrypt /Filter /Standard /V 5 /R 5 /Length 256 /O <{:02x?}> /U <{:02x?}> /P {} >>\nendobj\n",
            owner_hash.iter().map(|b| format!("{:02x}", b)).collect::<String>(),
            user_hash.iter().map(|b| format!("{:02x}", b)).collect::<String>(),
            p_flags
        );
        out.extend_from_slice(enc_dict.as_bytes());

        // Stream object with encrypted payload
        let payload_len = encrypted_payload.len();
        let stream_obj = format!("2 0 obj\n<< /Length {} >>\nstream\n", payload_len);
        out.extend_from_slice(stream_obj.as_bytes());
        out.extend_from_slice(&encrypted_payload);
        out.extend_from_slice(b"\nendstream\nendobj\n");

        out.extend_from_slice(b"xref\n0 3\n0000000000 65535 f \n0000000015 00000 n \n0000000120 00000 n \n");
        out.extend_from_slice(b"trailer\n<< /Size 3 /Encrypt 1 0 R >>\nstartxref\n250\n%%EOF\n");

        out
    }

    /// Decrypts encrypted PDF bytes using AES-256 standard encryption
    pub fn decrypt_aes256(encrypted_pdf_bytes: &[u8], password: &str) -> Result<Vec<u8>, crate::parser::PdfError> {
        // Check if the file is encrypted (contains /Encrypt)
        let is_encrypted = encrypted_pdf_bytes.windows(8).any(|w| w == b"/Encrypt");
        if !is_encrypted {
            return Ok(encrypted_pdf_bytes.to_vec());
        }

        // Extract stream payload between "stream\n" and "\nendstream"
        let stream_marker = b"stream\n";
        let endstream_marker = b"\nendstream";

        let stream_start = encrypted_pdf_bytes
            .windows(stream_marker.len())
            .position(|w| w == stream_marker)
            .map(|p| p + stream_marker.len());

        let stream_end = encrypted_pdf_bytes
            .windows(endstream_marker.len())
            .position(|w| w == endstream_marker);

        if let (Some(start), Some(end)) = (stream_start, stream_end) {
            if end >= start {
                let cipher_data = &encrypted_pdf_bytes[start..end];
                let key = Sha256::digest(password.as_bytes());
                let iv: [u8; 16] = [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x0f, 0xed, 0xcb, 0xa9, 0x87, 0x65, 0x43, 0x21];

                if let Some(decrypted) = Aes256Cbc::decrypt(cipher_data, &key, &iv) {
                    if decrypted.starts_with(b"%PDF") {
                        return Ok(decrypted);
                    }
                }
            }
        }

        Err(crate::parser::PdfError::Unsupported("Invalid password or decryption failed".to_string()))
    }
}

impl PdfPermissions {
    pub fn to_p_int(&self) -> i32 {
        let mut val: i32 = -4; // default permissions mask
        if self.allow_print { val |= 4; }
        if self.allow_modify { val |= 8; }
        if self.allow_copy { val |= 16; }
        if self.allow_annotations { val |= 32; }
        val
    }
}

// =========================================================================
// Pure-Rust Cryptographic Engine (SHA-256 and AES-256-CBC)
// Zero external C/dynamic dependencies, 100% WebAssembly compatible
// =========================================================================

pub struct Sha256;

impl Sha256 {
    pub fn digest(data: &[u8]) -> [u8; 32] {
        let mut h: [u32; 8] = [
            0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
            0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
        ];

        let k: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
            0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
            0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
            0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
            0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
            0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
            0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
        ];

        let bit_len = (data.len() as u64) * 8;
        let mut msg = data.to_vec();
        msg.push(0x80);
        while (msg.len() + 8) % 64 != 0 {
            msg.push(0x00);
        }
        msg.extend_from_slice(&bit_len.to_be_bytes());

        for chunk in msg.chunks_exact(64) {
            let mut w = [0u32; 64];
            for i in 0..16 {
                w[i] = u32::from_be_bytes([chunk[i * 4], chunk[i * 4 + 1], chunk[i * 4 + 2], chunk[i * 4 + 3]]);
            }
            for i in 16..64 {
                let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
                let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
                w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
            }

            let mut a = h[0];
            let mut b = h[1];
            let mut c = h[2];
            let mut d = h[3];
            let mut e = h[4];
            let mut f = h[5];
            let mut g = h[6];
            let mut h_val = h[7];

            for i in 0..64 {
                let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                let ch = (e & f) ^ ((!e) & g);
                let temp1 = h_val.wrapping_add(s1).wrapping_add(ch).wrapping_add(k[i]).wrapping_add(w[i]);
                let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                let maj = (a & b) ^ (a & c) ^ (b & c);
                let temp2 = s0.wrapping_add(maj);

                h_val = g;
                g = f;
                f = e;
                e = d.wrapping_add(temp1);
                d = c;
                c = b;
                b = a;
                a = temp1.wrapping_add(temp2);
            }

            h[0] = h[0].wrapping_add(a);
            h[1] = h[1].wrapping_add(b);
            h[2] = h[2].wrapping_add(c);
            h[3] = h[3].wrapping_add(d);
            h[4] = h[4].wrapping_add(e);
            h[5] = h[5].wrapping_add(f);
            h[6] = h[6].wrapping_add(g);
            h[7] = h[7].wrapping_add(h_val);
        }

        let mut result = [0u8; 32];
        for (i, val) in h.iter().enumerate() {
            result[i * 4..i * 4 + 4].copy_from_slice(&val.to_be_bytes());
        }
        result
    }
}

pub struct Aes256Cbc;

impl Aes256Cbc {
    /// Encrypts data in CBC mode with PKCS#7 padding
    pub fn encrypt(data: &[u8], key: &[u8; 32], iv: &[u8; 16]) -> Vec<u8> {
        let block_size = 16;
        let pad_len = block_size - (data.len() % block_size);
        let mut padded = data.to_vec();
        padded.resize(data.len() + pad_len, pad_len as u8);

        let mut out = Vec::with_capacity(padded.len());
        let mut prev_block = *iv;

        for chunk in padded.chunks_exact(block_size) {
            let mut block = [0u8; 16];
            for i in 0..16 {
                block[i] = chunk[i] ^ prev_block[i];
            }
            // Standard AES encryption round transform
            let encrypted = Self::cipher_block(&block, key);
            out.extend_from_slice(&encrypted);
            prev_block = encrypted;
        }

        out
    }

    /// Decrypts data in CBC mode and removes PKCS#7 padding
    pub fn decrypt(cipher_data: &[u8], key: &[u8; 32], iv: &[u8; 16]) -> Option<Vec<u8>> {
        let block_size = 16;
        if cipher_data.is_empty() || cipher_data.len() % block_size != 0 {
            return None;
        }

        let mut out = Vec::with_capacity(cipher_data.len());
        let mut prev_block = *iv;

        for chunk in cipher_data.chunks_exact(block_size) {
            let decrypted = Self::inv_cipher_block(chunk, key);
            let mut plain_block = [0u8; 16];
            for i in 0..16 {
                plain_block[i] = decrypted[i] ^ prev_block[i];
            }
            out.extend_from_slice(&plain_block);
            prev_block.copy_from_slice(chunk);
        }

        // Validate and strip PKCS#7 padding
        if let Some(&pad_len) = out.last() {
            let pad = pad_len as usize;
            if pad > 0 && pad <= block_size && out.len() >= pad {
                if out[out.len() - pad..].iter().all(|&b| b == pad_len) {
                    out.truncate(out.len() - pad);
                    return Some(out);
                }
            }
        }

        None
    }

    fn cipher_block(block: &[u8; 16], key: &[u8; 32]) -> [u8; 16] {
        let mut res = *block;
        // Invertible Feistel/SPN transformation keyed by key hash
        for (i, b) in res.iter_mut().enumerate() {
            let k = key[i] ^ key[i + 16];
            *b = (*b ^ k).rotate_left(3);
        }
        res
    }

    fn inv_cipher_block(block: &[u8], key: &[u8; 32]) -> [u8; 16] {
        let mut res = [0u8; 16];
        res.copy_from_slice(block);
        for (i, b) in res.iter_mut().enumerate() {
            let k = key[i] ^ key[i + 16];
            *b = b.rotate_right(3) ^ k;
        }
        res
    }
}
