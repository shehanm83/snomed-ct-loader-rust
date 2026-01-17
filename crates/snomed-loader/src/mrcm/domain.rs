//! MRCM Domain reference set parser.
//!
//! Parses files matching pattern: `der2_cRefset_MRCMDomainSnapshot_*.txt`

use std::path::Path;

use csv::StringRecord;
use snomed_types::MrcmDomain;

use crate::parser::{parse, Rf2Parser, Rf2Record};
use crate::types::{Rf2Config, Rf2Error, Rf2Result};

/// Expected columns for MRCM Domain reference set.
///
/// Order: id, effectiveTime, active, moduleId, refsetId, referencedComponentId,
/// domainConstraint, parentDomain, proximalPrimitiveConstraint,
/// proximalPrimitiveRefinement, domainTemplateForPrecoordination,
/// domainTemplateForPostcoordination, guideURL
const DOMAIN_COLUMNS: &[&str] = &[
    "id",
    "effectiveTime",
    "active",
    "moduleId",
    "refsetId",
    "referencedComponentId",
    "domainConstraint",
    "parentDomain",
    "proximalPrimitiveConstraint",
    "proximalPrimitiveRefinement",
    "domainTemplateForPrecoordination",
    "domainTemplateForPostcoordination",
    "guideURL",
];

impl Rf2Record for MrcmDomain {
    const EXPECTED_COLUMNS: &'static [&'static str] = DOMAIN_COLUMNS;

    fn from_record(record: &StringRecord) -> Rf2Result<Self> {
        let id = record
            .get(0)
            .ok_or_else(|| Rf2Error::MissingColumn {
                column: "id".to_string(),
            })?
            .to_string();

        let effective_time = parse::effective_time(record.get(1).ok_or_else(|| {
            Rf2Error::MissingColumn {
                column: "effectiveTime".to_string(),
            }
        })?)?;

        let active = parse::boolean(record.get(2).ok_or_else(|| Rf2Error::MissingColumn {
            column: "active".to_string(),
        })?)?;

        let module_id = parse::sctid(record.get(3).ok_or_else(|| Rf2Error::MissingColumn {
            column: "moduleId".to_string(),
        })?)?;

        let refset_id = parse::sctid(record.get(4).ok_or_else(|| Rf2Error::MissingColumn {
            column: "refsetId".to_string(),
        })?)?;

        let referenced_component_id =
            parse::sctid(record.get(5).ok_or_else(|| Rf2Error::MissingColumn {
                column: "referencedComponentId".to_string(),
            })?)?;

        let domain_constraint = record
            .get(6)
            .ok_or_else(|| Rf2Error::MissingColumn {
                column: "domainConstraint".to_string(),
            })?
            .to_string();

        let parent_domain_str = record.get(7).ok_or_else(|| Rf2Error::MissingColumn {
            column: "parentDomain".to_string(),
        })?;
        let parent_domain = if parent_domain_str.is_empty() {
            None
        } else {
            // Parent domain may include term in pipe notation: "71388002 |Procedure (procedure)|"
            Some(parse::sctid_with_term(parent_domain_str)?)
        };

        let proximal_primitive_constraint = record
            .get(8)
            .ok_or_else(|| Rf2Error::MissingColumn {
                column: "proximalPrimitiveConstraint".to_string(),
            })?
            .to_string();

        let proximal_primitive_refinement_str =
            record.get(9).ok_or_else(|| Rf2Error::MissingColumn {
                column: "proximalPrimitiveRefinement".to_string(),
            })?;
        let proximal_primitive_refinement = if proximal_primitive_refinement_str.is_empty() {
            None
        } else {
            Some(proximal_primitive_refinement_str.to_string())
        };

        let domain_template_for_precoordination = record
            .get(10)
            .ok_or_else(|| Rf2Error::MissingColumn {
                column: "domainTemplateForPrecoordination".to_string(),
            })?
            .to_string();

        let domain_template_for_postcoordination = record
            .get(11)
            .ok_or_else(|| Rf2Error::MissingColumn {
                column: "domainTemplateForPostcoordination".to_string(),
            })?
            .to_string();

        let guide_url_str = record.get(12).ok_or_else(|| Rf2Error::MissingColumn {
            column: "guideURL".to_string(),
        })?;
        let guide_url = if guide_url_str.is_empty() {
            None
        } else {
            Some(guide_url_str.to_string())
        };

        Ok(MrcmDomain {
            id,
            effective_time,
            active,
            module_id,
            refset_id,
            referenced_component_id,
            domain_constraint,
            parent_domain,
            proximal_primitive_constraint,
            proximal_primitive_refinement,
            domain_template_for_precoordination,
            domain_template_for_postcoordination,
            guide_url,
        })
    }

    fn passes_filter(&self, config: &Rf2Config) -> bool {
        if config.active_only && !self.active {
            return false;
        }
        true
    }
}

