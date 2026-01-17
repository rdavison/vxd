//! Fileformat handling (unix/dos/mac line endings).
//!
//! This module provides helpers for detecting and converting line endings.

/// Supported file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    /// Unix LF line endings.
    Unix,
    /// DOS/Windows CRLF line endings.
    Dos,
    /// Classic Mac CR line endings.
    Mac,
}

/// Detect file format based on line endings.
pub fn detect_fileformat(text: &str) -> FileFormat {
    if text.contains("\r\n") {
        FileFormat::Dos
    } else if text.contains('\r') {
        FileFormat::Mac
    } else {
        FileFormat::Unix
    }
}

/// Convert line endings to the requested file format.
pub fn convert_line_endings(text: &str, format: FileFormat) -> String {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    match format {
        FileFormat::Unix => normalized,
        FileFormat::Dos => normalized.replace('\n', "\r\n"),
        FileFormat::Mac => normalized.replace('\n', "\r"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_fileformat_unix() {
        let text = "one\ntwo\n";
        assert_eq!(detect_fileformat(text), FileFormat::Unix);
    }

    #[test]
    fn test_detect_fileformat_dos() {
        let text = "one\r\ntwo\r\n";
        assert_eq!(detect_fileformat(text), FileFormat::Dos);
    }

    #[test]
    fn test_detect_fileformat_mac() {
        let text = "one\rtwo\r";
        assert_eq!(detect_fileformat(text), FileFormat::Mac);
    }

    #[test]
    fn test_convert_line_endings() {
        let text = "one\r\ntwo\rthree\n";
        assert_eq!(convert_line_endings(text, FileFormat::Unix), "one\ntwo\nthree\n");
        assert_eq!(convert_line_endings(text, FileFormat::Dos), "one\r\ntwo\r\nthree\r\n");
        assert_eq!(convert_line_endings(text, FileFormat::Mac), "one\rtwo\rthree\r");
    }
}
