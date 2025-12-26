//! Git operations for working with repositories

use crate::error::{Error, Result};
use chrono::{TimeZone, Utc};
use git2::{BranchType, Repository};
use git_bench_core::{AuthorInfo, BenchmarkData, CommitInfo};
use std::path::Path;

/// Try to extract GitHub username from email or git name
fn extract_github_username(email: &Option<String>, name: &str) -> Option<String> {
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
fn get_github_commit_url(repo: &Repository, commit_id: &str) -> Option<String> {
    let remote = repo.find_remote("origin").ok()?;
    let url = remote.url()?;

    // Parse various GitHub URL formats:
    // - https://github.com/owner/repo.git
    // - https://github.com/owner/repo
    // - git@github.com:owner/repo.git
    // - ssh://git@github.com/owner/repo.git

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

/// Recursively collect all files from a directory into memory
/// Returns Vec of (relative_path, content)
fn collect_dir_files(src: &Path) -> Result<Vec<(std::path::PathBuf, Vec<u8>)>> {
    let mut files = Vec::new();
    collect_dir_files_recursive(src, src, &mut files)?;
    Ok(files)
}

fn collect_dir_files_recursive(
    base: &Path,
    current: &Path,
    files: &mut Vec<(std::path::PathBuf, Vec<u8>)>,
) -> Result<()> {
    for entry in std::fs::read_dir(current)
        .map_err(|e| Error::Other(format!("Failed to read dir '{}': {}", current.display(), e)))?
    {
        let entry = entry.map_err(|e| Error::Other(format!("Failed to read entry: {}", e)))?;
        let path = entry.path();

        if path.is_dir() {
            collect_dir_files_recursive(base, &path, files)?;
        } else {
            let relative = path.strip_prefix(base).unwrap_or(&path).to_path_buf();
            let content = std::fs::read(&path).map_err(|e| {
                Error::Other(format!("Failed to read file '{}': {}", path.display(), e))
            })?;
            files.push((relative, content));
        }
    }
    Ok(())
}

/// Configuration for GitHub Pages deployment
pub struct GhPagesConfig<'a> {
    pub branch: &'a str,
    pub data_dir: &'a str,
    pub remote: &'a str,
    pub skip_fetch: bool,
    pub dashboard_dir: Option<&'a Path>,
}

impl Default for GhPagesConfig<'_> {
    fn default() -> Self {
        Self {
            branch: "gh-pages",
            data_dir: "dev/bench",
            remote: "origin",
            skip_fetch: false,
            dashboard_dir: None,
        }
    }
}

/// Fetch existing benchmark data from gh-pages branch.
/// Returns empty data if the branch or file doesn't exist.
pub fn fetch_data_from_gh_pages(
    repo_path: &Path,
    branch: &str,
    data_dir: &str,
    remote: &str,
) -> BenchmarkData {
    // Try to fetch the remote branch first
    let _ = fetch_remote(repo_path, remote, branch);

    // Try to read data.json using git show
    let file_path = format!("{}/data.json", data_dir);
    let ref_spec = format!("{}/{}", remote, branch);

    let output = std::process::Command::new("git")
        .args(["show", &format!("{}:{}", ref_spec, file_path)])
        .current_dir(repo_path)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let content = String::from_utf8_lossy(&output.stdout);
            match serde_json::from_str::<BenchmarkData>(&content) {
                Ok(data) => {
                    tracing::info!(
                        "Loaded existing benchmark data from {}: {} entries",
                        ref_spec,
                        data.entries.values().map(|v| v.len()).sum::<usize>()
                    );
                    data
                }
                Err(e) => {
                    tracing::warn!("Failed to parse existing data.json: {}", e);
                    BenchmarkData::new()
                }
            }
        }
        _ => {
            tracing::info!(
                "No existing data.json found on {} (this is normal for first run)",
                ref_spec
            );
            BenchmarkData::new()
        }
    }
}

