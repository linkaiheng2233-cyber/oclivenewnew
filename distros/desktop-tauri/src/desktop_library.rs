//! Seed installed defaults into a writable user library without replacing user packages.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn same_directory(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

fn copy_package(source: &Path, destination: &Path) -> io::Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let target = destination.join(entry.file_name());
        if kind.is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "bundled packages must not contain symbolic links",
            ));
        }
        if kind.is_dir() {
            fs::create_dir(&target)?;
            copy_package(&entry.path(), &target)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), target)?;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported file in bundled package",
            ));
        }
    }
    Ok(())
}

fn seed_missing_packages(source: &Path, destination: &Path) -> io::Result<()> {
    if !source.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        if entry.file_name().to_string_lossy().starts_with('.') || !entry.file_type()?.is_dir() {
            continue;
        }
        let target = destination.join(entry.file_name());
        match fs::symlink_metadata(&target) {
            Ok(_) => continue,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        // A failed or interrupted copy never leaves a half-written visible package.
        let staging = tempfile::Builder::new()
            .prefix(".seed-")
            .tempdir_in(destination)?;
        copy_package(&entry.path(), staging.path())?;
        if let Err(error) = fs::rename(staging.path(), &target) {
            // Another instance may have finished seeding this package first.
            if fs::symlink_metadata(&target).is_err() {
                return Err(error);
            }
        }
    }
    Ok(())
}

pub(crate) fn prepare_desktop_library(
    resource_dir: Option<&Path>,
    resolved_roles: &Path,
    app_data: &Path,
    explicit_roles_override: bool,
) -> io::Result<PathBuf> {
    if explicit_roles_override {
        return Ok(resolved_roles.to_path_buf());
    }
    let Some(resource_dir) = resource_dir else {
        return Ok(resolved_roles.to_path_buf());
    };
    let candidates = [
        resource_dir.join("roles"), // Packaged chat-pro resource layout.
        resource_dir.join("resources/roles"),
    ];
    let managed = app_data.join("roles");
    let source = candidates.iter().find(|candidate| {
        candidate.is_dir()
            && (same_directory(candidate, resolved_roles)
                || same_directory(&managed, resolved_roles))
    });
    let Some(source) = source else {
        // Repository/developer role roots remain editable in place.
        return Ok(resolved_roles.to_path_buf());
    };
    seed_missing_packages(source, &managed)?;
    if let Some(parent) = source.parent() {
        seed_missing_packages(&parent.join("plugins"), &app_data.join("plugins"))?;
    }
    Ok(managed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package(root: &Path, name: &str, content: &str) {
        let directory = root.join(name);
        fs::create_dir_all(directory.join("assets")).unwrap();
        fs::write(directory.join("manifest.json"), content).unwrap();
        fs::write(directory.join("assets/portrait.png"), b"fixture").unwrap();
    }

    #[test]
    fn installed_library_copies_defaults_but_preserves_user_packages_across_restarts() {
        let temporary = tempfile::tempdir().unwrap();
        let resource = temporary.path().join("install");
        let source = resource.join("resources/roles");
        let app_data = temporary.path().join("user-data");
        package(&source, "mumu", "bundled role");
        package(&source, "another-role", "another default");
        package(
            &resource.join("resources/plugins"),
            "example-plugin",
            "bundled plugin",
        );
        let managed = app_data.join("roles");
        package(&managed, "mumu", "user modified role");
        package(&managed, "user-only", "imported role");

        assert_eq!(
            prepare_desktop_library(Some(&resource), &source, &app_data, false).unwrap(),
            managed
        );
        assert_eq!(
            fs::read_to_string(managed.join("mumu/manifest.json")).unwrap(),
            "user modified role"
        );
        assert!(managed.join("user-only/manifest.json").is_file());
        assert!(managed.join("another-role/assets/portrait.png").is_file());
        assert!(app_data
            .join("plugins")
            .join("example-plugin/manifest.json")
            .is_file());
        assert_eq!(fs::read_dir(&source).unwrap().count(), 2);

        package(&source, "new-default", "added by a later installer");
        prepare_desktop_library(Some(&resource), &managed, &app_data, false).unwrap();
        assert!(managed.join("new-default/manifest.json").is_file());
        assert_eq!(
            fs::read_to_string(managed.join("mumu/manifest.json")).unwrap(),
            "user modified role"
        );
        assert!(fs::read_dir(managed).unwrap().all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".seed-")));
    }

    #[test]
    fn explicit_overrides_and_developer_roots_are_not_relocated() {
        let temporary = tempfile::tempdir().unwrap();
        let resource = temporary.path().join("install");
        let source = resource.join("roles"); // Packaged chat-pro resource layout.
        let app_data = temporary.path().join("user-data");
        let developer = temporary.path().join("developer-packs");
        package(&source, "mumu", "bundled role");
        package(&developer, "example", "working tree role");
        assert_eq!(
            prepare_desktop_library(Some(&resource), &source, &app_data, true).unwrap(),
            source
        );
        assert_eq!(
            prepare_desktop_library(Some(&resource), &developer, &app_data, false).unwrap(),
            developer
        );
        assert!(!app_data.exists());
    }

    #[test]
    fn role_pack_import_writes_to_the_managed_library_instead_of_the_installer() {
        let temporary = tempfile::tempdir().unwrap();
        let resource = temporary.path().join("install");
        let source = resource.join("resources/roles");
        let app_data = temporary.path().join("user-data");
        package(&source, "bundled-example", "bundled fixture");
        let managed = prepare_desktop_library(Some(&resource), &source, &app_data, false).unwrap();
        let storage = oclive_kernel_host::infrastructure::storage::RoleStorage::new(&managed);
        let official =
            oclive_kernel_runtime::chat_pro_roles_dir(&[PathBuf::from(env!("CARGO_MANIFEST_DIR"))])
                .unwrap();

        let imported = oclive_kernel_host::infrastructure::import_role_pack(
            &storage,
            &official.join("mumu"),
            false,
            |_| {},
        )
        .unwrap();

        assert_eq!(imported, "mumu");
        assert_eq!(storage.load_role("mumu").unwrap().id, "mumu");
        assert!(managed.join("mumu/pipeline.ocblueprint").is_file());
        assert!(!source.join("mumu").exists());
    }
}
