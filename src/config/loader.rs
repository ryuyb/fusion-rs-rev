//! Configuration loader for fusion-rs
//!
//! This module provides the `ConfigLoader` struct that handles loading
//! configuration from multiple sources with proper precedence.

use std::path::{Path, PathBuf};

use config::{Config, Environment, File, FileFormat};

use crate::config::environment::Environment as AppEnvironment;
use crate::config::error::ConfigError;
use crate::config::settings::Settings;

/// Environment variable for configuration directory
const CONFIG_DIR_ENV: &str = "FUSION_CONFIG_DIR";

/// Environment variable for specific configuration file
const CONFIG_FILE_ENV: &str = "FUSION_CONFIG_FILE";

/// Default configuration directory
const DEFAULT_CONFIG_DIR: &str = "config";

/// Environment variable prefix for configuration overrides
const ENV_PREFIX: &str = "FUSION";

/// Separator for nested configuration keys in environment variables
const ENV_SEPARATOR: &str = "__";

/// Configuration loader that handles layered configuration loading
///
/// The loader supports the following configuration sources (in order of priority):
/// 1. `default.toml` - Base default configuration (required)
/// 2. `{environment}.toml` - Environment-specific configuration (optional)
/// 3. `local.toml` - Local development overrides (optional)
/// 4. `FUSION_*` environment variables (highest priority)
#[derive(Debug)]
pub struct ConfigLoader {
    /// Configuration directory path
    config_dir: PathBuf,
    /// Specific configuration file path (if set, skips layered loading)
    config_file: Option<PathBuf>,
    /// Current application environment
    environment: AppEnvironment,
}

impl ConfigLoader {
    /// Create a new configuration loader
    ///
    /// This reads environment variables to determine:
    /// - Configuration directory (`FUSION_CONFIG_DIR`)
    /// - Specific configuration file (`FUSION_CONFIG_FILE`)
    /// - Application environment (`FUSION_APP_ENV`)
    ///
    /// # Errors
    ///
    /// Returns an error if both `FUSION_CONFIG_DIR` and `FUSION_CONFIG_FILE` are set,
    /// as they are mutually exclusive.
    pub fn new() -> Result<Self, ConfigError> {
        let config_dir = std::env::var(CONFIG_DIR_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_CONFIG_DIR));

        let config_file = std::env::var(CONFIG_FILE_ENV).ok().map(PathBuf::from);

        // Check mutual exclusivity
        if config_file.is_some() && std::env::var(CONFIG_DIR_ENV).is_ok() {
            return Err(ConfigError::mutual_exclusivity(
                "FUSION_CONFIG_DIR and FUSION_CONFIG_FILE cannot both be set. \
                 Use FUSION_CONFIG_DIR for layered configuration or \
                 FUSION_CONFIG_FILE for a single configuration file.",
            ));
        }

        let environment = AppEnvironment::from_env();

