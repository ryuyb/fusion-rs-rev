//! Environment configuration for the application

use std::str::FromStr;
use serde::{Deserialize, Serialize};
use crate::config::error::ConfigError;

/// Application environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    /// Development environment
    Development,
    /// Test environment
    Test,
    /// Staging environment
    Staging,
    /// Production environment
    Production,
}

impl Environment {
    /// Environment variable name for reading the current environment
    pub const ENV_VAR: &'static str = "FUSION_APP_ENV";

    /// Read the environment from the `FUSION_APP_ENV` environment variable
    /// 
    /// Returns `Development` if the variable is not set or cannot be parsed.
    pub fn from_env() -> Self {
        std::env::var(Self::ENV_VAR)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_default()
    }

    /// Convert the environment to a string slice
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Test => "test",
            Environment::Staging => "staging",
            Environment::Production => "production",
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Environment::Development
    }
}

impl FromStr for Environment {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Environment::Development),
            "test" => Ok(Environment::Test),
            "staging" | "stage" => Ok(Environment::Staging),
            "production" | "prod" => Ok(Environment::Production),
            _ => Err(ConfigError::EnvVarError(format!(
                "Invalid environment '{}'. Valid values are: development, test, staging, production",
                s
            ))),
        }
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_from_str() {
        assert_eq!("development".parse::<Environment>().unwrap(), Environment::Development);
        assert_eq!("dev".parse::<Environment>().unwrap(), Environment::Development);
        assert_eq!("test".parse::<Environment>().unwrap(), Environment::Test);
        assert_eq!("staging".parse::<Environment>().unwrap(), Environment::Staging);
        assert_eq!("stage".parse::<Environment>().unwrap(), Environment::Staging);
        assert_eq!("production".parse::<Environment>().unwrap(), Environment::Production);
        assert_eq!("prod".parse::<Environment>().unwrap(), Environment::Production);
    }

    #[test]
    fn test_environment_case_insensitive() {
        assert_eq!("DEVELOPMENT".parse::<Environment>().unwrap(), Environment::Development);
        assert_eq!("Production".parse::<Environment>().unwrap(), Environment::Production);
    }

    #[test]
    fn test_environment_invalid() {
        assert!("invalid".parse::<Environment>().is_err());
    }

    #[test]
    fn test_environment_as_str() {
        assert_eq!(Environment::Development.as_str(), "development");
        assert_eq!(Environment::Test.as_str(), "test");
        assert_eq!(Environment::Staging.as_str(), "staging");
        assert_eq!(Environment::Production.as_str(), "production");
    }

    #[test]
    fn test_environment_default() {
        assert_eq!(Environment::default(), Environment::Development);
    }
}
