//! Configuration validation for CLI arguments

use crate::error::{Error, Result};
use std::path::Path;
use std::time::Duration;

/// Validate alert threshold format (e.g., "200%", "1.5x", "50ms")
pub fn validate_alert_threshold(threshold: &str) -> Result<()> {
    if threshold.is_empty() {
        return Err(Error::Validation(
            "Alert threshold cannot be empty".to_string(),
        ));
    }

    // Check for percentage format (e.g., "200%")
    if threshold.ends_with('%') {
        let num_str = threshold.strip_suffix('%').unwrap();
        if let Ok(num) = num_str.parse::<f64>() {
            if num > 0.0 {
                return Ok(());
            }
        }
        return Err(Error::Validation(
            "Alert threshold percentage must be greater than 0".to_string(),
        ));
    }

    // Check for multiplier format (e.g., "1.5x")
    if threshold.ends_with('x') {
        let num_str = threshold.strip_suffix('x').unwrap();
        if let Ok(num) = num_str.parse::<f64>() {
            if num > 0.0 {
                return Ok(());
            }
        }
        return Err(Error::Validation(
            "Alert threshold multiplier must be greater than 0".to_string(),
        ));
    }

    // Check for absolute time format (e.g., "50ms", "1s")
    if let Ok(duration) = parse_duration(threshold) {
        if !duration.is_zero() {
            return Ok(());
        }
        return Err(Error::Validation(
            "Alert threshold duration must be greater than 0".to_string(),
        ));
    }

    Err(Error::Validation(format!(
        "Invalid alert threshold format: '{}'. Expected formats: '200%', '1.5x', '50ms', '1s'",
        threshold
    )))
}

/// Parse duration string (e.g., "50ms", "1s", "1m")
fn parse_duration(duration_str: &str) -> Result<Duration> {
    if let Some(millis_str) = duration_str.strip_suffix("ms") {
        let millis: u64 = millis_str
            .parse()
            .map_err(|_| Error::Validation("Invalid millisecond duration".to_string()))?;
        Ok(Duration::from_millis(millis))
    } else if let Some(secs_str) = duration_str.strip_suffix('s') {
        let secs: u64 = secs_str
            .parse()
            .map_err(|_| Error::Validation("Invalid second duration".to_string()))?;
        Ok(Duration::from_secs(secs))
    } else if let Some(mins_str) = duration_str.strip_suffix('m') {
        let mins: u64 = mins_str
            .parse()
            .map_err(|_| Error::Validation("Invalid minute duration".to_string()))?;
        Ok(Duration::from_secs(mins * 60))
    } else {
        Err(Error::Validation("Invalid duration format".to_string()))
    }
}

/// Validate GitHub token format
pub fn validate_github_token(token: &str) -> Result<()> {
    if token.is_empty() {
        return Err(Error::Validation(
            "GitHub token cannot be empty".to_string(),
        ));
    }

    // GitHub tokens are typically 40 characters (classic) or start with specific prefixes
    if token.len() < 20 {
        return Err(Error::Validation(
            "GitHub token appears to be too short".to_string(),
        ));
    }

    // Check for valid characters (alphanumeric and some special chars)
    if !token
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(Error::Validation(
            "GitHub token contains invalid characters".to_string(),
        ));
    }

    Ok(())
}

/// Validate file path exists and is readable
pub fn validate_file_exists(path: &Path, description: &str) -> Result<()> {
    if !path.exists() {
        return Err(Error::Validation(format!(
            "{} does not exist: {}",
            description,
            path.display()
        )));
    }

    if !path.is_file() {
        return Err(Error::Validation(format!(
            "{} is not a file: {}",
            description,
            path.display()
        )));
    }

    Ok(())
}

/// Validate directory exists and is writable
pub fn validate_dir_writable(path: &Path, description: &str) -> Result<()> {
    if path.exists() && !path.is_dir() {
        return Err(Error::Validation(format!(
            "{} exists but is not a directory: {}",
            description,
            path.display()
        )));
    }

    // Try to create directory if it doesn't exist
    if !path.exists() {
        std::fs::create_dir_all(path).map_err(|e| {
            Error::Validation(format!(
                "Cannot create {}: {}: {}",
                description,
                path.display(),
                e
            ))
        })?;
    }

    // Test writability by creating a temporary file
    let test_file = path.join(".git_bench_write_test");
    match std::fs::write(&test_file, "test") {
        Ok(_) => {
            let _ = std::fs::remove_file(&test_file);
            Ok(())
        }
        Err(e) => Err(Error::Validation(format!(
            "{} is not writable: {}: {}",
            description,
            path.display(),
            e
        ))),
    }
}

