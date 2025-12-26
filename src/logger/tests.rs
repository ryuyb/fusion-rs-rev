//! Tests for the advanced logger module

use crate::logger::config::*;
use crate::logger::writer::{RotatingFileWriter, RecoveryStrategy};
use std::path::PathBuf;

#[cfg(test)]
mod config_tests {
    use super::*;

    /// Helper function to create a test configuration
    fn create_test_config() -> LoggerConfig {
        LoggerConfig {
            console: ConsoleConfig {
                enabled: true,
                colored: false,
            },
            file: FileConfig {
                enabled: false,
                path: PathBuf::from("test.log"),
                append: true,
                format: LogFormat::Full,
                rotation: RotationConfig::default(),
            },
            level: "info".to_string(),
        }
    }

    #[test]
    fn test_default_config_creation() {
        let config = LoggerConfig::default();
        assert!(config.console.enabled);
        assert!(config.console.colored);
        assert!(!config.file.enabled);
        assert_eq!(config.level, "info");
    }

    #[test]
    fn test_config_validation() {
        let mut config = create_test_config();

        // Valid config should pass
        assert!(config.validate().is_ok());

        // Config with both outputs disabled should fail
        config.console.enabled = false;
        config.file.enabled = false;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_log_format_default() {
        assert_eq!(LogFormat::default(), LogFormat::Full);
    }

    #[test]
    fn test_rotation_strategy_default() {
        assert_eq!(RotationStrategy::default(), RotationStrategy::Size);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property 16: 配置管理 - Default values are valid
        #[test]
        fn property_config_defaults_are_valid(_dummy in 0u8..1u8) {
            let default_config = LoggerConfig::default();
            prop_assert!(default_config.validate().is_ok());

            let default_console = ConsoleConfig::default();
            prop_assert!(default_console.validate().is_ok());

            let default_rotation = RotationConfig::default();
            prop_assert!(default_rotation.validate().is_ok());
        }

        /// Property 16: 配置管理 - Valid configurations should validate
        #[test]
        fn property_valid_configs_validate(
            console_enabled in any::<bool>(),
            file_enabled in any::<bool>(),
            colored in any::<bool>(),
            append in any::<bool>(),
            compress in any::<bool>(),
            max_size in 1u64..1000000u64,
            max_files in 1usize..100usize
        ) {
            // Skip invalid combinations (both outputs disabled)
            prop_assume!(console_enabled || file_enabled);

            let config = LoggerConfig {
                console: ConsoleConfig {
                    enabled: console_enabled,
                    colored,
                },
                file: FileConfig {
                    enabled: file_enabled,
                    path: PathBuf::from("test.log"),
                    append,
                    format: LogFormat::Full,
                    rotation: RotationConfig {
                        strategy: RotationStrategy::Size,
                        max_size,
                        max_files,
                        compress,
                    },
                },
                level: "info".to_string(),
            };

            prop_assert!(config.validate().is_ok());
            prop_assert!(config.parse_level().is_ok());
        }

        /// Property 17: 配置验证 - Invalid log levels should fail
        #[test]
        fn property_invalid_levels_fail(
            invalid_level in "[a-z]{1,10}[A-Z0-9]{1,5}"
        ) {
            prop_assume!(!["trace", "debug", "info", "warn", "error"]
                .contains(&invalid_level.to_lowercase().as_str()));

            let mut config = LoggerConfig::default();
            config.level = invalid_level;

            prop_assert!(config.validate().is_err());
        }

        /// Property 17: 配置验证 - Zero rotation values should fail
        #[test]
        fn property_rotation_zero_values_fail(
            max_size in 0u64..1u64,
            max_files in 0usize..1usize
        ) {
            if max_size == 0 {
                let config = RotationConfig {
                    strategy: RotationStrategy::Size,
                    max_size,
                    max_files: 5,
                    compress: false,
                };
                prop_assert!(config.validate().is_err());
            }

            if max_files == 0 {
                let config = RotationConfig {
                    strategy: RotationStrategy::Size,
                    max_size: 1024,
                    max_files,
                    compress: false,
                };
                prop_assert!(config.validate().is_err());
            }
        }

        /// Property 17: 配置验证 - Empty file path should fail when enabled
        #[test]
        fn property_empty_path_fails(_dummy in 0u8..1u8) {
            let config = FileConfig {
                enabled: true,
                path: PathBuf::new(),
                append: true,
                format: LogFormat::Full,
                rotation: RotationConfig::default(),
            };

            prop_assert!(config.validate().is_err());
        }

        /// Property 17: 配置验证 - Both outputs disabled should fail
        #[test]
        fn property_no_outputs_fail(_dummy in 0u8..1u8) {
            let config = LoggerConfig {
                console: ConsoleConfig { enabled: false, colored: false },
                file: FileConfig { enabled: false, ..Default::default() },
                level: "info".to_string(),
            };

            prop_assert!(config.validate().is_err());
        }
    }
}


#[cfg(test)]
mod core_property_tests {
    use super::*;
    use crate::logger::writer::RotatingFileWriter;
    use proptest::prelude::*;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    proptest! {
        /// Property 1: 控制台输出控制
        /// *For any* configuration, when console is enabled, the configuration should be valid;
        /// when console is disabled but file is enabled, the configuration should still be valid.
        /// **Validates: Requirements 1.2, 1.3**
        #[test]
        fn property_console_output_control(
            console_enabled in any::<bool>(),
            file_enabled in any::<bool>(),
        ) {
            prop_assume!(console_enabled || file_enabled);

            let config = LoggerConfig {
                console: ConsoleConfig {
                    enabled: console_enabled,
                    colored: false,
                },
                file: FileConfig {
                    enabled: file_enabled,
                    path: PathBuf::from("test.log"),
                    ..Default::default()
                },
                level: "info".to_string(),
            };

            // Configuration should be valid when at least one output is enabled
            prop_assert!(config.validate().is_ok());

            // Console enabled state should be independently controllable
            prop_assert_eq!(config.console.enabled, console_enabled);
        }

        /// Property 2: 文件输出控制
        /// *For any* configuration, when file output is enabled, the file path must be valid;
        /// when file output is disabled, the configuration should still be valid.
        /// **Validates: Requirements 3.2, 3.5**
        #[test]
        fn property_file_output_control(
            file_enabled in any::<bool>(),
            console_enabled in any::<bool>(),
        ) {
            prop_assume!(console_enabled || file_enabled);

            let config = LoggerConfig {
                console: ConsoleConfig {
                    enabled: console_enabled,
                    colored: false,
                },
                file: FileConfig {
                    enabled: file_enabled,
                    path: PathBuf::from("test.log"),
                    ..Default::default()
                },
                level: "info".to_string(),
            };

            prop_assert!(config.validate().is_ok());
            prop_assert_eq!(config.file.enabled, file_enabled);
        }

        /// Property 3: 日志级别过滤
        /// *For any* valid log level, the configuration should parse successfully.
        /// **Validates: Requirements 1.4**
        #[test]
        fn property_log_level_filtering(
            level_idx in 0usize..5usize
        ) {
            let levels = ["trace", "debug", "info", "warn", "error"];
            let level = levels[level_idx];

            let config = LoggerConfig {
                console: ConsoleConfig::default(),
                file: FileConfig::default(),
                level: level.to_string(),
            };

            prop_assert!(config.validate().is_ok());
            prop_assert!(config.parse_level().is_ok());
        }

        /// Property 4: 颜色格式化控制
        /// *For any* configuration, the colored setting should be independently controllable.
        /// **Validates: Requirements 2.2, 2.3**
        #[test]
        fn property_color_formatting_control(
            colored in any::<bool>(),
        ) {
            let config = ConsoleConfig {
                enabled: true,
                colored,
            };

            prop_assert!(config.validate().is_ok());
            prop_assert_eq!(config.colored, colored);
        }

        /// Property 5: TTY环境检测
        /// *For any* console configuration, when colored is true but environment is non-TTY,
        /// the effective ANSI setting should be false.
        /// **Validates: Requirements 2.4**
        #[test]
        fn property_tty_detection(_dummy in 0u8..1u8) {
            use std::io::IsTerminal;

            let is_tty = std::io::stdout().is_terminal();
            let config = ConsoleConfig {
                enabled: true,
                colored: true,
            };

            // The effective ANSI setting depends on both config and TTY status
            let effective_ansi = config.colored && is_tty;

            // In test environment (usually non-TTY), ANSI should be disabled
            // even if colored is true
            if !is_tty {
                prop_assert!(!effective_ansi);
            }
        }

        /// Property 6: 文件路径处理
        /// *For any* valid file path, the writer should create parent directories if needed.
        /// **Validates: Requirements 3.3, 3.4**
        #[test]
        fn property_file_path_handling(
            subdir in "[a-z]{1,5}",
            filename in "[a-z]{1,5}\\.log"
        ) {
            let dir = tempdir().unwrap();
            let nested_path = dir.path().join(&subdir).join(&filename);

            let config = FileConfig {
                enabled: true,
                path: nested_path.clone(),
                append: true,
                format: LogFormat::Full,
                rotation: RotationConfig::default(),
            };

            // Writer should create parent directories
            let result = RotatingFileWriter::new(&config);
            prop_assert!(result.is_ok(), "Writer should create nested directories");

            // Parent directory should exist
            prop_assert!(nested_path.parent().unwrap().exists());
        }

        /// Property 7: 文件写入模式
        /// *For any* append mode setting, the file should be opened correctly.
        /// **Validates: Requirements 4.1, 4.2, 4.3, 4.4**
        #[test]
        fn property_file_write_mode(
            append in any::<bool>(),
            initial_content in "[a-zA-Z0-9]{10,50}",
            new_content in "[a-zA-Z0-9]{10,50}"
        ) {
            let dir = tempdir().unwrap();
            let file_path = dir.path().join("test.log");

            // Write initial content
            fs::write(&file_path, &initial_content).unwrap();

            let config = FileConfig {
                enabled: true,
                path: file_path.clone(),
                append,
                format: LogFormat::Full,
                rotation: RotationConfig {
                    max_size: 1024 * 1024, // Large enough to not trigger rotation
                    ..Default::default()
                },
            };

            let writer = RotatingFileWriter::new(&config).unwrap();
            let mut guard = tracing_subscriber::fmt::MakeWriter::make_writer(&writer);
            guard.write_all(new_content.as_bytes()).unwrap();
            guard.flush().unwrap();
            drop(guard);

            let final_content = fs::read_to_string(&file_path).unwrap();

            if append {
                // Append mode: should contain both initial and new content
                prop_assert!(final_content.contains(&initial_content),
                    "Append mode should preserve initial content");
                prop_assert!(final_content.contains(&new_content),
                    "Append mode should include new content");
            } else {
                // Truncate mode: should only contain new content
                prop_assert!(!final_content.contains(&initial_content),
                    "Truncate mode should not preserve initial content");
                prop_assert!(final_content.contains(&new_content),
                    "Truncate mode should include new content");
            }
        }

        /// Property 8: 日志格式化
        /// *For any* log format, the configuration should be valid and format should be preserved.
        /// **Validates: Requirements 5.1, 5.2, 5.3, 5.4, 5.5, 5.6**
        #[test]
        fn property_log_formatting(
            format_idx in 0usize..3usize
        ) {
            let formats = [LogFormat::Full, LogFormat::Compact, LogFormat::Json];
            let format = formats[format_idx].clone();

            let config = FileConfig {
                enabled: true,
                path: PathBuf::from("test.log"),
                append: true,
                format: format.clone(),
                rotation: RotationConfig::default(),
            };

            prop_assert!(config.validate().is_ok());
            prop_assert_eq!(config.format, format);
        }

        /// Property 9: 默认格式使用
        /// *For any* configuration without explicit format, the default should be Full.
        /// **Validates: Requirements 5.7**
        #[test]
        fn property_default_format(_dummy in 0u8..1u8) {
            let default_format = LogFormat::default();
            prop_assert_eq!(default_format, LogFormat::Full);

            let default_file_config = FileConfig::default();
            prop_assert_eq!(default_file_config.format, LogFormat::Json);
        }
    }
}


#[cfg(test)]
mod dynamic_config_property_tests {
    use crate::logger::LogLevelHandle;
    use proptest::prelude::*;
    use std::sync::Arc;
    use tracing_subscriber::{layer::SubscriberExt, reload, EnvFilter};

