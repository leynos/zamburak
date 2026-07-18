//! Loader plumbing shared by the YAML and JSON policy entrypoints.
//!
//! Probes the declared schema version, dispatches to the canonical or
//! legacy parser, and wraps results with migration audit evidence.

use serde::Deserialize;

use crate::load_outcome::LoadOutcome;
use crate::migration::{
    LEGACY_POLICY_SCHEMA_VERSION, PolicyDefinitionV0, audit_for_canonical_policy,
    migrate_schema_v0_to_v1,
};

use super::schema::PolicyLoadError;
use super::{CANONICAL_POLICY_SCHEMA_VERSION, PolicyDefinition, PolicyLoadOutcome, SchemaVersion};

pub(super) fn parse_canonical_yaml_policy(
    policy_yaml: &str,
) -> Result<PolicyDefinition, PolicyLoadError> {
    serde_yaml::from_str::<PolicyDefinition>(policy_yaml).map_err(PolicyLoadError::InvalidYaml)
}

pub(super) fn parse_canonical_json_policy(
    policy_json: &str,
) -> Result<PolicyDefinition, PolicyLoadError> {
    serde_json::from_str::<PolicyDefinition>(policy_json).map_err(PolicyLoadError::InvalidJson)
}

pub(super) struct MigrationLoadParsers<
    ParseError,
    VersionParser,
    CanonicalParser,
    LegacyParser,
    ErrorMapper,
> {
    pub(super) parse_schema_version: VersionParser,
    pub(super) parse_canonical_policy: CanonicalParser,
    pub(super) parse_legacy_policy: LegacyParser,
    pub(super) map_parse_error: ErrorMapper,
    pub(super) _phantom: std::marker::PhantomData<ParseError>,
}

pub(super) fn load_with_migration_audit<
    ParseError,
    VersionParser,
    CanonicalParser,
    LegacyParser,
    ErrorMapper,
>(
    serialized_policy: &str,
    parsers: MigrationLoadParsers<
        ParseError,
        VersionParser,
        CanonicalParser,
        LegacyParser,
        ErrorMapper,
    >,
) -> Result<PolicyLoadOutcome, PolicyLoadError>
where
    VersionParser: for<'a> Fn(&'a str) -> Result<SchemaVersionProbe, ParseError>,
    CanonicalParser: for<'a> Fn(&'a str) -> Result<PolicyDefinition, PolicyLoadError>,
    LegacyParser: for<'a> Fn(&'a str) -> Result<PolicyDefinitionV0, ParseError>,
    ErrorMapper: Fn(ParseError) -> PolicyLoadError + Copy,
{
    let version_probe =
        (parsers.parse_schema_version)(serialized_policy).map_err(parsers.map_parse_error)?;

    match version_probe.schema_version {
        Some(schema_version) if schema_version == CANONICAL_POLICY_SCHEMA_VERSION => {
            let policy_definition = (parsers.parse_canonical_policy)(serialized_policy)?;
            canonical_load_outcome(policy_definition)
        }
        Some(schema_version) if schema_version == LEGACY_POLICY_SCHEMA_VERSION => {
            let legacy_policy = (parsers.parse_legacy_policy)(serialized_policy)
                .map_err(parsers.map_parse_error)?;
            let migration_outcome = migrate_schema_v0_to_v1(legacy_policy)
                .map_err(PolicyLoadError::MigrationAuditFailed)?;
            let policy_definition = migration_outcome
                .policy_definition
                .ensure_canonical_schema_version()?;
            Ok(PolicyLoadOutcome {
                load_outcome: LoadOutcome::new(
                    policy_definition,
                    migration_outcome.migration_audit,
                ),
            })
        }
        Some(schema_version) => Err(PolicyLoadError::UnsupportedSchemaVersion {
            found: schema_version,
            expected: CANONICAL_POLICY_SCHEMA_VERSION,
        }),
        None => {
            let policy_definition = (parsers.parse_canonical_policy)(serialized_policy)?;
            canonical_load_outcome(policy_definition)
        }
    }
}

fn canonical_load_outcome(
    policy_definition: PolicyDefinition,
) -> Result<PolicyLoadOutcome, PolicyLoadError> {
    let policy_definition = policy_definition.ensure_canonical_schema_version()?;
    let migration_audit = audit_for_canonical_policy(&policy_definition)
        .map_err(PolicyLoadError::MigrationAuditFailed)?;
    Ok(PolicyLoadOutcome {
        load_outcome: LoadOutcome::new(policy_definition, migration_audit),
    })
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct SchemaVersionProbe {
    schema_version: Option<SchemaVersion>,
}