        Ok(Self {
            config_dir,
            config_file,
            environment,
        })
    }

    /// Get the current application environment
    #[allow(dead_code)]
    pub fn environment(&self) -> AppEnvironment {
        self.environment
    }

    /// Get the configuration directory path
    #[allow(dead_code)]
    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    /// Load configuration from all sources
    ///
    /// If `FUSION_CONFIG_FILE` is set, loads only that file.
    /// Otherwise, performs layered loading from the configuration directory.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `default.toml` is not found (when using layered loading)
    /// - Configuration parsing fails
    /// - Configuration validation fails
    pub fn load(&self) -> Result<Settings, ConfigError> {
        let config = self.build_config()?;
        let settings: Settings = config.try_deserialize().map_err(|e| {
            ConfigError::ParseError(format!("Failed to deserialize configuration: {}", e))
        })?;

        // Validate the loaded settings
        settings.validate()?;

        Ok(settings)
    }

    /// Build the config::Config instance from all sources
    fn build_config(&self) -> Result<Config, ConfigError> {
        let builder = Config::builder();

        let builder = if let Some(ref config_file) = self.config_file {
            // Single file mode
            self.add_file_source(builder, config_file, true)?
        } else {
            // Layered loading mode
            self.build_layered_config(builder)?
        };

        // Add environment variables (always highest priority)
        // Note: Environment variables are case-insensitive and converted to lowercase
        // FUSION_SERVER__PORT -> server.port
        let builder = Self::add_env_source(builder);

        builder.build().map_err(ConfigError::from)
    }

    /// Build layered configuration from multiple files
    fn build_layered_config(
        &self,
        builder: config::ConfigBuilder<config::builder::DefaultState>,
    ) -> Result<config::ConfigBuilder<config::builder::DefaultState>, ConfigError> {
        // 1. Add default.toml (required)
        let default_path = self.config_dir.join("default.toml");
        let builder = self.add_file_source(builder, &default_path, true)?;

        // 2. Add {environment}.toml (optional)
        let env_path = self
            .config_dir
            .join(format!("{}.toml", self.environment.as_str()));
        let builder = self.add_file_source(builder, &env_path, false)?;

        // 3. Add local.toml (optional)
        let local_path = self.config_dir.join("local.toml");
        let builder = self.add_file_source(builder, &local_path, false)?;

        Ok(builder)
    }

    /// Add a file source to the config builder
    ///
    /// # Arguments
    ///
    /// * `builder` - The config builder to add the source to
    /// * `path` - Path to the configuration file
    /// * `required` - Whether the file is required to exist
    fn add_file_source(
        &self,
        builder: config::ConfigBuilder<config::builder::DefaultState>,
        path: &Path,
        required: bool,
    ) -> Result<config::ConfigBuilder<config::builder::DefaultState>, ConfigError> {
        if required && !path.exists() {
            return Err(ConfigError::file_not_found(format!(
                "Required configuration file not found: {}",
                path.display()
            )));
        }

        // Only add the file if it exists or is required
        // For optional files, we use File::new with required(false)
        Ok(builder.add_source(
            File::new(path.to_str().unwrap_or_default(), FileFormat::Toml).required(required),
        ))
    }

    /// Add environment variable source to the config builder
    ///
    /// Environment variables with prefix `FUSION_` are mapped to configuration keys.
    /// Double underscores (`__`) are used as separators for nested keys.
    ///
    /// Examples:
    /// - `FUSION_SERVER__PORT` -> `server.port`
    /// - `FUSION_DATABASE__URL` -> `database.url`
    fn add_env_source(
        builder: config::ConfigBuilder<config::builder::DefaultState>,
    ) -> config::ConfigBuilder<config::builder::DefaultState> {
        builder.add_source(
            Environment::with_prefix(ENV_PREFIX)
                .prefix_separator("_")
                .separator(ENV_SEPARATOR)
                .ignore_empty(true)
                .try_parsing(true),
        )
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            config_dir: PathBuf::from(DEFAULT_CONFIG_DIR),
            config_file: None,
            environment: AppEnvironment::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Global mutex to ensure tests run sequentially to avoid env var conflicts
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    /// Helper to create a temporary config directory with files
    fn setup_config_dir(files: &[(&str, &str)]) -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        for (name, content) in files {
            let path = temp_dir.path().join(name);
            fs::write(&path, content).expect("Failed to write config file");
        }
        temp_dir
    }

    /// Helper to safely set environment variables for a test
    struct EnvGuard {
        vars_to_restore: Vec<(String, Option<String>)>,
    }

    impl EnvGuard {
        fn new() -> Self {
            Self {
                vars_to_restore: Vec::new(),
            }
        }

        fn set(&mut self, key: &str, value: &str) {
            // Store original value for restoration
            let original = std::env::var(key).ok();
            self.vars_to_restore.push((key.to_string(), original));
            unsafe {
                std::env::set_var(key, value);
            }
        }

        fn remove(&mut self, key: &str) {
            // Store original value for restoration
            let original = std::env::var(key).ok();
            self.vars_to_restore.push((key.to_string(), original));
            unsafe {
                std::env::remove_var(key);
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            // Restore all environment variables
            for (key, original_value) in &self.vars_to_restore {
                unsafe {
                    match original_value {
                        Some(value) => std::env::set_var(key, value),
                        None => std::env::remove_var(key),
                    }
                }
            }
        }
    }

    #[test]
    fn test_config_loader_new_default() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let mut env = EnvGuard::new();

        // Clear environment variables
        env.remove("FUSION_CONFIG_DIR");
        env.remove("FUSION_CONFIG_FILE");
        env.remove("FUSION_APP_ENV");

        let loader = ConfigLoader::new().expect("Should create loader");
        assert_eq!(loader.config_dir, PathBuf::from("config"));
        assert!(loader.config_file.is_none());
        assert_eq!(loader.environment, AppEnvironment::Development);
    }

    #[test]
    fn test_config_loader_with_config_dir() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let mut env = EnvGuard::new();

        env.remove("FUSION_CONFIG_FILE");
        env.set("FUSION_CONFIG_DIR", "/custom/config");

        let loader = ConfigLoader::new().expect("Should create loader");
        assert_eq!(loader.config_dir, PathBuf::from("/custom/config"));
    }

    #[test]
    fn test_config_loader_with_config_file() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let mut env = EnvGuard::new();

        env.remove("FUSION_CONFIG_DIR");
        env.set("FUSION_CONFIG_FILE", "/path/to/config.toml");

        let loader = ConfigLoader::new().expect("Should create loader");
        assert_eq!(
            loader.config_file,
            Some(PathBuf::from("/path/to/config.toml"))
        );
    }

    #[test]
    fn test_config_loader_mutual_exclusivity_error() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let mut env = EnvGuard::new();

        env.set("FUSION_CONFIG_DIR", "/custom/config");
        env.set("FUSION_CONFIG_FILE", "/path/to/config.toml");

        let result = ConfigLoader::new();
        assert!(result.is_err());
        if let Err(ConfigError::MutualExclusivityError(msg)) = result {
            assert!(msg.contains("FUSION_CONFIG_DIR"));
            assert!(msg.contains("FUSION_CONFIG_FILE"));
        } else {
            panic!("Expected MutualExclusivityError");
        }
    }

    #[test]
    fn test_config_loader_environment_from_env() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let mut env = EnvGuard::new();

        env.remove("FUSION_CONFIG_DIR");
        env.remove("FUSION_CONFIG_FILE");
        env.set("FUSION_APP_ENV", "production");

        let loader = ConfigLoader::new().expect("Should create loader");
        assert_eq!(loader.environment, AppEnvironment::Production);
    }

    #[test]
    fn test_load_missing_default_toml() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let mut env = EnvGuard::new();

        let temp_dir = setup_config_dir(&[]);

        env.set("FUSION_CONFIG_DIR", temp_dir.path().to_str().unwrap());
        env.remove("FUSION_CONFIG_FILE");
        env.remove("FUSION_APP_ENV");

        let loader = ConfigLoader::new().expect("Should create loader");
        let result = loader.load();

        assert!(result.is_err());
        if let Err(ConfigError::FileNotFound(msg)) = result {
            assert!(msg.contains("default.toml"));
        } else {
            panic!("Expected FileNotFound error");
        }
    }

    #[test]
    fn test_load_default_toml_only() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let mut env = EnvGuard::new();

        let default_config = r#"