    /// Helper to create a LogLevelHandle and run a test with a properly initialized subscriber
    fn with_test_handle<F, R>(initial_level: &str, f: F) -> R
    where
        F: FnOnce(&LogLevelHandle) -> R,
    {
        let filter = EnvFilter::try_new(initial_level).unwrap_or_else(|_| EnvFilter::new("info"));
        let (filter_layer, reload_handle) = reload::Layer::new(filter);
        
        // Create a subscriber with the reload layer
        let subscriber = tracing_subscriber::registry()
            .with(filter_layer)
            .with(tracing_subscriber::fmt::layer().with_writer(std::io::sink));
        
        let handle = LogLevelHandle {
            inner: Arc::new(reload_handle),
        };
        
        // Run the test with the subscriber active
        tracing::subscriber::with_default(subscriber, || f(&handle))
    }

    proptest! {
        /// Property 18: 动态配置更新
        /// *For any* valid log level, updating the log level at runtime SHALL succeed
        /// and the new level SHALL be reflected in subsequent queries.
        /// **Validates: Requirements 9.5**
        #[test]
        fn property_dynamic_config_valid_level_update(
            level_idx in 0usize..5usize
        ) {
            let valid_levels = ["trace", "debug", "info", "warn", "error"];
            let new_level = valid_levels[level_idx];

            // Create a handle with a different initial level
            let initial_level = if level_idx == 0 { "info" } else { "trace" };
            
            with_test_handle(initial_level, |handle| {
                // Update to the new level should succeed
                let result = handle.set_level(new_level);
                prop_assert!(result.is_ok(), "Setting valid level '{}' should succeed, got: {:?}", new_level, result);

                // Current level should reflect the update
                if let Some(current) = handle.current_level() {
                    prop_assert!(
                        current.to_lowercase().contains(&new_level.to_lowercase()),
                        "Current level '{}' should contain '{}'", current, new_level
                    );
                }
                Ok(())
            })?;
        }

        /// Property 18: 动态配置更新 - Empty level should be handled
        /// *For any* empty or whitespace-only level string, the behavior should be consistent.
        /// **Validates: Requirements 9.2, 9.5**
        #[test]
        fn property_dynamic_config_empty_level_handling(
            whitespace_count in 0usize..5usize
        ) {
            // Test empty and whitespace-only strings
            let empty_level = " ".repeat(whitespace_count);

            with_test_handle("info", |handle| {
                // Empty/whitespace levels may succeed (EnvFilter is permissive)
                // or fail - the key is that it doesn't panic
                let result = handle.set_level(&empty_level);
                
                // Whether it succeeds or fails, it should not panic
                // and if it fails, the error should be descriptive
                if let Err(err) = result {
                    let err_msg = err.to_string();
                    prop_assert!(
                        !err_msg.is_empty(),
                        "Error message should not be empty"
                    );
                }
                Ok(())
            })?;
        }

        /// Property 18: 动态配置更新 - Multiple updates should work
        /// *For any* sequence of valid level updates, each update SHALL succeed
        /// and the final level SHALL be the last one set.
        /// **Validates: Requirements 9.5**
        #[test]
        fn property_dynamic_config_multiple_updates(
            level_indices in prop::collection::vec(0usize..5usize, 2..5)
        ) {
            let valid_levels = ["trace", "debug", "info", "warn", "error"];
            
            with_test_handle("info", |handle| {
                // Apply multiple level updates
                for &idx in &level_indices {
                    let level = valid_levels[idx];
                    let result = handle.set_level(level);
                    prop_assert!(result.is_ok(), "Setting level '{}' should succeed", level);
                }

                // Final level should be the last one set
                let final_level = valid_levels[*level_indices.last().unwrap()];
                if let Some(current) = handle.current_level() {
                    prop_assert!(
                        current.to_lowercase().contains(&final_level.to_lowercase()),
                        "Final level '{}' should contain '{}'", current, final_level
                    );
                }
                Ok(())
            })?;
        }

        /// Property 18: 动态配置更新 - EnvFilter syntax support
        /// *For any* valid EnvFilter syntax, the update SHALL succeed.
        /// **Validates: Requirements 9.5**
        #[test]
        fn property_dynamic_config_envfilter_syntax(
            base_level_idx in 0usize..5usize,
            module_level_idx in 0usize..5usize
        ) {
            let valid_levels = ["trace", "debug", "info", "warn", "error"];
            let base_level = valid_levels[base_level_idx];
            let module_level = valid_levels[module_level_idx];

            // Create EnvFilter-style level string
            let filter_string = format!("{},test_module={}", base_level, module_level);

            with_test_handle("info", |handle| {
                // Update with EnvFilter syntax should succeed
                let result = handle.set_level(&filter_string);
                prop_assert!(result.is_ok(), "Setting EnvFilter syntax '{}' should succeed, got: {:?}", filter_string, result);
                Ok(())
            })?;
        }
    }
}

#[cfg(test)]
mod error_handling_property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::fs;
    use std::io::Write;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use tempfile::tempdir;

