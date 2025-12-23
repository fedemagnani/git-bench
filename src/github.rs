//! GitHub API integration

use crate::data::{AuthorInfo, CommitInfo};
use crate::error::{Error, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;

/// GitHub API client
pub struct GitHubClient {
    client: reqwest::blocking::Client,
    token: Option<String>,
    api_base: String,
}

impl GitHubClient {
    /// Create a new GitHub client
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
                    .map_err(|_| Error::ConfigError("Invalid token format".to_string()))?,
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

    /// Check if we have authentication
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    /// Get commit information from GitHub
    pub fn get_commit(&self, owner: &str, repo: &str, sha: &str) -> Result<CommitInfo> {
        let url = format!("{}/repos/{}/{}/commits/{}", self.api_base, owner, repo, sha);

        let response: GitHubCommitResponse = self
            .client
            .get(&url)
            .send()?
            .error_for_status()
            .map_err(|e| Error::GitHubError(format!("Failed to get commit: {}", e)))?
            .json()?;

        Ok(CommitInfo {
            id: response.sha,
            message: response.commit.message.lines().next().unwrap_or("").to_string(),
            timestamp: response.commit.author.date,
            url: Some(response.html_url),
            author: Some(AuthorInfo {
                name: response.commit.author.name,
                email: Some(response.commit.author.email),
                username: response.author.map(|a| a.login),
            }),
        })
    }

    /// Create a commit comment
    pub fn create_commit_comment(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
        body: &str,
    ) -> Result<String> {
        if !self.is_authenticated() {
            return Err(Error::GitHubError(
                "GitHub token required for creating comments".to_string(),
            ));
        }

        let url = format!(
            "{}/repos/{}/{}/commits/{}/comments",
            self.api_base, owner, repo, sha
        );

        let payload = serde_json::json!({
            "body": body
        });

        let response: CommentResponse = self
            .client
            .post(&url)
            .json(&payload)
            .send()?
            .error_for_status()
            .map_err(|e| Error::GitHubError(format!("Failed to create comment: {}", e)))?
            .json()?;

        Ok(response.html_url)
    }

    /// Create an issue comment (for PRs)
    pub fn create_issue_comment(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        body: &str,
    ) -> Result<String> {
        if !self.is_authenticated() {
            return Err(Error::GitHubError(
                "GitHub token required for creating comments".to_string(),
            ));
        }

        let url = format!(
            "{}/repos/{}/{}/issues/{}/comments",
            self.api_base, owner, repo, issue_number
        );

        let payload = serde_json::json!({
            "body": body
        });

        let response: CommentResponse = self
            .client
            .post(&url)
            .json(&payload)
            .send()?
            .error_for_status()
            .map_err(|e| Error::GitHubError(format!("Failed to create issue comment: {}", e)))?
            .json()?;

        Ok(response.html_url)
    }

    /// Get the authenticated user
    pub fn get_authenticated_user(&self) -> Result<String> {
        if !self.is_authenticated() {
            return Err(Error::GitHubError("Not authenticated".to_string()));
        }

        let url = format!("{}/user", self.api_base);

        let response: UserResponse = self
            .client
            .get(&url)
            .send()?
            .error_for_status()
            .map_err(|e| Error::GitHubError(format!("Failed to get user: {}", e)))?
            .json()?;

        Ok(response.login)
    }
}

// GitHub API response types

#[derive(Debug, Deserialize)]
struct GitHubCommitResponse {
    sha: String,
    html_url: String,
    commit: GitHubCommitData,
    author: Option<GitHubUser>,
}

#[derive(Debug, Deserialize)]
struct GitHubCommitData {
    message: String,
    author: GitHubAuthor,
}

#[derive(Debug, Deserialize)]
struct GitHubAuthor {
    name: String,
    email: String,
    date: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    login: String,
}

#[derive(Debug, Deserialize)]
struct CommentResponse {
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct UserResponse {
    login: String,
}

/// Parse a GitHub repository URL or string into owner and repo
pub fn parse_github_repo(repo: &str) -> Result<(String, String)> {
    // Handle various formats:
    // - owner/repo
    // - https://github.com/owner/repo
    // - git@github.com:owner/repo.git
    // - github.com/owner/repo

    let repo = repo.trim();

    // Remove .git suffix if present
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

    // Try SSH format (git@github.com:owner/repo)
    if let Some(path) = repo.strip_prefix("git@github.com:") {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() == 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }

    // Try github.com/owner/repo format
    if let Some(path) = repo.strip_prefix("github.com/") {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }

    Err(Error::ConfigError(format!(
        "Could not parse GitHub repository from: {}",
        repo
    )))
}

/// Environment variables for GitHub Actions
#[derive(Debug, Clone, Default)]
pub struct GitHubActionsEnv {
    /// GITHUB_TOKEN
    pub token: Option<String>,
    /// GITHUB_REPOSITORY (owner/repo)
    pub repository: Option<String>,
    /// GITHUB_SHA
    pub sha: Option<String>,
    /// GITHUB_REF
    pub ref_name: Option<String>,
    /// GITHUB_EVENT_NAME
    pub event_name: Option<String>,
    /// GITHUB_WORKSPACE
    pub workspace: Option<String>,
    /// GITHUB_SERVER_URL
    pub server_url: Option<String>,
}

impl GitHubActionsEnv {
    /// Load environment variables from GitHub Actions
    pub fn from_env() -> Self {
        Self {
            token: std::env::var("GITHUB_TOKEN").ok(),
            repository: std::env::var("GITHUB_REPOSITORY").ok(),
            sha: std::env::var("GITHUB_SHA").ok(),
            ref_name: std::env::var("GITHUB_REF").ok(),
            event_name: std::env::var("GITHUB_EVENT_NAME").ok(),
            workspace: std::env::var("GITHUB_WORKSPACE").ok(),
            server_url: std::env::var("GITHUB_SERVER_URL").ok().or(Some("https://github.com".to_string())),
        }
    }

    /// Check if running in GitHub Actions
    pub fn is_github_actions() -> bool {
        std::env::var("GITHUB_ACTIONS").map(|v| v == "true").unwrap_or(false)
    }

    /// Get owner and repo from GITHUB_REPOSITORY
    pub fn get_owner_repo(&self) -> Option<(String, String)> {
        self.repository.as_ref().and_then(|r| parse_github_repo(r).ok())
    }

    /// Build a commit URL
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

    #[test]
    fn test_parse_github_repo_with_git_suffix() {
        let (owner, repo) = parse_github_repo("https://github.com/owner/repo.git").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_github_repo_bare_domain() {
        let (owner, repo) = parse_github_repo("github.com/owner/repo").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }
}

