//! RF2 file discovery and loading utilities.

use std::fs;
use std::path::{Path, PathBuf};

use crate::types::{Rf2Error, Rf2Files, Rf2Result};

/// Discovers RF2 files in a SNOMED CT release directory.
///
/// Searches for the Snapshot/Terminology directory and locates
/// concept, description, and relationship files. Also searches
/// for MRCM reference set files in Refset/Metadata.
pub fn discover_rf2_files<P: AsRef<Path>>(path: P) -> Rf2Result<Rf2Files> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(Rf2Error::DirectoryNotFound {
            path: path.display().to_string(),
        });
    }

    // Try to find the Terminology directory
    let terminology_dir = find_terminology_dir(path)?;

    let mut files = Rf2Files::new();

    // Scan for RF2 files in Terminology directory
    for entry in fs::read_dir(&terminology_dir)? {
        let entry = entry?;
        let filename = entry.file_name();
        let filename_str = filename.to_string_lossy();

        if !filename_str.ends_with(".txt") {
            continue;
        }

        if filename_str.starts_with("sct2_Concept_Snapshot") {
            files.concept_file = Some(entry.path());
            // Extract release date from filename
            if let Some(date) = extract_release_date(&filename_str) {
                files.release_date = Some(date);
            }
        } else if filename_str.starts_with("sct2_Description_Snapshot") {
            files.description_file = Some(entry.path());
        } else if filename_str.starts_with("sct2_Relationship_Snapshot") {
            files.relationship_file = Some(entry.path());
        } else if filename_str.starts_with("sct2_StatedRelationship_Snapshot") {
            files.stated_relationship_file = Some(entry.path());
        } else if filename_str.starts_with("sct2_TextDefinition_Snapshot") {
            files.text_definition_file = Some(entry.path());
        }
    }

    // Try to find MRCM files in Refset/Metadata directory
    if let Some(snapshot_dir) = terminology_dir.parent() {
        let metadata_dir = snapshot_dir.join("Refset").join("Metadata");
        if metadata_dir.exists() {
            discover_mrcm_files(&metadata_dir, &mut files)?;
        }
    }

    if !files.has_required_files() {
        let missing = files.missing_files();
        return Err(Rf2Error::RequiredFileMissing {
            file_type: missing.join(", "),
            directory: terminology_dir.display().to_string(),
        });
    }

    Ok(files)
}

/// Discovers MRCM reference set files in a Metadata directory.
fn discover_mrcm_files(metadata_dir: &Path, files: &mut Rf2Files) -> Rf2Result<()> {
    if !metadata_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(metadata_dir)? {
        let entry = entry?;
        let filename = entry.file_name();
        let filename_str = filename.to_string_lossy();

        if !filename_str.ends_with(".txt") {
            continue;
        }

        if filename_str.contains("MRCMDomainSnapshot") && !filename_str.contains("ModuleScope") {
            files.mrcm_domain = Some(entry.path());
        } else if filename_str.contains("MRCMAttributeDomainSnapshot") {
            files.mrcm_attribute_domain = Some(entry.path());
        } else if filename_str.contains("MRCMAttributeRangeSnapshot") {
            files.mrcm_attribute_range = Some(entry.path());
        }
    }

    Ok(())
}

/// Finds the Terminology directory within an RF2 release structure.
fn find_terminology_dir(base: &Path) -> Rf2Result<PathBuf> {
    // Check if base is already the Terminology directory
    if base.ends_with("Terminology") && base.is_dir() {
        return Ok(base.to_path_buf());
    }

    // Check for Snapshot/Terminology
    let snapshot_term = base.join("Snapshot").join("Terminology");
    if snapshot_term.exists() {
        return Ok(snapshot_term);
    }

    // Check for just Terminology
    let term = base.join("Terminology");
    if term.exists() {
        return Ok(term);
    }

    // Search one level deep for a directory containing the structure
    for entry in fs::read_dir(base)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let subdir = entry.path();

            // Check subdir/Snapshot/Terminology
            let sub_snapshot_term = subdir.join("Snapshot").join("Terminology");
            if sub_snapshot_term.exists() {
                return Ok(sub_snapshot_term);
            }

            // Check subdir/Terminology
            let sub_term = subdir.join("Terminology");
            if sub_term.exists() {
                return Ok(sub_term);
            }
        }
    }

    Err(Rf2Error::DirectoryNotFound {
        path: format!("Terminology directory not found in {}", base.display()),
    })
}

/// Extracts release date from RF2 filename.
///
/// RF2 files have names like `sct2_Concept_Snapshot_INT_20251201.txt`
fn extract_release_date(filename: &str) -> Option<String> {
    let without_ext = filename.trim_end_matches(".txt");
    let parts: Vec<&str> = without_ext.split('_').collect();

    if let Some(&last) = parts.last() {
        if last.len() == 8 && last.chars().all(|c| c.is_ascii_digit()) {
            return Some(last.to_string());
        }
    }

    None
}

/// Formats a byte count as a human-readable string.
pub fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    const GB: usize = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_release_date() {
        assert_eq!(
            extract_release_date("sct2_Concept_Snapshot_INT_20251201.txt"),
            Some("20251201".to_string())
        );
        assert_eq!(
            extract_release_date("sct2_Description_Snapshot-en_INT_20251201.txt"),
            Some("20251201".to_string())
        );
        assert_eq!(extract_release_date("invalid_filename.txt"), None);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 bytes");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }
}
