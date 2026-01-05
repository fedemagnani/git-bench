//! File system operations for deployment

use crate::error::{Error, Result};
use std::path::{Path, PathBuf};

/// Recursively collect all files from a directory into memory
/// Returns Vec of (relative_path, content)
pub fn collect_dir_files(src: &Path) -> Result<Vec<(PathBuf, Vec<u8>)>> {
    let mut files = Vec::new();
    collect_dir_files_recursive(src, src, &mut files)?;
    Ok(files)
}

fn collect_dir_files_recursive(
    base: &Path,
    current: &Path,
    files: &mut Vec<(PathBuf, Vec<u8>)>,
) -> Result<()> {
    for entry in std::fs::read_dir(current)
        .map_err(|e| Error::Io(format!("Failed to read dir '{}': {}", current.display(), e)))?
    {
        let entry = entry.map_err(|e| Error::Io(format!("Failed to read entry: {}", e)))?;
        let path = entry.path();

        if path.is_dir() {
            collect_dir_files_recursive(base, &path, files)?;
        } else {
            let relative = path.strip_prefix(base).unwrap_or(&path).to_path_buf();
            let content = std::fs::read(&path).map_err(|e| {
                Error::Io(format!("Failed to read file '{}': {}", path.display(), e))
            })?;
            files.push((relative, content));
        }
    }
    Ok(())
}

/// Write collected files to destination directory
pub fn write_files_to_dir(files: &[(PathBuf, Vec<u8>)], dest_dir: &Path) -> Result<()> {
    for (relative_path, content) in files {
        let dest_path = dest_dir.join(relative_path);
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
    Ok(())
}

/// Create directory if it doesn't exist
pub fn ensure_dir_exists(dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir).map_err(|e| Error::FileWrite {
        path: dir.display().to_string(),
        source: e,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_collect_and_write_files() {
        let src_dir = TempDir::new().unwrap();
        let dest_dir = TempDir::new().unwrap();

        // Create test files
        fs::write(src_dir.path().join("file1.txt"), "content1").unwrap();
        fs::create_dir_all(src_dir.path().join("subdir")).unwrap();
        fs::write(src_dir.path().join("subdir/file2.txt"), "content2").unwrap();

        // Collect files
        let files = collect_dir_files(src_dir.path()).unwrap();
        assert_eq!(files.len(), 2);

        // Write files to destination
        write_files_to_dir(&files, dest_dir.path()).unwrap();

        // Verify files were written correctly
        assert_eq!(
            fs::read_to_string(dest_dir.path().join("file1.txt")).unwrap(),
            "content1"
        );
        assert_eq!(
            fs::read_to_string(dest_dir.path().join("subdir/file2.txt")).unwrap(),
            "content2"
        );
    }
}
