//! Generic RF2 file parser.
//!
//! Provides a streaming parser for SNOMED CT RF2 tab-delimited files.

use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::marker::PhantomData;
use std::path::Path;

use csv::{Reader, ReaderBuilder, StringRecord};
use snomed_types::SctId;

use crate::types::{Rf2Config, Rf2Error, Rf2Result};

/// Trait for types that can be parsed from RF2 records.
///
/// Implement this trait for custom RF2 record types.
pub trait Rf2Record: Sized {
    /// Expected column names for this record type.
    const EXPECTED_COLUMNS: &'static [&'static str];

    /// Parse a record from a CSV StringRecord.
    fn from_record(record: &StringRecord) -> Rf2Result<Self>;

    /// Returns true if this record passes the given filter config.
    fn passes_filter(&self, config: &Rf2Config) -> bool;
}

/// A streaming parser for RF2 files.
///
/// This parser reads RF2 files record-by-record to avoid loading
/// entire files into memory.
pub struct Rf2Parser<R: Read, T: Rf2Record> {
    reader: Reader<R>,
    config: Rf2Config,
    records_read: usize,
    _marker: PhantomData<T>,
}

impl<T: Rf2Record> Rf2Parser<BufReader<File>, T> {
    /// Creates a new parser from a file path.
    ///
    /// # Arguments
    /// * `path` - Path to the RF2 file
    /// * `config` - Parser configuration
    ///
    /// # Errors
    /// Returns an error if the file cannot be opened or has invalid headers.
    pub fn from_path<P: AsRef<Path>>(path: P, config: Rf2Config) -> Rf2Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(Rf2Error::FileNotFound {
                path: path.display().to_string(),
            });
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Self::from_reader(reader, config)
    }

    /// Counts the total number of lines in the file (for progress reporting).
    ///
    /// This performs a fast line count by counting newlines.
    pub fn count_lines<P: AsRef<Path>>(path: P) -> Rf2Result<usize> {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        let count = reader.lines().count();
        // Subtract 1 for header row
        Ok(count.saturating_sub(1))
    }
}

impl<R: Read, T: Rf2Record> Rf2Parser<R, T> {
    /// Creates a new parser from a reader.
    pub fn from_reader(reader: R, config: Rf2Config) -> Rf2Result<Self> {
        let mut csv_reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .flexible(false)
            .trim(csv::Trim::None)
            .from_reader(reader);

        // Validate headers
        Self::validate_headers(&mut csv_reader)?;

        Ok(Self {
            reader: csv_reader,
            config,
            records_read: 0,
            _marker: PhantomData,
        })
    }