[application]
name = "test-app"
version = "1.0.0"

[server]
host = "127.0.0.1"
port = 3000
request_timeout = 30
keep_alive_timeout = 75

[database]
url = "postgres://localhost/test"
max_connections = 10
min_connections = 1
connection_timeout = 30

[logger]
level = "info"

[logger.console]
enabled = true
colored = true

[logger.file]
enabled = false
path = "logs/app.log"
append = true
format = "json"

[logger.file.rotation]
strategy = "size"
max_size = 10485760
max_files = 5
compress = false
"#;

        let temp_dir = setup_config_dir(&[("default.toml", default_config)]);

        env.set("FUSION_CONFIG_DIR", temp_dir.path().to_str().unwrap());
        env.remove("FUSION_CONFIG_FILE");
        env.remove("FUSION_APP_ENV");

        let loader = ConfigLoader::new().expect("Should create loader");
        let settings = loader.load().expect("Should load settings");

        assert_eq!(settings.application.name, "test-app");
        assert_eq!(settings.application.version, "1.0.0");
        assert_eq!(settings.server.port, 3000);
        assert_eq!(settings.database.url, "postgres://localhost/test");
    }

    #[test]
    fn test_load_with_environment_override() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let mut env = EnvGuard::new();

        let default_config = r#"
