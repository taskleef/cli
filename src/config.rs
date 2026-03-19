use std::fs;
use std::path::PathBuf;

use crate::error::{Result, TaskleefError};

#[derive(Debug, Clone)]
pub struct Config {
    pub api_url: String,
    pub api_key: String,
}

impl Config {
    pub fn load(auth_file: Option<&str>) -> Result<Self> {
        let mut api_key = std::env::var("TASKLEEF_API_KEY").ok();
        let api_url = std::env::var("TASKLEEF_API_URL")
            .unwrap_or_else(|_| "https://taskleef.com".to_string());

        if let Some(path) = auth_file {
            let expanded = expand_tilde(path);
            if !expanded.exists() {
                return Err(TaskleefError::AuthFileNotFound(path.to_string()));
            }
            let contents = fs::read_to_string(&expanded)?;
            if let Some(key) = parse_auth_file(&contents, "TASKLEEF_API_KEY") {
                api_key = Some(key);
            } else {
                return Err(TaskleefError::ApiKeyNotInAuthFile);
            }
        }

        let api_key = api_key.ok_or(TaskleefError::MissingApiKey)?;

        Ok(Config { api_url, api_key })
    }
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix('~') {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest.strip_prefix('/').unwrap_or(rest));
        }
    }
    PathBuf::from(path)
}

fn parse_auth_file(contents: &str, key: &str) -> Option<String> {
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let k = k.trim();
            let v = v.trim();
            if k == key {
                // Strip optional quotes
                let v = v
                    .strip_prefix('"')
                    .and_then(|v| v.strip_suffix('"'))
                    .or_else(|| v.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
                    .unwrap_or(v);
                return Some(v.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_auth_file_simple() {
        let contents = "TASKLEEF_API_KEY=my-secret-key\n";
        assert_eq!(
            parse_auth_file(contents, "TASKLEEF_API_KEY"),
            Some("my-secret-key".to_string())
        );
    }

    #[test]
    fn test_parse_auth_file_double_quoted() {
        let contents = "TASKLEEF_API_KEY=\"my-secret-key\"\n";
        assert_eq!(
            parse_auth_file(contents, "TASKLEEF_API_KEY"),
            Some("my-secret-key".to_string())
        );
    }

    #[test]
    fn test_parse_auth_file_single_quoted() {
        let contents = "TASKLEEF_API_KEY='my-secret-key'\n";
        assert_eq!(
            parse_auth_file(contents, "TASKLEEF_API_KEY"),
            Some("my-secret-key".to_string())
        );
    }

    #[test]
    fn test_parse_auth_file_with_comments() {
        let contents = "# This is a comment\nTASKLEEF_API_KEY=my-key\nOTHER=value\n";
        assert_eq!(
            parse_auth_file(contents, "TASKLEEF_API_KEY"),
            Some("my-key".to_string())
        );
    }

    #[test]
    fn test_parse_auth_file_missing_key() {
        let contents = "OTHER_KEY=value\n";
        assert_eq!(parse_auth_file(contents, "TASKLEEF_API_KEY"), None);
    }

    #[test]
    fn test_parse_auth_file_empty() {
        assert_eq!(parse_auth_file("", "TASKLEEF_API_KEY"), None);
    }

    #[test]
    fn test_expand_tilde() {
        let expanded = expand_tilde("~/.taskleef.auth");
        assert!(!expanded.to_string_lossy().starts_with('~'));
        assert!(expanded.to_string_lossy().ends_with(".taskleef.auth"));
    }

    #[test]
    fn test_expand_no_tilde() {
        let expanded = expand_tilde("/tmp/auth");
        assert_eq!(expanded, PathBuf::from("/tmp/auth"));
    }

    #[test]
    fn test_config_load_from_auth_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "TASKLEEF_API_KEY=file-key").unwrap();

        // Clear env vars for this test
        std::env::remove_var("TASKLEEF_API_KEY");

        let config = Config::load(Some(file.path().to_str().unwrap())).unwrap();
        assert_eq!(config.api_key, "file-key");
        assert_eq!(config.api_url, "https://taskleef.com");
    }

    #[test]
    fn test_config_load_auth_file_not_found() {
        std::env::remove_var("TASKLEEF_API_KEY");
        let result = Config::load(Some("/nonexistent/file"));
        assert!(result.is_err());
    }

    #[test]
    fn test_config_load_from_env() {
        std::env::set_var("TASKLEEF_API_KEY", "env-key");
        std::env::set_var("TASKLEEF_API_URL", "https://custom.example.com");

        let config = Config::load(None).unwrap();
        assert_eq!(config.api_key, "env-key");
        assert_eq!(config.api_url, "https://custom.example.com");

        // Clean up
        std::env::remove_var("TASKLEEF_API_URL");
    }

    #[test]
    fn test_config_load_no_key() {
        std::env::remove_var("TASKLEEF_API_KEY");
        let result = Config::load(None);
        assert!(result.is_err());
    }
}