    /// Validates that the file has the expected column headers.
    fn validate_headers(reader: &mut Reader<R>) -> Rf2Result<()> {
        let headers = reader.headers()?;
        let expected = T::EXPECTED_COLUMNS;

        if headers.len() < expected.len() {
            return Err(Rf2Error::InvalidHeader {
                expected: expected.len(),
                found: headers.len(),
            });
        }

        for (i, expected_col) in expected.iter().enumerate() {
            let found = headers.get(i).unwrap_or("");
            // Handle UTF-8 BOM at start of file
            let found = found.trim_start_matches('\u{feff}');
            if found != *expected_col {
                return Err(Rf2Error::UnexpectedColumn {
                    position: i,
                    expected: expected_col.to_string(),
                    found: found.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Returns the number of records read so far.
    pub fn records_read(&self) -> usize {
        self.records_read
    }

    /// Returns the parser configuration.
    pub fn config(&self) -> &Rf2Config {
        &self.config
    }

    /// Parses all records into a Vec, applying filters.
    ///
    /// Note: This loads all matching records into memory.
    pub fn parse_all(mut self) -> Rf2Result<Vec<T>> {
        let mut results = Vec::new();
        for record in self.by_ref().flatten() {
            results.push(record);
        }
        Ok(results)
    }

    /// Parses records in batches, calling the callback for each batch.
    ///
    /// This is useful for processing large files without loading
    /// everything into memory.
    pub fn parse_batched<F>(mut self, mut callback: F) -> Rf2Result<usize>
    where
        F: FnMut(Vec<T>) -> Rf2Result<()>,
    {
        let batch_size = self.config.batch_size;
        let mut batch = Vec::with_capacity(batch_size);
        let mut total = 0;

        for record in self.by_ref().flatten() {
            batch.push(record);
            if batch.len() >= batch_size {
                total += batch.len();
                callback(std::mem::take(&mut batch))?;
                batch = Vec::with_capacity(batch_size);
            }
        }

        // Process remaining records
        if !batch.is_empty() {
            total += batch.len();
            callback(batch)?;
        }

        Ok(total)
    }
}

impl<R: Read, T: Rf2Record> Iterator for Rf2Parser<R, T> {
    type Item = Rf2Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut record = StringRecord::new();
            match self.reader.read_record(&mut record) {
                Ok(true) => {
                    self.records_read += 1;

                    // Skip empty records
                    if record.is_empty() || record.iter().all(|f| f.trim().is_empty()) {
                        continue;
                    }

                    match T::from_record(&record) {
                        Ok(parsed) => {
                            if parsed.passes_filter(&self.config) {
                                return Some(Ok(parsed));
                            }
                            // Record filtered out, continue to next
                            continue;
                        }
                        Err(e) => return Some(Err(e)),
                    }
                }
                Ok(false) => return None, // End of file
                Err(e) => return Some(Err(e.into())),
            }
        }
    }
}

/// Helper functions for parsing RF2 field values.
pub mod parse {
    use super::{Rf2Error, Rf2Result, SctId};

    /// Parses an SCTID from a string.
    pub fn sctid(value: &str) -> Rf2Result<SctId> {
        value.parse::<u64>().map_err(|_| Rf2Error::InvalidSctId {
            value: value.to_string(),
        })
    }

    /// Parses an SCTID that may include term description in pipe notation.
    ///
    /// RF2 files sometimes include the term after the SCTID in pipes, like:
    /// `71388002 |Procedure (procedure)|`
    pub fn sctid_with_term(value: &str) -> Rf2Result<SctId> {
        let numeric_part = value.split_whitespace().next().unwrap_or("");

        if numeric_part.is_empty() {
            return Err(Rf2Error::InvalidSctId {
                value: value.to_string(),
            });
        }

        numeric_part
            .parse::<u64>()
            .map_err(|_| Rf2Error::InvalidSctId {
                value: value.to_string(),
            })
    }

    /// Parses a boolean from "0" or "1".
    pub fn boolean(value: &str) -> Rf2Result<bool> {
        match value {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(Rf2Error::InvalidBoolean {
                value: value.to_string(),
            }),
        }
    }

    /// Parses an effective time (YYYYMMDD) as u32.
    pub fn effective_time(value: &str) -> Rf2Result<u32> {
        if value.len() != 8 {
            return Err(Rf2Error::InvalidDate {
                value: value.to_string(),
            });
        }
        value.parse::<u32>().map_err(|_| Rf2Error::InvalidDate {
            value: value.to_string(),
        })
    }

    /// Parses an integer value.
    pub fn integer<T: std::str::FromStr>(value: &str) -> Rf2Result<T> {
        value.parse::<T>().map_err(|_| Rf2Error::InvalidInteger {
            value: value.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sctid() {
        assert_eq!(parse::sctid("404684003").unwrap(), 404684003u64);
        assert_eq!(
            parse::sctid("900000000000207008").unwrap(),
            900000000000207008u64
        );
        assert!(parse::sctid("not_a_number").is_err());
        assert!(parse::sctid("").is_err());
    }

    #[test]
    fn test_parse_boolean() {
        assert!(!parse::boolean("0").unwrap());
        assert!(parse::boolean("1").unwrap());
        assert!(parse::boolean("true").is_err());
        assert!(parse::boolean("2").is_err());
    }

    #[test]
    fn test_parse_effective_time() {
        assert_eq!(parse::effective_time("20020131").unwrap(), 20020131u32);
        assert_eq!(parse::effective_time("20251201").unwrap(), 20251201u32);
        assert!(parse::effective_time("2020-01-31").is_err());
        assert!(parse::effective_time("2002013").is_err());
    }

    #[test]
    fn test_parse_integer() {
        assert_eq!(parse::integer::<u16>("0").unwrap(), 0u16);
        assert_eq!(parse::integer::<u16>("123").unwrap(), 123u16);
        assert!(parse::integer::<u16>("abc").is_err());
    }
}