/// Deploy benchmark data to GitHub Pages branch.
/// The new_run is merged with existing data.json on gh-pages to preserve history.
pub fn deploy_to_gh_pages(
    repo_path: &Path,
    new_run: &git_bench_core::BenchmarkRun,
    suite_name: &str,
    max_items: Option<usize>,
    config: &GhPagesConfig,
) -> Result<String> {
    let repo = Repository::open(repo_path)?;

    let original_ref = repo
        .head()?
        .name()
        .ok_or_else(|| Error::Git(git2::Error::from_str("Cannot get current branch")))?
        .to_string();

    // Collect dashboard files BEFORE checking out gh-pages (they won't exist after checkout)
    let dashboard_files: Option<Vec<(std::path::PathBuf, Vec<u8>)>> =
        if let Some(dashboard_src) = config.dashboard_dir {
            if dashboard_src.exists() {
                Some(collect_dir_files(dashboard_src)?)
            } else {
                None
            }
        } else {
            None
        };

    // Check for uncommitted changes
    let statuses = repo.statuses(None)?;
    let has_changes = statuses.iter().any(|s| {
        s.status().intersects(
            git2::Status::WT_MODIFIED
                | git2::Status::WT_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_NEW,
        )
    });

    if has_changes {
        std::process::Command::new("git")
            .args(["stash", "push", "-m", "git-bench: temporary stash"])
            .current_dir(repo_path)
            .output()
            .map_err(|e| Error::Other(format!("Failed to stash changes: {}", e)))?;
    }

    // Fetch gh-pages branch
    if !config.skip_fetch {
        let _ = fetch_remote(repo_path, config.remote, config.branch);
    }

    // Check if branch exists
    let branch_exists_locally = repo.find_branch(config.branch, BranchType::Local).is_ok();
    let branch_exists_remote = repo
        .find_branch(
            &format!("{}/{}", config.remote, config.branch),
            BranchType::Remote,
        )
        .is_ok();

    if branch_exists_locally || branch_exists_remote {
        checkout_branch(repo_path, config.branch)?;
    } else {
        create_orphan_branch(repo_path, config.branch)?;
    }

    // Create data directory
    let data_dir = repo_path.join(config.data_dir);
    std::fs::create_dir_all(&data_dir).map_err(|e| Error::FileWrite {
        path: data_dir.display().to_string(),
        source: e,
    })?;

    // Write dashboard files from memory
    if let Some(files) = dashboard_files {
        for (relative_path, content) in files {
            let dest_path = data_dir.join(&relative_path);
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| Error::FileWrite {
                    path: parent.display().to_string(),
                    source: e,
                })?;
            }
            std::fs::write(&dest_path, content).map_err(|e| Error::FileWrite {
                path: dest_path.display().to_string(),
                source: e,
            })?;
        }
    }

    // Load existing data.json from gh-pages (if it exists) to preserve history
    let dest_file = data_dir.join("data.json");
    let mut data = if dest_file.exists() {
        git_bench_core::BenchmarkData::load_from_file(&dest_file).unwrap_or_else(|e| {
            tracing::warn!("Failed to load existing data.json, starting fresh: {}", e);
            git_bench_core::BenchmarkData::new()
        })
    } else {
        git_bench_core::BenchmarkData::new()
    };

    // Add the new run to the existing data (this preserves history)
    data.add_run(suite_name, new_run.clone(), max_items);

    // Write merged data
    let data_content = serde_json::to_string_pretty(&data)
        .map_err(|e| Error::Other(format!("Failed to serialize benchmark data: {}", e)))?;
    std::fs::write(&dest_file, data_content).map_err(|e| Error::FileWrite {
        path: dest_file.display().to_string(),
        source: e,
    })?;

    // Stage all files in data directory
    let mut index = repo.index()?;
    index.add_all([config.data_dir], git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;

    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let has_staged_changes = if let Ok(head) = repo.head() {
        if let Ok(parent) = head.peel_to_commit() {
            let parent_tree = parent.tree()?;
            let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)?;
            diff.deltas().count() > 0
        } else {
            true
        }
    } else {
        true
    };

    let commit_id = if has_staged_changes {
        let sig = repo.signature().unwrap_or_else(|_| {
            git2::Signature::now("git-bench", "git-bench@users.noreply.github.com").unwrap()
        });

        let parent_commit = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        let parents: Vec<&git2::Commit> = parent_commit.iter().collect();

        let commit_id = repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Update benchmark data [git-bench]",
            &tree,
            &parents,
        )?;

        Some(commit_id.to_string())
    } else {
        None
    };

    // Push
    push_to_remote_with_auth(repo_path, config.remote, config.branch)?;

    // Return to original branch
    let obj = repo.revparse_single(&original_ref)?;
    repo.checkout_tree(&obj, None)?;
    repo.set_head(&original_ref)?;

    // Restore stash
    if has_changes {
        std::process::Command::new("git")
            .args(["stash", "pop"])
            .current_dir(repo_path)
            .output()
            .ok();
    }

    Ok(commit_id.unwrap_or_else(|| "No changes".to_string()))
}

fn create_orphan_branch(repo_path: &Path, branch_name: &str) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["checkout", "--orphan", branch_name])
        .current_dir(repo_path)
        .output()
        .map_err(|e| Error::Other(format!("Failed to create orphan branch: {}", e)))?;

    if !output.status.success() {
        return Err(Error::Other(format!(
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

fn push_to_remote_with_auth(repo_path: &Path, remote_name: &str, branch: &str) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["push", remote_name, branch, "--force"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| Error::Other(format!("Failed to push: {}", e)))?;

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
}
