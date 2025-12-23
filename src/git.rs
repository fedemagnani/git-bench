//! Git operations for working with repositories

use crate::data::{AuthorInfo, CommitInfo};
use crate::error::{Error, Result};
use chrono::{TimeZone, Utc};
use git2::{BranchType, Repository};
use std::path::Path;

/// Get commit information from a local git repository
pub fn get_commit_info(repo_path: &Path, commit_ref: Option<&str>) -> Result<CommitInfo> {
    let repo = Repository::open(repo_path)?;

    let commit = if let Some(ref_name) = commit_ref {
        // Try to find the commit by ref or SHA
        let obj = repo.revparse_single(ref_name)?;
        obj.peel_to_commit()?
    } else {
        // Get HEAD commit
        let head = repo.head()?;
        head.peel_to_commit()?
    };

    let author = commit.author();
    let timestamp = Utc.timestamp_opt(commit.time().seconds(), 0).single()
        .ok_or_else(|| Error::GitError(git2::Error::from_str("Invalid timestamp")))?;

    Ok(CommitInfo {
        id: commit.id().to_string(),
        message: commit.message().unwrap_or("").lines().next().unwrap_or("").to_string(),
        timestamp,
        url: None, // Will be filled in by GitHub integration
        author: Some(AuthorInfo {
            name: author.name().unwrap_or("Unknown").to_string(),
            email: author.email().map(|s| s.to_string()),
            username: None,
        }),
    })
}

/// Get the remote URL for the repository
pub fn get_remote_url(repo_path: &Path, remote_name: Option<&str>) -> Result<Option<String>> {
    let repo = Repository::open(repo_path)?;
    let remote_name = remote_name.unwrap_or("origin");

    let result = match repo.find_remote(remote_name) {
        Ok(remote) => Ok(remote.url().map(|s| s.to_string())),
        Err(_) => Ok(None),
    };
    result
}

/// Check if a branch exists
pub fn branch_exists(repo_path: &Path, branch_name: &str) -> Result<bool> {
    let repo = Repository::open(repo_path)?;
    Ok(repo.find_branch(branch_name, BranchType::Local).is_ok()
        || repo.find_branch(branch_name, BranchType::Remote).is_ok())
}

/// Check out a branch
pub fn checkout_branch(repo_path: &Path, branch_name: &str) -> Result<()> {
    let repo = Repository::open(repo_path)?;

    // Find the branch
    let branch = repo
        .find_branch(branch_name, BranchType::Local)
        .or_else(|_| {
            // Try to find remote branch and create local tracking branch
            let remote_branch = repo.find_branch(&format!("origin/{}", branch_name), BranchType::Remote)?;
            let commit = remote_branch.get().peel_to_commit()?;
            repo.branch(branch_name, &commit, false)
        })?;

    let ref_name = branch.get().name().ok_or_else(|| {
        Error::GitError(git2::Error::from_str("Invalid branch reference"))
    })?;

    let obj = repo.revparse_single(ref_name)?;
    repo.checkout_tree(&obj, None)?;
    repo.set_head(ref_name)?;

    Ok(())
}

/// Create a new branch if it doesn't exist
pub fn create_branch_if_not_exists(repo_path: &Path, branch_name: &str) -> Result<bool> {
    let repo = Repository::open(repo_path)?;

    if repo.find_branch(branch_name, BranchType::Local).is_ok() {
        return Ok(false); // Branch already exists
    }

    let head = repo.head()?;
    let commit = head.peel_to_commit()?;
    repo.branch(branch_name, &commit, false)?;

    Ok(true)
}

/// Stage a file for commit
pub fn stage_file(repo_path: &Path, file_path: &Path) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    let mut index = repo.index()?;

    // Get relative path
    let relative_path = file_path
        .strip_prefix(repo_path)
        .unwrap_or(file_path);

    index.add_path(relative_path)?;
    index.write()?;

    Ok(())
}

/// Create a commit with staged changes
pub fn create_commit(repo_path: &Path, message: &str) -> Result<String> {
    let repo = Repository::open(repo_path)?;
    let mut index = repo.index()?;

    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let head = repo.head()?;
    let parent_commit = head.peel_to_commit()?;

    let sig = repo.signature()?;

    let commit_id = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        message,
        &tree,
        &[&parent_commit],
    )?;

    Ok(commit_id.to_string())
}

/// Fetch from a remote
pub fn fetch_remote(repo_path: &Path, remote_name: &str, branch: &str) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    let mut remote = repo.find_remote(remote_name)?;

    let refspec = format!("+refs/heads/{}:refs/remotes/{}/{}", branch, remote_name, branch);

    remote.fetch(&[&refspec], None, None)?;

    Ok(())
}

/// Push to a remote
pub fn push_to_remote(repo_path: &Path, remote_name: &str, branch: &str) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    let mut remote = repo.find_remote(remote_name)?;

    let refspec = format!("refs/heads/{}:refs/heads/{}", branch, branch);

    remote.push(&[&refspec], None)?;

    Ok(())
}

/// Get the root directory of a git repository
pub fn get_repo_root(start_path: &Path) -> Result<std::path::PathBuf> {
    let repo = Repository::discover(start_path)?;
    let workdir = repo.workdir().ok_or_else(|| {
        Error::GitError(git2::Error::from_str("Repository has no working directory"))
    })?;

    Ok(workdir.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn init_test_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();

        // Create initial commit
        let sig = repo.signature().unwrap_or_else(|_| {
            git2::Signature::now("Test", "test@example.com").unwrap()
        });

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
    fn test_branch_exists() {
        let dir = init_test_repo();

        // master/main should exist after init with commit
        let exists = branch_exists(dir.path(), "master").unwrap()
            || branch_exists(dir.path(), "main").unwrap();
        assert!(exists);

        // Random branch should not exist
        let not_exists = branch_exists(dir.path(), "nonexistent").unwrap();
        assert!(!not_exists);
    }

    #[test]
    fn test_create_branch() {
        let dir = init_test_repo();

        let created = create_branch_if_not_exists(dir.path(), "test-branch").unwrap();
        assert!(created);

        let exists = branch_exists(dir.path(), "test-branch").unwrap();
        assert!(exists);

        // Creating again should return false
        let created_again = create_branch_if_not_exists(dir.path(), "test-branch").unwrap();
        assert!(!created_again);
    }
}