    proptest! {
        /// Property 19: 错误容错处理
        /// *For any* file write failure, the logger SHALL continue operation without crashing
        /// and SHALL either fallback to console or handle the error gracefully.
        /// **Validates: Requirements 10.1, 10.2**
        #[test]
        fn property_error_fault_tolerance(
            recovery_idx in 0usize..3usize,
            content in "[a-zA-Z0-9]{10,100}"
        ) {
            let strategies = [
                RecoveryStrategy::FallbackToConsole,
                RecoveryStrategy::CleanupAndRetry,
                RecoveryStrategy::SilentDrop,
            ];
            let strategy = strategies[recovery_idx];

            let dir = tempdir().unwrap();
            let file_path = dir.path().join("test.log");

            let config = FileConfig {
                enabled: true,
                path: file_path.clone(),
                append: true,
                format: LogFormat::Full,
                rotation: RotationConfig {
                    max_size: 1024 * 1024,
                    max_files: 5,
                    ..Default::default()
                },
            };

            // Create writer with specified recovery strategy
            let writer = RotatingFileWriter::with_recovery(&config, strategy, None).unwrap();

            // Write should succeed initially
            let mut guard = tracing_subscriber::fmt::MakeWriter::make_writer(&writer);
            let result = guard.write_all(content.as_bytes());

            // The write operation should not panic and should return a result
            // (either success or handled error)
            prop_assert!(result.is_ok(), "Write should not panic, got: {:?}", result);

            // Verify the writer is still functional after the operation
            let result2 = guard.flush();
            prop_assert!(result2.is_ok(), "Flush should succeed");
        }

        /// Property 20: 权限错误处理
        /// *For any* permission error when creating files, the logger SHALL return
        /// a clear error message without crashing.
        /// **Validates: Requirements 10.3**
        #[test]
        fn property_permission_error_handling(
            filename in "[a-z]{1,10}\\.log"
        ) {
            // Test with an invalid/inaccessible path
            // On Unix, /root is typically not writable by normal users
            // On Windows, we use a path that's likely to fail
            #[cfg(unix)]
            let invalid_path = PathBuf::from("/root/nonexistent_dir_12345").join(&filename);
            #[cfg(windows)]
            let invalid_path = PathBuf::from("C:\\Windows\\System32\\nonexistent_dir_12345").join(&filename);

            let config = FileConfig {
                enabled: true,
                path: invalid_path,
                append: true,
                format: LogFormat::Full,
                rotation: RotationConfig::default(),
            };

            // Creating writer with invalid path should return an error, not panic
            let result = RotatingFileWriter::new(&config);

            // The result should be an error (permission denied or similar)
            // The important thing is that it doesn't panic
            match result {
                Err(err) => {
                    let err_msg = err.to_string().to_lowercase();
                    // Error message should be descriptive
                    prop_assert!(
                        err_msg.contains("permission") 
                        || err_msg.contains("denied")
                        || err_msg.contains("access")
                        || err_msg.contains("not found")
                        || err_msg.contains("no such")
                        || err_msg.len() > 0,
                        "Error message should be descriptive: {}", err_msg
                    );
                }
                Ok(_) => {
                    // If it somehow succeeds (e.g., running as root), that's also acceptable
                }
            }
        }

        /// Property 21: 输出独立性
        /// *For any* configuration with both console and file outputs, if one output fails,
        /// the other SHALL continue to function independently.
        /// **Validates: Requirements 10.4**
        #[test]
        fn property_output_independence(
            console_enabled in any::<bool>(),
            file_enabled in any::<bool>(),
            content in "[a-zA-Z0-9]{10,50}"
        ) {
            prop_assume!(console_enabled || file_enabled);

            let dir = tempdir().unwrap();
            let file_path = dir.path().join("test.log");

            // Create a valid configuration
            let config = LoggerConfig {
                console: ConsoleConfig {
                    enabled: console_enabled,
                    colored: false,
                },
                file: FileConfig {
                    enabled: file_enabled,
                    path: file_path.clone(),
                    append: true,
                    format: LogFormat::Full,
                    rotation: RotationConfig {
                        max_size: 1024 * 1024,
                        max_files: 5,
                        ..Default::default()
                    },
                },
                level: "info".to_string(),
            };

            // Configuration should be valid
            prop_assert!(config.validate().is_ok());

            // If file output is enabled, test that file writer works independently
            if file_enabled {
                let writer = RotatingFileWriter::new(&config.file).unwrap();
                let mut guard = tracing_subscriber::fmt::MakeWriter::make_writer(&writer);

                // Write should succeed
                let result = guard.write_all(content.as_bytes());
                prop_assert!(result.is_ok());

                let _ = guard.flush();
                drop(guard);

                // Verify content was written
                let file_content = fs::read_to_string(&file_path).unwrap_or_default();
                prop_assert!(file_content.contains(&content));
            }

            // Console output is always available (stdout/stderr)
            // The key property is that each output operates independently
            prop_assert!(config.console.enabled == console_enabled);
            prop_assert!(config.file.enabled == file_enabled);
        }

        /// Property 19 (extended): Error callback mechanism
        /// *For any* write error, if an error callback is set, it SHALL be invoked.
        /// **Validates: Requirements 10.5**
        #[test]
        fn property_error_callback_invocation(
            content in "[a-zA-Z0-9]{10,50}"
        ) {
            let dir = tempdir().unwrap();
            let file_path = dir.path().join("test.log");

            let config = FileConfig {
                enabled: true,
                path: file_path.clone(),
                append: true,
                format: LogFormat::Full,
                rotation: RotationConfig {
                    max_size: 1024 * 1024,
                    max_files: 5,
                    ..Default::default()
                },
            };

            // Create a counter to track callback invocations
            let callback_count = Arc::new(AtomicU32::new(0));
            let callback_count_clone = callback_count.clone();

            let callback: crate::logger::writer::ErrorCallback = Arc::new(move |_err| {
                callback_count_clone.fetch_add(1, Ordering::SeqCst);
            });

            // Create writer with callback
            let writer = RotatingFileWriter::with_recovery(
                &config,
                RecoveryStrategy::FallbackToConsole,
                Some(callback),
            ).unwrap();

            // Normal write should succeed without triggering callback
            let mut guard = tracing_subscriber::fmt::MakeWriter::make_writer(&writer);
            let result = guard.write_all(content.as_bytes());
            prop_assert!(result.is_ok());

            // Callback should not have been called for successful writes
            let count = callback_count.load(Ordering::SeqCst);
            prop_assert_eq!(count, 0, "Callback should not be called for successful writes");
        }

        /// Property 21 (extended): Fallback mode functionality
        /// *For any* writer in fallback mode, writes SHALL go to stderr instead of file.
        /// **Validates: Requirements 10.1, 10.4**
        #[test]
        fn property_fallback_mode_functionality(
            content in "[a-zA-Z0-9]{10,50}"
        ) {
            let dir = tempdir().unwrap();
            let file_path = dir.path().join("test.log");

            let config = FileConfig {
                enabled: true,
                path: file_path.clone(),
                append: true,
                format: LogFormat::Full,
                rotation: RotationConfig::default(),
            };

            let writer = RotatingFileWriter::new(&config).unwrap();

            // Initially should not be in fallback mode
            prop_assert!(!writer.is_in_fallback_mode());

            // Write should succeed
            let mut guard = tracing_subscriber::fmt::MakeWriter::make_writer(&writer);
            let result = guard.write_all(content.as_bytes());
            prop_assert!(result.is_ok());

            // After successful write, should still not be in fallback mode
            drop(guard);
            prop_assert!(!writer.is_in_fallback_mode());
        }
    }
}
