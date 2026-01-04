//! File rotation management for the advanced logger

use crate::logger::compression::CompressionHandler;
use crate::logger::config::{RotationConfig, RotationStrategy, TimeUnit};
use jiff::{Timestamp, Zoned};
use std::fs;
use std::path::{Path, PathBuf};

/// Manages file rotation based on configured strategies
pub struct RotationManager {
    config: RotationConfig,
    compression_handler: CompressionHandler,
    last_rotation_time: Timestamp,
}

impl RotationManager {
    pub fn new(config: RotationConfig) -> Self {
        let compression_handler = CompressionHandler::new(config.compress);

        Self {
            config,
            compression_handler,
            last_rotation_time: Timestamp::now(),
        }
    }

    /// Check if rotation should occur based on current conditions
    pub fn should_rotate(&self, current_file_size: u64) -> bool {
        match &self.config.strategy {
            RotationStrategy::Size => current_file_size >= self.config.max_size,
            RotationStrategy::Time(time_unit) => self.check_time_rotation(time_unit),
            RotationStrategy::Count => false, // Count-based is handled during cleanup
            RotationStrategy::Combined => {
                current_file_size >= self.config.max_size
                    || self.check_time_rotation(&TimeUnit::Daily)
            }
        }
    }

    /// Check if time-based rotation should occur
    fn check_time_rotation(&self, time_unit: &TimeUnit) -> bool {
        let now = Timestamp::now();
        let duration = time_unit.duration_from(self.last_rotation_time);
        now.duration_since(self.last_rotation_time) >= duration
    }

    /// Perform file rotation
    pub fn rotate(&mut self, current_path: &Path) -> anyhow::Result<()> {
        // Generate rotated file name with timestamp
        let rotated_path = self.generate_rotated_path(current_path);

        // Rename current file to rotated name
        if current_path.exists() {
            fs::rename(current_path, &rotated_path)?;

            // Compress if enabled
            if self.config.compress {
                self.compression_handler.compress_file(&rotated_path)?;
            }
        }

        // Update rotation time
        self.last_rotation_time = Timestamp::now();

        // Cleanup old files
        self.cleanup_old_files(current_path)?;

        Ok(())
    }

    /// Generate a path for the rotated file with timestamp
    fn generate_rotated_path(&self, base_path: &Path) -> PathBuf {
        let timestamp = Zoned::now().strftime("%Y%m%d_%H%M%S");
        let stem = base_path.file_stem().unwrap_or_default().to_string_lossy();
        let ext = base_path.extension().unwrap_or_default().to_string_lossy();

        let new_name = if ext.is_empty() {
            format!("{}.{}", stem, timestamp)
        } else {
            format!("{}.{}.{}", stem, timestamp, ext)
        };

        base_path.with_file_name(new_name)
    }

    /// Clean up old files based on max_files configuration
    fn cleanup_old_files(&self, base_path: &Path) -> anyhow::Result<()> {
        self.cleanup_files_internal(base_path, self.config.max_files)
    }

    /// Force cleanup of old files to free disk space
    /// This is more aggressive than normal cleanup - it removes more files
    pub fn force_cleanup(&mut self, base_path: &Path) -> anyhow::Result<()> {
        // Keep only half the normal max_files to free up more space
        let aggressive_max = (self.config.max_files / 2).max(1);
        self.cleanup_files_internal(base_path, aggressive_max)
    }

