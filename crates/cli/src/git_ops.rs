//! Core git operations using libgit2

use crate::error::{Error, Result};
use chrono::{TimeZone, Utc};
use git2::{BranchType, Repository};
use git_bench_core::{AuthorInfo, CommitInfo};
use std::path::Path;

/// Try to extract GitHub username from email or git name
pub fn extract_github_username(email: &Option<String>, name: &str) -> Option<String> {
    // Try GitHub noreply email formats:
    // - username@users.noreply.github.com
    // - 12345+username@users.noreply.github.com
    if let Some(email) = email {
        if email.ends_with("@users.noreply.github.com") {
            let local_part = email.split('@').next()?;
            // Handle "id+username" format
            if let Some(pos) = local_part.find('+') {
                return Some(local_part[pos + 1..].to_string());
            }
            return Some(local_part.to_string());
        }
    }

    // Fallback: use name if it looks like a username (no spaces, reasonable length)
    if !name.contains(' ') && !name.is_empty() && name.len() <= 39 {
        return Some(name.to_string());
    }

    None
}

/// Extract GitHub repo URL from git remote (origin)
pub fn get_github_commit_url(repo: &Repository, commit_id: &str) -> Option<String> {
    let remote = repo.find_remote("origin").ok()?;
    let url = remote.url()?;

    // Parse various GitHub URL formats:
    // - https://github.com/owner/repo.git
    // - https://github.com/owner/repo
    // - git@github.com:owner/repo.git
    // - ssh://git@github.com:owner/repo.git

    let repo_path = if url.starts_with("git@github.com:") {
        // SSH format: git@github.com:owner/repo.git
        url.strip_prefix("git@github.com:")?
            .trim_end_matches(".git")
    } else if url.contains("github.com/") {
        // HTTPS format
        let start = url.find("github.com/")? + "github.com/".len();
        url[start..].trim_end_matches(".git")
    } else {
        return None;
    };

    Some(format!(
        "https://github.com/{}/commit/{}",
        repo_path, commit_id
    ))
}

/// Get commit information from a local git repository
pub fn get_commit_info(repo_path: &Path, commit_ref: Option<&str>) -> Result<CommitInfo> {
    let repo = Repository::open(repo_path)?;

    let commit = if let Some(ref_name) = commit_ref {
        let obj = repo.revparse_single(ref_name)?;
        obj.peel_to_commit()?
    } else {
        let head = repo.head()?;
        head.peel_to_commit()?
    };

    let author = commit.author();
    let timestamp = Utc
        .timestamp_opt(commit.time().seconds(), 0)
        .single()
        .ok_or_else(|| Error::Git(git2::Error::from_str("Invalid timestamp")))?;

    let name = author.name().unwrap_or("Unknown").to_string();
    let email = author.email().map(|s| s.to_string());

    // Try to extract GitHub username from email or name
    let username = extract_github_username(&email, &name);

    let commit_id = commit.id().to_string();

    // Try to get commit URL from GitHub remote
    let url = get_github_commit_url(&repo, &commit_id);

    Ok(CommitInfo {
        id: commit_id,
        message: commit
            .message()
            .unwrap_or("")
            .lines()
            .next()
            .unwrap_or("")
            .to_string(),
        timestamp,
        url,
        author: Some(AuthorInfo {
            name,
            email,
            username,
        }),
    })
}

/// Check out a branch
pub fn checkout_branch(repo_path: &Path, branch_name: &str) -> Result<()> {
    let repo = Repository::open(repo_path)?;

    let branch = repo
        .find_branch(branch_name, BranchType::Local)
        .or_else(|_| {
            let remote_branch =
                repo.find_branch(&format!("origin/{}", branch_name), BranchType::Remote)?;
            let commit = remote_branch.get().peel_to_commit()?;
            repo.branch(branch_name, &commit, false)
        })?;

    let ref_name = branch
        .get()
        .name()
        .ok_or_else(|| Error::Git(git2::Error::from_str("Invalid branch reference")))?;

    let obj = repo.revparse_single(ref_name)?;
    repo.checkout_tree(&obj, None)?;
    repo.set_head(ref_name)?;

    Ok(())
}