[application]
name = "test-app"
version = "1.0.0"

[server]
host = "127.0.0.1"
port = 3000
request_timeout = 30
keep_alive_timeout = 75

[database]
url = "postgres://localhost/test"
max_connections = 10
min_connections = 1
connection_timeout = 30

[logger]
level = "info"

[logger.console]
enabled = true
colored = true

[logger.file]
enabled = false
"#;

        let production_config = r#"
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgres://prod-server/production"
max_connections = 50
"#;

        let temp_dir = setup_config_dir(&[
            ("default.toml", default_config),
            ("production.toml", production_config),
        ]);

        env.set("FUSION_CONFIG_DIR", temp_dir.path().to_str().unwrap());
        env.remove("FUSION_CONFIG_FILE");
        env.set("FUSION_APP_ENV", "production");

        let loader = ConfigLoader::new().expect("Should create loader");
        let settings = loader.load().expect("Should load settings");

        // Values from production.toml should override default.toml
        assert_eq!(settings.server.host, "0.0.0.0");
        assert_eq!(settings.server.port, 8080);
        assert_eq!(settings.database.url, "postgres://prod-server/production");
        assert_eq!(settings.database.max_connections, 50);

        // Values not in production.toml should come from default.toml
        assert_eq!(settings.application.name, "test-app");
        assert_eq!(settings.server.request_timeout, 30);
        assert_eq!(settings.database.min_connections, 1);
    }

    #[test]
    fn test_load_with_local_override() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let mut env = EnvGuard::new();

        let default_config = r#"
[application]
name = "test-app"
version = "1.0.0"

[server]
host = "127.0.0.1"
port = 3000
request_timeout = 30
keep_alive_timeout = 75

[database]
url = "postgres://localhost/test"
max_connections = 10
min_connections = 1
connection_timeout = 30

[logger]
level = "info"

[logger.console]
enabled = true
colored = true

[logger.file]
enabled = false
"#;

        let local_config = r#"
[server]
port = 9999

[database]
url = "postgres://localhost/local_dev"
"#;

        let temp_dir = setup_config_dir(&[
            ("default.toml", default_config),
            ("local.toml", local_config),
        ]);

        env.set("FUSION_CONFIG_DIR", temp_dir.path().to_str().unwrap());
        env.remove("FUSION_CONFIG_FILE");
        env.remove("FUSION_APP_ENV");

        let loader = ConfigLoader::new().expect("Should create loader");
        let settings = loader.load().expect("Should load settings");

        // Values from local.toml should override default.toml
        assert_eq!(settings.server.port, 9999);
        assert_eq!(settings.database.url, "postgres://localhost/local_dev");

        // Values not in local.toml should come from default.toml
        assert_eq!(settings.application.name, "test-app");
        assert_eq!(settings.server.host, "127.0.0.1");
    }

    #[test]
    fn test_load_with_env_var_override() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let mut env = EnvGuard::new();

        let default_config = r#"
[application]
name = "test-app"
version = "1.0.0"

[server]
host = "127.0.0.1"
port = 3000
request_timeout = 30
keep_alive_timeout = 75

[database]
url = "postgres://localhost/test"
max_connections = 10
min_connections = 1
connection_timeout = 30

[logger]
level = "info"

[logger.console]
enabled = true
colored = true

[logger.file]
enabled = false
"#;

        let temp_dir = setup_config_dir(&[("default.toml", default_config)]);

        env.set("FUSION_CONFIG_DIR", temp_dir.path().to_str().unwrap());
        env.remove("FUSION_CONFIG_FILE");
        env.remove("FUSION_APP_ENV");

        // Set environment variable overrides
        // The config crate converts env var names to lowercase and uses __ as separator
        env.set("FUSION_SERVER__PORT", "4000");
        env.set("FUSION_DATABASE__URL", "postgres://env-override/db");

        let loader = ConfigLoader::new().expect("Should create loader");
        let settings = loader.load().expect("Should load settings");

        // Environment variables should override file values
        assert_eq!(settings.server.port, 4000);
        assert_eq!(settings.database.url, "postgres://env-override/db");

        // Values not overridden should come from default.toml
        assert_eq!(settings.application.name, "test-app");
        assert_eq!(settings.server.host, "127.0.0.1");
    }

    #[test]
    fn test_load_full_precedence_chain() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let mut env = EnvGuard::new();

        let default_config = r#"
