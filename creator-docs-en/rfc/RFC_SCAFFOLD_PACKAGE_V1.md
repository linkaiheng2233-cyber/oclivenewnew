# RFC: OCLive Scaffold Package v1

[中文](../../creator-docs/rfc/RFC_SCAFFOLD_PACKAGE_V1.md)

> **Status (2026-08-01):** the Stage 2A discovery contract and Stage 2B bounded declarative-generation contract are frozen. CI and scaffolding remain independent. This document is the SSOT for package discovery, source locking, command namespaces, generation transactions, trust notices, and compatibility. CI impact planning remains owned by [`SOMEDAY_TOOLCHAIN_CI.md`](../roadmap/SOMEDAY_TOOLCHAIN_CI.md).

## Boundary

A Scaffold Package is a local developer instruction and generation-declaration package. It may describe instructions, generators, defaults, and namespaced commands. It does not prove that generated output is correct.

No package may define or override `oclive ci`, workflows, validator coordinates, runners, secrets, caches, concurrency, timeouts, gate strength, the impact algorithm, or job-skipping policy. CI always re-analyzes generated files. Stage 2B only materializes the local declarative file contract below: it does not execute third-party scripts or proxy built-in commands and has no marketplace, network installation, or composition runtime.

## Audit and command surface

The 2026-08-01 audit found 25 top-level commands, including 10 commands that require `--experimental` but still appear beside stable commands. It also found three conflicting uses of “template”: five `init --template` kernel recipes, legacy `.oclive-template.tar.gz` archive commands, and the new Scaffold Package concept.

Stable visible commands after Stage 2A are `init`, `dev`, `pack`, `doctor`, `plugin`, `registry`, `lint`, `profile`, `config`, `ci`, `scaffold`, `kernel`, `explain`, `migrate-app-data`, and `completions`.

Experimental commands remain callable with `--experimental` but are hidden from default help: `build`, `bench`, `blueprint`, `compose`, `debug`, `dashboard`, `learn`, `test`, `market`, and `collab`. The legacy `template` command remains callable but hidden and only manages project archives. `init`, `plugin create`, `pack create`, and `scripts/scaffold-ui-slot-plugin.mjs` remain domain generators; Stage 2A only registers their official declarations.

## Discovery and configuration

The manifest name is `oclive.scaffold.json`. Sources are project (`<project>/.oclive/scaffolds/*/`), user (`<OCLIVE_HOME>/scaffolds/*/`), and compiled official packages. Default precedence is `project > user > official`. User configuration lives at `<OCLIVE_HOME>/scaffold.config.json`; project configuration at `<project>/.oclive/scaffold.config.json` overrides matching settings.

Duplicate IDs in one source, a missing explicitly selected source, malformed manifests, path traversal, and symlink escape are hard failures. A higher-precedence package shadows the same ID in lower sources without deleting the official fallback.

## Manifest, compatibility, and namespace

V1 carries package identity and independent SemVer, compatibility ranges, a command namespace, generator declarations, command declarations and requested permissions, defaults, reserved `dependencies` / `extends` / `composition`, and a namespaced extension envelope.

Third parties cannot use `com.oclive.*` or request `ci.*` capabilities. Commands are declarations only in Stage 2A. A declaration never grants permission.

New readers support older v1 packages within their compatibility range. Readers must reject a newer unsupported `schema_version` with an upgrade or migration message instead of guessing backward compatibility. Reserved composition fields are preserved and diagnosed but not resolved or executed.

## Lock and trust record

`<project>/.oclive/scaffold.lock.json` deterministically records the effective source order, reader version, selected ID/version/source/relative locator/maintainer/manifest SHA-256, `official` versus `untrusted_local` trust, requested permissions, namespaces, and unresolved composition declarations. Writing requires `--write-lock` and uses atomic replacement.

The lock is neither authorization nor CI evidence. Project and user packages always display source, maintainer, scope, requested permissions, third-party maintenance responsibility, and the fact that packages cannot control CI.

## Stage 2A CLI

`oclive scaffold` provides `list`, `inspect`, `validate`, and `resolve`; only `resolve --write-lock` mutates disk. Stage 2A does not provide install, update, market, run, network, or composition execution and does not replace the existing domain generators.

## Stage 2B bounded declarative generation

Stage 2B adds `oclive scaffold generate <package-id> <generator-id> --output <new-directory>`, subject to these hard boundaries:

1. A local `instruction` generator must pin its package-relative instruction document with a 64-character lowercase `sha256`. Older v1 packages without the digest remain discoverable but cannot generate; migration raises the compatibility range to `>=1.1,<2` and adds the digest.
2. The strict instruction document uses `schema_version: 1` and declares only string variables plus file mappings. Every source pins its SHA-256. `text` supports exact `{{variable}}` replacement and `copy` performs byte-for-byte copying. There are no conditions, loops, expressions, includes, shells, networks, or lifecycle hooks.
3. Variable precedence is `--set key=value`, then string values in manifest `defaults`, then instruction defaults. Unknown variables, non-string manifest defaults used by the instruction, missing required variables, and unknown placeholders fail closed. Provenance records variable names, never values.
4. The package must declare `project.write`. Project and user packages additionally require an exact current `.oclive/scaffold.lock.json` match for ID, version, source, locator, and manifest digest, plus per-invocation `--accept-untrusted`. This acknowledgement only grants the bounded write; it never grants declared process, network, environment, or user-config capabilities.
5. The package root, instruction, and every source must canonicalize inside the selected package root. Symlink escapes, absolute or parent paths, duplicate targets, and file/directory conflicts are rejected.
6. The output must not exist and its parent must already exist. Validation, digest checks, variable resolution, and in-memory rendering finish before a temporary tree is written beside the destination and atomically renamed once. Failures leave no partial destination. There is no `--force` and no existing-project mutation in Stage 2B.
7. `--dry-run` performs the same validation and render planning without writes. Success writes `.oclive/scaffold.provenance.json` with package source, maintainer, manifest/instruction digests, generator ID, variable names, and each output path/digest; it contains no timestamp or variable value.
8. Official `builtin` drivers continue to delegate to existing domain commands such as `init`, `plugin create`, and `pack create`; `scaffold generate` only prints the precise delegation guidance and does not duplicate those generators.

Stage 2B still provides no `add/remove/update/install/run`, never executes `commands[].entry`, never resolves `dependencies` / `extends` / `composition`, has no network access, and grants no CI control. Generated output still needs ordinary CI or local validation; provenance is not quality evidence.