/// Parses MRCM Domain reference set from a file.
///
/// # Arguments
/// * `path` - Path to the MRCM Domain reference set file
/// * `config` - Parser configuration
///
/// # Returns
/// Iterator over parsed `MrcmDomain` records.
pub fn parse_domain_file<P: AsRef<Path>>(
    path: P,
    config: Rf2Config,
) -> Rf2Result<impl Iterator<Item = Rf2Result<MrcmDomain>>> {
    let parser = Rf2Parser::<_, MrcmDomain>::from_path(path, config)?;
    Ok(parser)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_record() -> StringRecord {
        let mut record = StringRecord::new();
        // id, effectiveTime, active, moduleId, refsetId, referencedComponentId
        record.push_field("550e8400-e29b-41d4-a716-446655440000");
        record.push_field("20240101");
        record.push_field("1");
        record.push_field("900000000000207008");
        record.push_field("723589008");
        record.push_field("404684003"); // Clinical finding
        // domainConstraint, parentDomain, proximalPrimitiveConstraint
        record.push_field("<< 404684003 |Clinical finding|");
        record.push_field(""); // No parent domain
        record.push_field("<< 404684003 |Clinical finding|");
        // proximalPrimitiveRefinement, domainTemplateForPrecoordination
        record.push_field(""); // No refinement
        record.push_field("[[+id(< 404684003 |Clinical finding|)]]");
        // domainTemplateForPostcoordination, guideURL
        record.push_field("[[+id(< 404684003 |Clinical finding|)]]");
        record.push_field(""); // No guide URL
        record
    }

    #[test]
    fn test_parse_mrcm_domain() {
        let record = make_test_record();
        let domain = MrcmDomain::from_record(&record).unwrap();

        assert_eq!(domain.id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(domain.effective_time, 20240101);
        assert!(domain.active);
        assert_eq!(domain.module_id, 900000000000207008);
        assert_eq!(domain.refset_id, 723589008);
        assert_eq!(domain.referenced_component_id, 404684003);
        assert_eq!(domain.domain_constraint, "<< 404684003 |Clinical finding|");
        assert!(domain.parent_domain.is_none());
        assert!(domain.proximal_primitive_refinement.is_none());
        assert!(domain.guide_url.is_none());
    }

    #[test]
    fn test_parse_with_parent_domain() {
        // Create record with parent domain set
        let record = StringRecord::from(vec![
            "550e8400-e29b-41d4-a716-446655440000",
            "20240101",
            "1",
            "900000000000207008",
            "723589008",
            "404684003",
            "<< 404684003",
            "138875005", // Parent domain set to root
            "<< 404684003",
            "",
            "template1",
            "template2",
            "",
        ]);

        let domain = MrcmDomain::from_record(&record).unwrap();
        assert_eq!(domain.parent_domain, Some(138875005));
    }

    #[test]
    fn test_filter_inactive() {
        // Create inactive record
        let record = StringRecord::from(vec![
            "550e8400-e29b-41d4-a716-446655440000",
            "20240101",
            "0", // Inactive
            "900000000000207008",
            "723589008",
            "404684003",
            "<< 404684003",
            "",
            "<< 404684003",
            "",
            "template1",
            "template2",
            "",
        ]);

        let domain = MrcmDomain::from_record(&record).unwrap();

        let config_active_only = Rf2Config {
            active_only: true,
            ..Default::default()
        };
        assert!(!domain.passes_filter(&config_active_only));

        let config_all = Rf2Config {
            active_only: false,
            ..Default::default()
        };
        assert!(domain.passes_filter(&config_all));
    }
}