/// Validate branch name format
pub fn validate_branch_name(branch: &str) -> Result<()> {
    if branch.is_empty() {
        return Err(Error::Validation("Branch name cannot be empty".to_string()));
    }

    // Git branch name rules
    if branch.len() > 255 {
        return Err(Error::Validation(
            "Branch name cannot exceed 255 characters".to_string(),
        ));
    }

    // Cannot start with dot or end with .lock
    if branch.starts_with('.') || branch.ends_with(".lock") {
        return Err(Error::Validation(
            "Branch name cannot start with '.' or end with '.lock'".to_string(),
        ));
    }

    // Cannot contain invalid characters
    let invalid_patterns = ["..", "@{", "~", "^", ":", "?", "*", "[", " ", "\t"];
    for invalid in &invalid_patterns {
        if branch.contains(invalid) {
            return Err(Error::Validation(format!(
                "Branch name cannot contain '{}'",
                invalid
            )));
        }
    }

    // Cannot end with .lock
    if branch.ends_with(".lock") {
        return Err(Error::Validation(
            "Branch name cannot end with '.lock'".to_string(),
        ));
    }

    Ok(())
}

/// Validate maximum items in chart
pub fn validate_max_items(max_items: Option<usize>) -> Result<()> {
    if let Some(max) = max_items {
        if max == 0 {
            return Err(Error::Validation(
                "Maximum items in chart cannot be 0".to_string(),
            ));
        }
        if max > 10000 {
            return Err(Error::Validation(
                "Maximum items in chart cannot exceed 10000".to_string(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_validate_alert_threshold() {
        // Valid formats
        assert!(validate_alert_threshold("200%").is_ok());
        assert!(validate_alert_threshold("1.5x").is_ok());
        assert!(validate_alert_threshold("50ms").is_ok());
        assert!(validate_alert_threshold("1s").is_ok());
        assert!(validate_alert_threshold("1m").is_ok());

        // Invalid formats
        assert!(validate_alert_threshold("").is_err());
        assert!(validate_alert_threshold("0%").is_err());
        assert!(validate_alert_threshold("-100%").is_err());
        assert!(validate_alert_threshold("0x").is_err());
        assert!(validate_alert_threshold("-1x").is_err());
        assert!(validate_alert_threshold("0ms").is_err());
        assert!(validate_alert_threshold("invalid").is_err());
    }

    #[test]
    fn test_validate_github_token() {
        // Valid tokens
        assert!(validate_github_token("ghp_1234567890abcdef1234567890abcdef12345678").is_ok());
        assert!(validate_github_token("1234567890abcdef1234567890abcdef12345678").is_ok());

        // Invalid tokens
        assert!(validate_github_token("").is_err());
        assert!(validate_github_token("short").is_err());
        assert!(validate_github_token("token with spaces").is_err());
    }

    #[test]
    fn test_validate_branch_name() {
        // Valid names
        assert!(validate_branch_name("main").is_ok());
        assert!(validate_branch_name("feature/test").is_ok());
        assert!(validate_branch_name("bugfix-123").is_ok());

        // Invalid names
        assert!(validate_branch_name("").is_err());
        assert!(validate_branch_name(".hidden").is_err());
        assert!(validate_branch_name("name.lock").is_err());
        assert!(validate_branch_name("name with space").is_err());
        assert!(validate_branch_name("name@{").is_err());
    }

    #[test]
    fn test_validate_max_items() {
        assert!(validate_max_items(None).is_ok());
        assert!(validate_max_items(Some(100)).is_ok());
        assert!(validate_max_items(Some(10000)).is_ok());

        assert!(validate_max_items(Some(0)).is_err());
        assert!(validate_max_items(Some(10001)).is_err());
    }

    #[test]
    fn test_validate_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "test").unwrap();

        assert!(validate_file_exists(&file_path, "Test file").is_ok());
        assert!(
            validate_file_exists(&temp_dir.path().join("nonexistent.txt"), "Test file").is_err()
        );
        assert!(validate_file_exists(&temp_dir.path(), "Test file").is_err());
    }

    #[test]
    fn test_validate_dir_writable() {
        let temp_dir = TempDir::new().unwrap();

        assert!(validate_dir_writable(temp_dir.path(), "Test dir").is_ok());

        let readonly_file = temp_dir.path().join("readonly");
        std::fs::write(&readonly_file, "test").unwrap();

        assert!(validate_dir_writable(&readonly_file, "Test dir").is_err());
    }
}
