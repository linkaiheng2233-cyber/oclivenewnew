use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Current JSON manifest schema understood by this reader.
pub const SCAFFOLD_SCHEMA_VERSION: u32 = 1;
/// Semantic version of the contract represented by schema v1.
pub const SCAFFOLD_CONTRACT_VERSION: &str = "1.0.0";
pub const SCAFFOLD_CONFIG_SCHEMA_VERSION: u32 = 1;
pub const SCAFFOLD_LOCK_SCHEMA_VERSION: u32 = 1;
pub const SCAFFOLD_MANIFEST_FILENAME: &str = "oclive.scaffold.json";
pub const SCAFFOLD_CONFIG_FILENAME: &str = "scaffold.config.json";
pub const SCAFFOLD_LOCK_FILENAME: &str = "scaffold.lock.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaffoldManifest {
    pub schema_version: u32,
    pub package: ScaffoldPackageIdentity,
    pub compatibility: ScaffoldCompatibility,
    pub command_namespace: String,
    #[serde(default)]
    pub generators: Vec<GeneratorDeclaration>,
    #[serde(default)]
    pub commands: Vec<CommandDeclaration>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub defaults: BTreeMap<String, Value>,
    #[serde(default)]
    pub dependencies: Vec<PackageReference>,
    #[serde(default)]
    pub extends: Vec<PackageReference>,
    #[serde(default)]
    pub composition: CompositionDeclaration,
    #[serde(default)]
    pub extensions: BTreeMap<String, ExtensionEnvelope>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaffoldPackageIdentity {
    pub id: String,
    pub version: String,
    pub display_name: String,
    pub description: String,
    pub maintainer: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaffoldCompatibility {
    pub oclive_cli: String,
    pub scaffold_contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratorDeclaration {
    pub id: String,
    pub kind: String,
    pub driver: GeneratorDriver,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GeneratorDriver {
    /// Existing official CLI/script driver. Custom packages cannot claim this kind.
    Builtin { target: String },
    /// Local declarative instruction or generation-rule document.
    Instruction { path: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandDeclaration {
    pub name: String,
    pub description: String,
    pub entry: CommandEntry,
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CommandEntry {
    /// Existing official command. Custom packages cannot claim this kind.
    Builtin { target: String },
    /// A local script declaration. Stage 2A records but never executes it.
    Script {
        path: String,
        runtime: ScriptRuntime,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptRuntime {
    Node,
    Python,
    Native,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageReference {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompositionDeclaration {
    #[serde(default)]
    pub order_before: Vec<String>,
    #[serde(default)]
    pub order_after: Vec<String>,
    #[serde(default)]
    pub conflict_groups: Vec<String>,
}

impl CompositionDeclaration {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.order_before.is_empty()
            && self.order_after.is_empty()
            && self.conflict_groups.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionEnvelope {
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub config: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaffoldSource {
    Project,
    User,
    Official,
}

impl ScaffoldSource {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::User => "user",
            Self::Official => "official",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaffoldTrust {
    Official,
    UntrustedLocal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaffoldConfig {
    pub schema_version: u32,
    #[serde(default)]
    pub source_order: Option<Vec<ScaffoldSource>>,
    #[serde(default)]
    pub package_sources: BTreeMap<String, ScaffoldSource>,
    #[serde(default)]
    pub package_enabled: BTreeMap<String, bool>,
}

impl Default for ScaffoldConfig {
    fn default() -> Self {
        Self::empty()
    }
}

impl ScaffoldConfig {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            schema_version: SCAFFOLD_CONFIG_SCHEMA_VERSION,
            source_order: None,
            package_sources: BTreeMap::new(),
            package_enabled: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationIssue {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestValidation {
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
}

impl ManifestValidation {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogCandidate {
    pub source: ScaffoldSource,
    pub locator: String,
    pub manifest_sha256: String,
    pub trust: ScaffoldTrust,
    pub manifest: ScaffoldManifest,
    #[serde(default)]
    pub warnings: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogIssue {
    pub source: ScaffoldSource,
    pub locator: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogScan {
    pub candidates: Vec<CatalogCandidate>,
    pub issues: Vec<CatalogIssue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedPackage {
    pub source: ScaffoldSource,
    pub locator: String,
    pub manifest_sha256: String,
    pub trust: ScaffoldTrust,
    pub manifest: ScaffoldManifest,
    #[serde(default)]
    pub warnings: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShadowedPackage {
    pub id: String,
    pub version: String,
    pub source: ScaffoldSource,
    pub locator: String,
    pub selected_source: ScaffoldSource,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedCatalog {
    pub schema_version: u32,
    pub reader_version: String,
    pub source_order: Vec<ScaffoldSource>,
    pub packages: Vec<ResolvedPackage>,
    pub shadowed: Vec<ShadowedPackage>,
    #[serde(default)]
    pub warnings: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaffoldLock {
    pub schema_version: u32,
    pub scaffold_contract: String,
    pub reader_version: String,
    pub source_order: Vec<ScaffoldSource>,
    pub packages: Vec<ScaffoldLockPackage>,
    #[serde(default)]
    pub warnings: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaffoldLockPackage {
    pub id: String,
    pub version: String,
    pub source: ScaffoldSource,
    pub locator: String,
    pub manifest_sha256: String,
    pub maintainer: String,
    pub trust: ScaffoldTrust,
    pub command_namespace: String,
    pub permissions: Vec<String>,
    #[serde(default)]
    pub unresolved_dependencies: Vec<PackageReference>,
    #[serde(default)]
    pub unresolved_extends: Vec<PackageReference>,
    pub composition_declared: bool,
}