[application]
name = "default-app"
version = "1.0.0"

[server]
host = "127.0.0.1"
port = 3000
request_timeout = 30
keep_alive_timeout = 75

[database]
url = "postgres://default/db"
max_connections = 10
min_connections = 1
connection_timeout = 30

[logger]
level = "info"

[logger.console]
enabled = true
colored = true

[logger.file]
enabled = false
"#;

        let development_config = r#"
[application]
name = "dev-app"

[server]
port = 3001

[database]
url = "postgres://dev/db"
"#;

        let local_config = r#"
[server]
port = 3002

[database]
url = "postgres://local/db"
"#;

        let temp_dir = setup_config_dir(&[
            ("default.toml", default_config),
            ("development.toml", development_config),
            ("local.toml", local_config),
        ]);

        env.set("FUSION_CONFIG_DIR", temp_dir.path().to_str().unwrap());
        env.remove("FUSION_CONFIG_FILE");
        env.remove("FUSION_APP_ENV"); // defaults to development

        // Set environment variable override (highest priority)
        env.set("FUSION_SERVER__PORT", "3003");

        let loader = ConfigLoader::new().expect("Should create loader");
        let settings = loader.load().expect("Should load settings");

        // Environment variable has highest priority
        assert_eq!(settings.server.port, 3003);

        // local.toml overrides development.toml for database.url
        assert_eq!(settings.database.url, "postgres://local/db");

        // development.toml overrides default.toml for application.name
        assert_eq!(settings.application.name, "dev-app");

        // default.toml provides base values
        assert_eq!(settings.server.host, "127.0.0.1");
        assert_eq!(settings.application.version, "1.0.0");
    }

    #[test]
    fn test_load_single_file_mode() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let mut env = EnvGuard::new();

        let single_config = r#"
[application]
name = "single-file-app"
version = "2.0.0"

[server]
host = "0.0.0.0"
port = 5000
request_timeout = 60
keep_alive_timeout = 120

[database]
url = "postgres://single/db"
max_connections = 20
min_connections = 2
connection_timeout = 60

[logger]
level = "debug"

[logger.console]
enabled = true
colored = false

[logger.file]
enabled = false
"#;

        let temp_dir = setup_config_dir(&[("single.toml", single_config)]);
        let config_file_path = temp_dir.path().join("single.toml");

        env.remove("FUSION_CONFIG_DIR");
        env.set("FUSION_CONFIG_FILE", config_file_path.to_str().unwrap());
        env.remove("FUSION_APP_ENV");

        let loader = ConfigLoader::new().expect("Should create loader");
        let settings = loader.load().expect("Should load settings");

        assert_eq!(settings.application.name, "single-file-app");
        assert_eq!(settings.application.version, "2.0.0");
        assert_eq!(settings.server.host, "0.0.0.0");
        assert_eq!(settings.server.port, 5000);
        assert_eq!(settings.database.url, "postgres://single/db");
    }

    #[test]
    fn test_optional_files_not_required() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let mut env = EnvGuard::new();

        // Only default.toml exists, no environment or local files
        let default_config = r#"
[application]
name = "test-app"
version = "1.0.0"

[server]
host = "127.0.0.1"
port = 3000
request_timeout = 30
keep_alive_timeout = 75

[database]
url = "postgres://localhost/test"
max_connections = 10
min_connections = 1
connection_timeout = 30

[logger]
level = "info"

[logger.console]
enabled = true
colored = true

[logger.file]
enabled = false
"#;

        let temp_dir = setup_config_dir(&[("default.toml", default_config)]);

        env.set("FUSION_CONFIG_DIR", temp_dir.path().to_str().unwrap());
        env.remove("FUSION_CONFIG_FILE");
        env.set("FUSION_APP_ENV", "staging"); // staging.toml doesn't exist

        let loader = ConfigLoader::new().expect("Should create loader");
        // Should succeed even though staging.toml and local.toml don't exist
        let settings = loader.load().expect("Should load settings");

        assert_eq!(settings.application.name, "test-app");
    }
}
