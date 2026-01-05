//! High-level deployment logic for GitHub Pages

use crate::error::{Error, Result};
use crate::file_manager::{collect_dir_files, ensure_dir_exists, write_files_to_dir};
use crate::git_ops::{
    checkout_branch, create_orphan_branch, fetch_remote, get_commit_info, has_uncommitted_changes,
    pop_stash, push_to_remote, stash_changes,
};
use git2::Repository;
use git_bench_core::{BenchmarkData, BenchmarkRun};
use std::path::{Path, PathBuf};

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
    new_run: &BenchmarkRun,
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
    let dashboard_files: Option<Vec<(PathBuf, Vec<u8>)>> =
        if let Some(dashboard_src) = config.dashboard_dir {
            if dashboard_src.exists() {
                Some(collect_dir_files(dashboard_src)?)
            } else {
                None
            }
        } else {
            None
        };

    // Check for uncommitted changes and stash if needed
    let has_changes = has_uncommitted_changes(&repo)?;
    if has_changes {
        stash_changes(repo_path, "git-bench: temporary stash")?;
    }

    // Fetch gh-pages branch
    if !config.skip_fetch {
        let _ = fetch_remote(repo_path, config.remote, config.branch);
    }

    // Check if branch exists and checkout or create
    let branch_exists_locally = repo
        .find_branch(config.branch, git2::BranchType::Local)
        .is_ok();
    let branch_exists_remote = repo
        .find_branch(
            &format!("{}/{}", config.remote, config.branch),
            git2::BranchType::Remote,
        )
        .is_ok();

    if branch_exists_locally || branch_exists_remote {
        checkout_branch(repo_path, config.branch)?;
    } else {
        create_orphan_branch(repo_path, config.branch)?;
    }

    // Create data directory
    let data_dir = repo_path.join(config.data_dir);
    ensure_dir_exists(&data_dir)?;

    // Write dashboard files from memory
    if let Some(files) = dashboard_files {
        write_files_to_dir(&files, &data_dir)?;
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

    // Push to remote
    push_to_remote(repo_path, config.remote, config.branch)?;

    // Return to original branch
    let obj = repo.revparse_single(&original_ref)?;
    repo.checkout_tree(&obj, None)?;
    repo.set_head(&original_ref)?;

    // Restore stash if we stashed changes
    if has_changes {
        let _ = pop_stash(repo_path);
    }

    Ok(commit_id.unwrap_or_else(|| "No changes".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git_ops::get_commit_info;
    use std::process::Command;
    use tempfile::TempDir;

    fn create_test_repo() -> TempDir {
        let dir = TempDir::new().unwrap();

        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .expect("Failed to init git repo");

        // Configure git user
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(dir.path())
            .output()
            .expect("Failed to configure git user");

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(dir.path())
            .output()
            .expect("Failed to configure git email");

        // Create initial commit
        std::fs::write(dir.path().join("test.txt"), "test content").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .expect("Failed to add files");

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(dir.path())
            .output()
            .expect("Failed to create initial commit");

        dir
    }

    #[test]
    fn test_gh_pages_config_default() {
        let config = GhPagesConfig::default();
        assert_eq!(config.branch, "gh-pages");
        assert_eq!(config.data_dir, "dev/bench");
        assert_eq!(config.remote, "origin");
        assert!(!config.skip_fetch);
        assert!(config.dashboard_dir.is_none());
    }

    #[test]
    fn test_get_commit_info_integration() {
        let dir = create_test_repo();
        let info = get_commit_info(dir.path(), None).unwrap();
        assert_eq!(info.message, "Initial commit");
        assert!(!info.id.is_empty());
    }
}