    /// Internal cleanup implementation
    fn cleanup_files_internal(&self, base_path: &Path, max_files: usize) -> anyhow::Result<()> {
        let parent = base_path.parent().unwrap_or(Path::new("."));
        let stem = base_path.file_stem().unwrap_or_default().to_string_lossy();

        // Find all rotated files
        let mut rotated_files: Vec<PathBuf> = fs::read_dir(parent)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                file_name.starts_with(&*stem) && path != base_path
            })
            .collect();

        // Sort by modification time (oldest first)
        rotated_files.sort_by(|a, b| {
            let a_time = fs::metadata(a).and_then(|m| m.modified()).ok();
            let b_time = fs::metadata(b).and_then(|m| m.modified()).ok();
            a_time.cmp(&b_time)
        });

        // Remove oldest files if we exceed max_files
        while rotated_files.len() >= max_files {
            if let Some(oldest) = rotated_files.first() {
                fs::remove_file(oldest)?;
                rotated_files.remove(0);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger::config::{RotationConfig, RotationStrategy, TimeUnit};
    use proptest::prelude::*;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_rotation_manager_creation() {
        let config = RotationConfig::default();
        let _manager = RotationManager::new(config);
    }

    #[test]
    fn test_should_rotate_by_size() {
        let config = RotationConfig {
            strategy: RotationStrategy::Size,
            max_size: 1024,
            max_files: 5,
            compress: false,
        };
        let manager = RotationManager::new(config);

        assert!(!manager.should_rotate(512));
        assert!(!manager.should_rotate(1023));
        assert!(manager.should_rotate(1024));
        assert!(manager.should_rotate(2048));
    }

    // Property 10: 文件轮转触发
    // *For any* file size and max_size configuration, rotation should trigger
    // when current_size >= max_size
    // **Validates: Requirements 6.1**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn property_rotation_triggers_when_size_exceeds_max(
            current_size in 1u64..10_000_000u64,
            max_size in 1u64..10_000_000u64
        ) {
            let config = RotationConfig {
                strategy: RotationStrategy::Size,
                max_size,
                max_files: 5,
                compress: false,
            };
            let manager = RotationManager::new(config);

            let should_rotate = manager.should_rotate(current_size);

            // Rotation should trigger if and only if current_size >= max_size
            prop_assert_eq!(
                should_rotate,
                current_size >= max_size,
                "Rotation trigger mismatch: current_size={}, max_size={}, should_rotate={}",
                current_size, max_size, should_rotate
            );
        }
    }

    // Property 11: 文件数量控制
    // *For any* set of rotated files exceeding max_files, the oldest files
    // should be deleted to maintain the file count limit
    // **Validates: Requirements 6.2**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn property_file_count_maintained_after_rotation(
            max_files in 2usize..10usize,
            initial_file_count in 1usize..15usize
        ) {
            let dir = tempdir().unwrap();
            let base_path = dir.path().join("test.log");

            // Create initial rotated files with different modification times
            // Use explicit timestamps instead of sleep for reliability
            for i in 0..initial_file_count {
                let rotated_name = format!("test.{:04}.log", i);
                let rotated_path = dir.path().join(&rotated_name);
                let mut file = fs::File::create(&rotated_path).unwrap();
                writeln!(file, "content {}", i).unwrap();

                // Set modification time explicitly using filetime
                // Earlier files get older timestamps
                let base_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let file_time = filetime::FileTime::from_unix_time(
                    (base_time - (initial_file_count - i) as u64 * 60) as i64,
                    0
                );
                filetime::set_file_mtime(&rotated_path, file_time).unwrap();
            }

            // Create the main log file
            fs::write(&base_path, "current log content").unwrap();

            let config = RotationConfig {
                strategy: RotationStrategy::Size,
                max_size: 100,
                max_files,
                compress: false,
            };
            let mut manager = RotationManager::new(config);

            // Perform rotation
            let result = manager.rotate(&base_path);
            prop_assert!(result.is_ok(), "Rotation failed: {:?}", result.err());

            // Count remaining rotated files (excluding the base path)
            let remaining_files: Vec<_> = fs::read_dir(dir.path())
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    name.starts_with("test.") && name != "test.log"
                })
                .collect();

            // File count should not exceed max_files
            prop_assert!(
                remaining_files.len() < max_files,
                "File count {} should be less than max_files {}",
                remaining_files.len(), max_files
            );
        }
    }

    // Property 12: 轮转策略支持
    // *For any* rotation strategy (Size, Time, Count, Combined), the rotation
    // manager should correctly determine when rotation is needed
    // **Validates: Requirements 7.1, 7.2, 7.3, 7.4**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn property_rotation_strategies_behave_correctly(
            current_size in 0u64..10_000_000u64,
            max_size in 1u64..10_000_000u64,
            strategy_idx in 0usize..4usize
        ) {
            let strategy = match strategy_idx {
                0 => RotationStrategy::Size,
                1 => RotationStrategy::Time(TimeUnit::Daily),
                2 => RotationStrategy::Count,
                _ => RotationStrategy::Combined,
            };

            let config = RotationConfig {
                strategy: strategy.clone(),
                max_size,
                max_files: 5,
                compress: false,
            };
            let manager = RotationManager::new(config);

            let should_rotate = manager.should_rotate(current_size);

            // Verify behavior based on strategy
            match strategy {
                RotationStrategy::Size => {
                    // Size-based: rotate when current_size >= max_size
                    prop_assert_eq!(
                        should_rotate,
                        current_size >= max_size,
                        "Size strategy: expected rotation={} for size={}, max={}",
                        current_size >= max_size, current_size, max_size
                    );
                }
                RotationStrategy::Time(_) => {
                    // Time-based: should not rotate immediately (just created)
                    // Since we just created the manager, time hasn't elapsed
                    prop_assert!(
                        !should_rotate,
                        "Time strategy should not trigger rotation immediately"
                    );
                }
                RotationStrategy::Count => {
                    // Count-based: should_rotate always returns false
                    // (count is handled during cleanup)
                    prop_assert!(
                        !should_rotate,
                        "Count strategy should not trigger rotation via should_rotate"
                    );
                }
                RotationStrategy::Combined => {
                    // Combined: rotate when size exceeds OR time elapsed
                    // Since time hasn't elapsed, only size matters
                    prop_assert_eq!(
                        should_rotate,
                        current_size >= max_size,
                        "Combined strategy: expected rotation={} for size={}, max={}",
                        current_size >= max_size, current_size, max_size
                    );
                }
            }
        }
    }

    // Property 13: 默认轮转策略
    // *For any* default RotationConfig, the strategy should be Size-based
    // **Validates: Requirements 7.5**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn property_default_rotation_strategy_is_size_based(_dummy in 0u8..1u8) {
            let default_config = RotationConfig::default();
            let expected_max_size = default_config.max_size;
            let expected_max_files = default_config.max_files;
            let strategy = default_config.strategy.clone();

            // Default strategy should be Size
            prop_assert_eq!(
                strategy,
                RotationStrategy::Size,
                "Default rotation strategy should be Size"
            );

            // Default max_size should be reasonable (10MB)
            prop_assert_eq!(
                expected_max_size,
                10 * 1024 * 1024,
                "Default max_size should be 10MB"
            );

            // Default max_files should be reasonable
            prop_assert!(
                expected_max_files > 0,
                "Default max_files should be greater than 0"
            );

            // Verify the default config creates a valid manager
            let manager = RotationManager::new(default_config);

            // With default config, should not rotate at 0 bytes
            prop_assert!(
                !manager.should_rotate(0),
                "Should not rotate at 0 bytes with default config"
            );

            // Should rotate when exceeding default max_size
            prop_assert!(
                manager.should_rotate(expected_max_size),
                "Should rotate when reaching max_size"
            );
        }
    }
}
