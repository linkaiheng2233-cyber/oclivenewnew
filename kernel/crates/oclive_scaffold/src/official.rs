/// Compiled official fallback manifest.
#[derive(Debug, Clone, Copy)]
pub struct OfficialManifestDocument {
    pub locator: &'static str,
    pub json: &'static str,
}

/// Official packages describe existing generators; Stage 2A does not execute them.
#[must_use]
pub const fn official_manifest_documents() -> &'static [OfficialManifestDocument] {
    &[
        OfficialManifestDocument {
            locator: "builtin://com.oclive.scaffold.kernel",
            json: include_str!("../official/kernel.oclive.scaffold.json"),
        },
        OfficialManifestDocument {
            locator: "builtin://com.oclive.scaffold.plugin",
            json: include_str!("../official/plugin.oclive.scaffold.json"),
        },
        OfficialManifestDocument {
            locator: "builtin://com.oclive.scaffold.role-pack",
            json: include_str!("../official/role-pack.oclive.scaffold.json"),
        },
        OfficialManifestDocument {
            locator: "builtin://com.oclive.scaffold.project-archive",
            json: include_str!("../official/project-archive.oclive.scaffold.json"),
        },
    ]
}
