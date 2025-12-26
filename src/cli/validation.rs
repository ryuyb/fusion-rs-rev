//! CLI argument validation functions
//!
//! This module provides custom validation functions for CLI arguments
//! that go beyond what clap can validate automatically.

use std::path::PathBuf;
use std::fs;

/// Validate port number is within valid range (1-65535)
pub fn validate_port(port_str: &str) -> Result<u16, String> {
    let port: u16 = port_str.parse()
        .map_err(|_| format!("Port must be a valid number between 1 and 65535, got: '{}'", port_str))?;
    
    if port == 0 {
        return Err("Port must be between 1 and 65535. Port 0 is not allowed.".to_string());
    }
    
    Ok(port)
}

/// Validate that a file path is accessible (exists and is readable)
pub fn validate_config_file_path(path_str: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path_str);
    
    // Check if file exists
    if !path.exists() {
        return Err(format!("Configuration file does not exist: '{}'", path_str));
    }
    
    // Check if it's a file (not a directory)
    if !path.is_file() {
        return Err(format!("Configuration path is not a file: '{}'", path_str));
    }
    
    // Check if file is readable
    match fs::File::open(&path) {
        Ok(_) => Ok(path),
        Err(e) => Err(format!("Cannot read configuration file '{}': {}", path_str, e))
    }
}

/// Validate rollback steps is a positive number
pub fn validate_rollback_steps(steps_str: &str) -> Result<u32, String> {
    let steps: u32 = steps_str.parse()
        .map_err(|_| format!("Rollback steps must be a valid positive number, got: '{}'", steps_str))?;
    
    if steps == 0 {
        return Err("Rollback steps must be greater than 0".to_string());
    }
    
    // Reasonable upper limit to prevent accidental mass rollbacks
    if steps > 100 {
        return Err("Rollback steps cannot exceed 100 for safety reasons".to_string());
    }
    
    Ok(steps)
}

/// Validate host address format (basic validation)
pub fn validate_host_address(host_str: &str) -> Result<String, String> {
    let host = host_str.trim();
    
    if host.is_empty() {
        return Err("Host address cannot be empty".to_string());
    }
    
    // Check for common invalid characters
    if host.contains(' ') {
        return Err("Host address cannot contain spaces".to_string());
    }
    
    // Basic validation for common formats
    if host == "localhost" || host == "0.0.0.0" || host.starts_with("127.") {
        return Ok(host.to_string());
    }
    
    // Basic IPv4 validation
    if host.chars().all(|c| c.is_ascii_digit() || c == '.') {
        let parts: Vec<&str> = host.split('.').collect();
        if parts.len() == 4 {
            for part in parts {
                if part.parse::<u8>().is_err() {
                    return Err(format!("Invalid IPv4 address format: '{}'", host_str));
                }
            }
            return Ok(host.to_string());
        }
    }
    
    // For other formats (hostnames, IPv6), do basic validation
    if host.len() > 253 {
        return Err("Host address is too long (maximum 253 characters)".to_string());
    }
    
    // Allow hostnames and other valid formats
    Ok(host.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_validation_valid_ports() {
        let valid_ports = ["1", "80", "443", "3000", "8080", "65535"];
        
        for port_str in valid_ports {
            let result = validate_port(port_str);
            assert!(result.is_ok(), "Port {} should be valid", port_str);
        }
    }

    #[test]
    fn test_port_validation_invalid_ports() {
        let invalid_ports = ["0", "65536", "99999", "abc", "-1", ""];
        
        for port_str in invalid_ports {
            let result = validate_port(port_str);
            assert!(result.is_err(), "Port {} should be invalid", port_str);
        }
    }

    #[test]
    fn test_host_validation_valid_hosts() {
        let valid_hosts = [
            "localhost",
            "127.0.0.1",
            "0.0.0.0",
            "192.168.1.1",
            "10.0.0.1",
            "example.com",
            "my-server.local"
        ];
        
        for host in valid_hosts {
            let result = validate_host_address(host);
            assert!(result.is_ok(), "Host {} should be valid", host);
        }
    }

    #[test]
    fn test_host_validation_invalid_hosts() {
        let invalid_hosts = [
            "",
            "   ",
            "host with spaces",
            "999.999.999.999",
            &"x".repeat(300),
        ];
        
        for host in invalid_hosts {
            let result = validate_host_address(host);
            assert!(result.is_err(), "Host '{}' should be invalid", host);
        }
    }

    #[test]
    fn test_rollback_steps_validation_valid() {
        let valid_steps = ["1", "5", "10", "50", "100"];
        
        for steps_str in valid_steps {
            let result = validate_rollback_steps(steps_str);
            assert!(result.is_ok(), "Steps {} should be valid", steps_str);
        }
    }

    #[test]
    fn test_rollback_steps_validation_invalid() {
        let invalid_steps = ["0", "101", "999", "-1", "abc", ""];
        
        for steps_str in invalid_steps {
            let result = validate_rollback_steps(steps_str);
            assert!(result.is_err(), "Steps '{}' should be invalid", steps_str);
        }
    }
}
