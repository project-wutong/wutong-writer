// SPDX-License-Identifier: AGPL-3.0-only
//! Wutong Writer engine.
//! Document session: exact-bytes model of a native source file (FILES-001/002).

use wutong_core::wire::sha256_digest;

/// Detected line ending style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    /// No line breaks.
    None,
    /// `\n`
    Lf,
    /// `\r\n`
    CrLf,
    /// Both styles present (preserved as-is).
    Mixed,
}

/// An open document. The original bytes are kept verbatim; saving an unmodified session is a no-op.
#[derive(Debug, Clone)]
pub struct DocumentSession {
    original: Vec<u8>,
    current: Vec<u8>,
}

impl DocumentSession {
    /// Open from raw bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            current: bytes.clone(),
            original: bytes,
        }
    }
    /// `sha256:` digest of the current bytes.
    pub fn digest(&self) -> String {
        sha256_digest(&self.current).to_string()
    }
    /// Has the content changed since open/save?
    pub fn is_dirty(&self) -> bool {
        self.original != self.current
    }
    /// Starts with a UTF-8 BOM?
    pub fn has_bom(&self) -> bool {
        self.current.starts_with(&[0xef, 0xbb, 0xbf])
    }
    /// Line ending style.
    pub fn line_ending(&self) -> LineEnding {
        let crlf = self.current.windows(2).filter(|w| w == b"\r\n").count();
        let lf = self.current.iter().filter(|&&b| b == b'\n').count();
        match (crlf, lf - crlf) {
            (0, 0) => LineEnding::None,
            (0, _) => LineEnding::Lf,
            (_, 0) => LineEnding::CrLf,
            _ => LineEnding::Mixed,
        }
    }
    /// Replace the byte range `[start, end)` with `text` (minimal source edit).
    pub fn replace_range(&mut self, start: usize, end: usize, text: &str) -> Result<(), String> {
        if start > end || end > self.current.len() {
            return Err("range out of bounds".into());
        }
        self.current.splice(start..end, text.bytes());
        Ok(())
    }
    /// Bytes to write, or `None` when nothing changed (never rewrite untouched files).
    pub fn bytes_to_save(&self) -> Option<&[u8]> {
        self.is_dirty().then_some(&self.current[..])
    }
    /// Mark saved.
    pub fn mark_saved(&mut self) {
        self.original = self.current.clone();
    }
    /// Did the file change on disk behind our back? (`ERR_CONFLICT` territory)
    pub fn external_change(&self, on_disk: &[u8]) -> bool {
        on_disk != self.original
    }
}

/// One-line description used by the GUI status bar.
pub fn describe(bytes: Vec<u8>) -> String {
    let s = DocumentSession::from_bytes(bytes);
    format!(
        "{} bytes · {} · line endings {:?}{}",
        s.current.len(),
        s.digest(),
        s.line_ending(),
        if s.has_bom() { " · BOM" } else { "" }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unmodified_save_is_noop_and_bytes_are_preserved() {
        let raw = b"\xef\xbb\xbf# Title\r\n\r\n% keep\r\nbody\n".to_vec();
        let mut s = DocumentSession::from_bytes(raw.clone());
        assert!(s.bytes_to_save().is_none());
        assert!(s.has_bom());
        assert_eq!(s.line_ending(), LineEnding::Mixed);
        s.replace_range(5, 10, "标题").unwrap();
        let out = s.bytes_to_save().unwrap().to_vec();
        assert_eq!(&out[..5], &raw[..5]);
        assert_eq!(&out[out.len() - 15..], &raw[raw.len() - 15..]);
        s.mark_saved();
        assert!(!s.is_dirty());
        assert!(s.external_change(&raw));
    }
}
