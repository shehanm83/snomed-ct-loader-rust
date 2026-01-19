//! Parser-specific types for RF2 file processing.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during RF2 file parsing.
#[derive(Error, Debug)]
pub enum Rf2Error {
    /// I/O error reading RF2 file.
    #[error("IO error reading RF2 file: {0}")]
    Io(#[from] std::io::Error),

    /// CSV parsing error.
    #[error("CSV parsing error: {0}")]
    Csv(#[from] csv::Error),

    /// Invalid SCTID format.
    #[error("Invalid SCTID format: {value}")]
    InvalidSctId {
        /// The invalid value that was encountered.
        value: String,
    },

    /// Missing required column in RF2 file.
    #[error("Missing required column: {column}")]
    MissingColumn {
        /// The name of the missing column.
        column: String,
    },

    /// Invalid date format.
    #[error("Invalid date format: {value}")]
    InvalidDate {
        /// The invalid date value.
        value: String,
    },

    /// Invalid boolean value.
    #[error("Invalid boolean value: {value} (expected 0 or 1)")]
    InvalidBoolean {
        /// The invalid boolean value.
        value: String,
    },

    /// Invalid integer value.
    #[error("Invalid integer value: {value}")]
    InvalidInteger {
        /// The invalid integer value.
        value: String,
    },

    /// File not found.
    #[error("File not found: {path}")]
    FileNotFound {
        /// The path that was not found.
        path: String,
    },

    /// Directory not found.
    #[error("Directory not found: {path}")]
    DirectoryNotFound {
        /// The path that was not found.
        path: String,
    },

    /// Required file missing from RF2 directory.
    #[error("Required RF2 file not found: {file_type} in {directory}")]
    RequiredFileMissing {
        /// The type of file that was missing.
        file_type: String,
        /// The directory that was searched.
        directory: String,
    },

    /// Invalid header - column count mismatch.
    #[error("Invalid header: expected {expected} columns, found {found}")]
    InvalidHeader {
        /// Expected column count.
        expected: usize,
        /// Found column count.
        found: usize,
    },

    /// Unexpected column name.
    #[error("Unexpected column '{found}' at position {position}, expected '{expected}'")]
    UnexpectedColumn {
        /// The column position.
        position: usize,
        /// Expected column name.
        expected: String,
        /// Found column name.
        found: String,
    },

    /// Generic parse error.
    #[error("Parse error: {0}")]
    Parse(String),
}

/// Result type for RF2 operations.
pub type Rf2Result<T> = Result<T, Rf2Error>;

/// Configuration for RF2 parsing.
#[derive(Debug, Clone)]
pub struct Rf2Config {
    /// Whether to filter to active records only.
    pub active_only: bool,
    /// Batch size for processing (affects memory usage).
    pub batch_size: usize,
}

impl Default for Rf2Config {
    fn default() -> Self {
        Self {
            active_only: true,
            batch_size: 10_000,
        }
    }
}

/// Configuration specific to description parsing.
#[derive(Debug, Clone)]
pub struct DescriptionConfig {
    /// Base RF2 configuration.
    pub base: Rf2Config,
    /// Language codes to include (empty = all languages).
    pub language_codes: Vec<String>,
    /// Description type IDs to include (empty = all types).
    pub type_ids: Vec<u64>,
}

impl Default for DescriptionConfig {
    fn default() -> Self {
        Self {
            base: Rf2Config::default(),
            language_codes: vec!["en".to_string()],
            type_ids: vec![], // All types by default
        }
    }
}

impl DescriptionConfig {
    /// Creates a config that only includes English FSN and synonyms.
    pub fn english_terms() -> Self {
        Self {
            base: Rf2Config::default(),
            language_codes: vec!["en".to_string()],
            type_ids: vec![
                900000000000003001, // FSN
                900000000000013009, // Synonym
            ],
        }
    }

    /// Creates a config that only includes FSN descriptions.
    pub fn fsn_only() -> Self {
        Self {
            base: Rf2Config::default(),
            language_codes: vec!["en".to_string()],
            type_ids: vec![900000000000003001], // FSN only
        }
    }
}

/// Configuration specific to relationship parsing.
#[derive(Debug, Clone, Default)]
pub struct RelationshipConfig {
    /// Base RF2 configuration.
    pub base: Rf2Config,
    /// Relationship type IDs to include (empty = all types).
    pub type_ids: Vec<u64>,
    /// Characteristic type IDs to include (empty = all types).
    pub characteristic_type_ids: Vec<u64>,
}

impl RelationshipConfig {
    /// Creates a config for inferred relationships only.
    pub fn inferred_only() -> Self {
        Self {
            base: Rf2Config::default(),
            type_ids: vec![],
            characteristic_type_ids: vec![900000000000011006], // Inferred
        }
    }

    /// Creates a config for IS_A relationships only.
    pub fn is_a_only() -> Self {
        Self {
            base: Rf2Config::default(),
            type_ids: vec![116680003], // IS_A
            characteristic_type_ids: vec![],
        }
    }
}

/// Statistics from parsing an RF2 file.
#[derive(Debug, Clone, Default)]
pub struct ParseStats {
    /// Total records read from file.
    pub total_records: usize,
    /// Records that passed filters.
    pub filtered_records: usize,
    /// Records skipped (inactive, wrong language, etc.).
    pub skipped_records: usize,
    /// Parse errors encountered (non-fatal).
    pub error_count: usize,
    /// Time taken to parse in milliseconds.
    pub parse_time_ms: u64,
}

impl ParseStats {
    /// Returns the percentage of records that passed filters.
    pub fn filter_rate(&self) -> f64 {
        if self.total_records == 0 {
            0.0
        } else {
            (self.filtered_records as f64 / self.total_records as f64) * 100.0
        }
    }
}

/// Discovered RF2 files in a release directory.
#[derive(Debug, Clone, Default)]
pub struct Rf2Files {
    /// Path to concept file.
    pub concept_file: Option<PathBuf>,
    /// Path to description file.
    pub description_file: Option<PathBuf>,
    /// Path to relationship file.
    pub relationship_file: Option<PathBuf>,
    /// Path to stated relationship file (if separate).
    pub stated_relationship_file: Option<PathBuf>,
    /// Path to text definition file.
    pub text_definition_file: Option<PathBuf>,
    /// Path to MRCM Domain reference set file.
    pub mrcm_domain: Option<PathBuf>,
    /// Path to MRCM Attribute Domain reference set file.
    pub mrcm_attribute_domain: Option<PathBuf>,
    /// Path to MRCM Attribute Range reference set file.
    pub mrcm_attribute_range: Option<PathBuf>,
    /// Paths to simple reference set files.
    pub simple_refset_files: Vec<PathBuf>,
    /// Paths to language reference set files.
    pub language_refset_files: Vec<PathBuf>,
    /// Paths to association reference set files.
    pub association_refset_files: Vec<PathBuf>,
    /// Paths to OWL expression reference set files.
    pub owl_expression_files: Vec<PathBuf>,
    /// Path to concrete relationship file.
    pub concrete_relationship_file: Option<PathBuf>,
    /// Release date extracted from filename (YYYYMMDD).
    pub release_date: Option<String>,
}

impl Rf2Files {
    /// Creates a new empty Rf2Files.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if all required files (concept, description, relationship) are present.
    pub fn has_required_files(&self) -> bool {
        self.concept_file.is_some()
            && self.description_file.is_some()
            && self.relationship_file.is_some()
    }

    /// Returns a list of missing required files.
    pub fn missing_files(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.concept_file.is_none() {
            missing.push("Concept");
        }
        if self.description_file.is_none() {
            missing.push("Description");
        }
        if self.relationship_file.is_none() {
            missing.push("Relationship");
        }
        missing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rf2_config_default() {
        let config = Rf2Config::default();
        assert!(config.active_only);
        assert_eq!(config.batch_size, 10_000);
    }

    #[test]
    fn test_description_config_english_terms() {
        let config = DescriptionConfig::english_terms();
        assert_eq!(config.language_codes, vec!["en"]);
        assert_eq!(config.type_ids.len(), 2);
    }

    #[test]
    fn test_relationship_config_inferred_only() {
        let config = RelationshipConfig::inferred_only();
        assert!(config.type_ids.is_empty());
        assert_eq!(config.characteristic_type_ids, vec![900000000000011006]);
    }

    #[test]
    fn test_parse_stats_filter_rate() {
        let stats = ParseStats {
            total_records: 100,
            filtered_records: 75,
            ..Default::default()
        };
        assert!((stats.filter_rate() - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_rf2_files_missing() {
        let files = Rf2Files {
            concept_file: Some(PathBuf::from("concept.txt")),
            description_file: None,
            relationship_file: None,
            ..Default::default()
        };

        assert!(!files.has_required_files());
        let missing = files.missing_files();
        assert_eq!(missing.len(), 2);
        assert!(missing.contains(&"Description"));
        assert!(missing.contains(&"Relationship"));
    }
}
