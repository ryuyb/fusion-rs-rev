//! File compression support for the advanced logger

use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Handles compression of rotated log files
pub struct CompressionHandler {
    enabled: bool,
}

impl CompressionHandler {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Compress a file using gzip compression
    pub fn compress_file(&self, file_path: &Path) -> anyhow::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Read the original file
        let input = fs::read(file_path)?;

        // Create compressed file path
        let compressed_path = file_path.with_extension(
            format!(
                "{}.gz",
                file_path.extension().unwrap_or_default().to_string_lossy()
            )
            .trim_start_matches('.'),
        );

        // Compress and write
        let output_file = File::create(&compressed_path)?;
        let mut encoder = GzEncoder::new(output_file, Compression::default());
        encoder.write_all(&input)?;
        encoder.finish()?;

        // Remove original file
        fs::remove_file(file_path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_compression_disabled() {
        let handler = CompressionHandler::new(false);
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.log");

        fs::write(&file_path, "test content").unwrap();

        assert!(handler.compress_file(&file_path).is_ok());
        assert!(file_path.exists());
    }

    #[test]
    fn test_compression_enabled() {
        let handler = CompressionHandler::new(true);
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.log");

        fs::write(&file_path, "test content for compression").unwrap();

        assert!(handler.compress_file(&file_path).is_ok());
        assert!(!file_path.exists());

        let compressed_path = dir.path().join("test.log.gz");
        assert!(compressed_path.exists());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use flate2::read::GzDecoder;
    use proptest::prelude::*;
    use std::fs;
    use std::io::Read;
    use tempfile::tempdir;

    proptest! {
        /// Property 14: 文件压缩控制
        /// *For any* file content and compression enabled flag, when compression is disabled,
        /// the original file should remain unchanged; when enabled, the original file should
        /// be deleted and a compressed file should be created.
        /// **Validates: Requirements 8.1, 8.2, 8.3, 8.4**
        #[test]
        fn property_compression_control(
            enabled in any::<bool>(),
            content in "[a-zA-Z0-9 \n]{1,1000}"
        ) {
            let dir = tempdir().unwrap();
            let file_path = dir.path().join("test.log");
            let compressed_path = dir.path().join("test.log.gz");

            fs::write(&file_path, &content).unwrap();

            let handler = CompressionHandler::new(enabled);
            let result = handler.compress_file(&file_path);
            prop_assert!(result.is_ok());

            if enabled {
                prop_assert!(!file_path.exists(), "Original file should be deleted when compression is enabled");
                prop_assert!(compressed_path.exists(), "Compressed file should exist when compression is enabled");
            } else {
                prop_assert!(file_path.exists(), "Original file should remain when compression is disabled");
                prop_assert!(!compressed_path.exists(), "No compressed file should be created when compression is disabled");

                let read_content = fs::read_to_string(&file_path).unwrap();
                prop_assert_eq!(read_content, content);
            }
        }

        /// Property 15: 压缩格式支持 (Round-trip property)
        /// *For any* file content, compressing and then decompressing should produce
        /// the original content (gzip format support).
        /// **Validates: Requirements 8.5**
        #[test]
        fn property_compression_format_roundtrip(
            content in "[a-zA-Z0-9 \n]{1,1000}"
        ) {
            let dir = tempdir().unwrap();
            let file_path = dir.path().join("test.log");
            let compressed_path = dir.path().join("test.log.gz");

            fs::write(&file_path, &content).unwrap();

            let handler = CompressionHandler::new(true);
            let result = handler.compress_file(&file_path);
            prop_assert!(result.is_ok());

            prop_assert!(compressed_path.exists(), "Compressed file should exist");

            let compressed_data = fs::read(&compressed_path).unwrap();
            let mut decoder = GzDecoder::new(&compressed_data[..]);
            let mut decompressed_content = String::new();
            let decode_result = decoder.read_to_string(&mut decompressed_content);
            prop_assert!(decode_result.is_ok(), "Decompression should succeed");

            prop_assert_eq!(
                decompressed_content, content,
                "Decompressed content should match original content"
            );
        }
    }
}