/// Fetch from a remote
pub fn fetch_remote(repo_path: &Path, remote_name: &str, branch: &str) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    let mut remote = repo.find_remote(remote_name)?;

    let refspec = format!(
        "+refs/heads/{}:refs/remotes/{}/{}",
        branch, remote_name, branch
    );

    remote.fetch(&[&refspec], None, None)?;

    Ok(())
}

/// Create an orphan branch (no history)
pub fn create_orphan_branch(repo_path: &Path, branch_name: &str) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["checkout", "--orphan", branch_name])
        .current_dir(repo_path)
        .output()
        .map_err(|e| Error::Io(format!("Failed to create orphan branch: {}", e)))?;

    if !output.status.success() {
        return Err(Error::Io(format!(
            "Failed to create orphan branch: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let _ = std::process::Command::new("git")
        .args(["rm", "-rf", "--cached", "."])
        .current_dir(repo_path)
        .output();

    // Clean working directory except .git
    if let Ok(entries) = std::fs::read_dir(repo_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.file_name().map(|n| n != ".git").unwrap_or(false) {
                if path.is_dir() {
                    let _ = std::fs::remove_dir_all(&path);
                } else {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    }

    Ok(())
}

/// Push to remote with force
pub fn push_to_remote(repo_path: &Path, remote_name: &str, branch: &str) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["push", remote_name, branch, "--force"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| Error::Io(format!("Failed to push: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("Everything up-to-date") {
            return Err(Error::Other(format!(
                "Failed to push to {}: {}",
                branch, stderr
            )));
        }
    }

    Ok(())
}

/// Check if repository has uncommitted changes
pub fn has_uncommitted_changes(repo: &Repository) -> Result<bool> {
    let statuses = repo.statuses(None)?;
    let has_changes = statuses.iter().any(|s| {
        s.status().intersects(
            git2::Status::WT_MODIFIED
                | git2::Status::WT_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_NEW,
        )
    });
    Ok(has_changes)
}

/// Stash current changes
pub fn stash_changes(repo_path: &Path, message: &str) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["stash", "push", "-m", message])
        .current_dir(repo_path)
        .output()
        .map_err(|e| Error::Io(format!("Failed to stash changes: {}", e)))?;

    if !output.status.success() {
        return Err(Error::Other(format!(
            "Failed to stash changes: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Restore stashed changes
pub fn pop_stash(repo_path: &Path) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["stash", "pop"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| Error::Io(format!("Failed to pop stash: {}", e)))?;

    if !output.status.success() {
        return Err(Error::Other(format!(
            "Failed to pop stash: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn init_test_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();

        let sig = repo
            .signature()
            .unwrap_or_else(|_| git2::Signature::now("Test", "test@example.com").unwrap());

        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();

        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap();

        dir
    }

    #[test]
    fn test_get_commit_info() {
        let dir = init_test_repo();
        let info = get_commit_info(dir.path(), None).unwrap();
        assert_eq!(info.message, "Initial commit");
        assert!(!info.id.is_empty());
    }

    #[test]
    fn test_extract_github_username() {
        // Test noreply email formats
        assert_eq!(
            extract_github_username(
                &Some("username@users.noreply.github.com".to_string()),
                "test"
            ),
            Some("username".to_string())
        );
        assert_eq!(
            extract_github_username(
                &Some("12345+username@users.noreply.github.com".to_string()),
                "test"
            ),
            Some("username".to_string())
        );

        // Test fallback to name
        assert_eq!(
            extract_github_username(&None, "validusername"),
            Some("validusername".to_string())
        );

        // Test invalid cases
        assert_eq!(extract_github_username(&None, "invalid username"), None);
        assert_eq!(extract_github_username(&None, ""), None);
    }
}
