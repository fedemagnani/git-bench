//! GitHub API integration

use crate::error::{Error, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;

/// GitHub API client
pub struct GitHubClient {
    client: reqwest::blocking::Client,
    token: Option<String>,
    #[allow(dead_code)]
    api_base: String,
}

impl GitHubClient {
    pub fn new(token: Option<String>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github.v3+json"),
        );
        headers.insert(USER_AGENT, HeaderValue::from_static("git-bench"));

        if let Some(ref t) = token {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", t))
                    .map_err(|_| Error::GitHub("Invalid token format".to_string()))?,
            );
        }

        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            client,
            token,
            api_base: "https://api.github.com".to_string(),
        })
    }

    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    pub fn create_commit_comment(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
        body: &str,
    ) -> Result<String> {
        if !self.is_authenticated() {
            return Err(Error::GitHub(
                "GitHub token required for creating comments".to_string(),
            ));
        }

        let url = format!(
            "{}/repos/{}/{}/commits/{}/comments",
            self.api_base, owner, repo, sha
        );

        let payload = serde_json::json!({ "body": body });

        let response: CommentResponse = self
            .client
            .post(&url)
            .json(&payload)
            .send()?
            .error_for_status()
            .map_err(|e| Error::GitHub(format!("Failed to create comment: {}", e)))?
            .json()?;

        Ok(response.html_url)
    }
}

#[derive(Debug, Deserialize)]
struct CommentResponse {
    html_url: String,
}

/// Parse a GitHub repository URL or string into owner and repo
pub fn parse_github_repo(repo: &str) -> Result<(String, String)> {
    let repo = repo.trim();
    let repo = repo.strip_suffix(".git").unwrap_or(repo);

    // Try simple owner/repo format
    if !repo.contains("://") && !repo.contains('@') && !repo.contains("github.com") {
        let parts: Vec<&str> = repo.split('/').collect();
        if parts.len() == 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }

    // Try URL format
    if let Ok(url) = url::Url::parse(repo) {
        let path = url.path().trim_start_matches('/');
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }

    // Try SSH format
    if let Some(path) = repo.strip_prefix("git@github.com:") {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() == 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }

    // Try github.com/owner/repo
    if let Some(path) = repo.strip_prefix("github.com/") {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }

    Err(Error::GitHub(format!(
        "Could not parse GitHub repository from: {}",
        repo
    )))
}

/// Environment variables for GitHub Actions
#[derive(Debug, Clone, Default)]
pub struct GitHubActionsEnv {
    pub token: Option<String>,
    pub repository: Option<String>,
    pub sha: Option<String>,
    pub server_url: Option<String>,
}

impl GitHubActionsEnv {
    pub fn from_env() -> Self {
        Self {
            token: std::env::var("GITHUB_TOKEN").ok(),
            repository: std::env::var("GITHUB_REPOSITORY").ok(),
            sha: std::env::var("GITHUB_SHA").ok(),
            server_url: std::env::var("GITHUB_SERVER_URL")
                .ok()
                .or(Some("https://github.com".to_string())),
        }
    }

    pub fn is_github_actions() -> bool {
        std::env::var("GITHUB_ACTIONS")
            .map(|v| v == "true")
            .unwrap_or(false)
    }

    pub fn get_owner_repo(&self) -> Option<(String, String)> {
        self.repository
            .as_ref()
            .and_then(|r| parse_github_repo(r).ok())
    }

    pub fn commit_url(&self, sha: &str) -> Option<String> {
        let (owner, repo) = self.get_owner_repo()?;
        let server = self.server_url.as_ref()?;
        Some(format!("{}/{}/{}/commit/{}", server, owner, repo, sha))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_repo_simple() {
        let (owner, repo) = parse_github_repo("owner/repo").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_github_repo_https() {
        let (owner, repo) = parse_github_repo("https://github.com/owner/repo").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_github_repo_ssh() {
        let (owner, repo) = parse_github_repo("git@github.com:owner/repo.git").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }
}
