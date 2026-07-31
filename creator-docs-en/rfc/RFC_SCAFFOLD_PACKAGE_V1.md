# RFC: OCLive Scaffold Package v1

[中文](../../creator-docs/rfc/RFC_SCAFFOLD_PACKAGE_V1.md)

> **Status (2026-08-01):** Stage 2A contract frozen. CI and scaffolding remain independent. This document is the SSOT for package discovery, source locking, command namespaces, trust notices, and compatibility. CI impact planning remains owned by [`SOMEDAY_TOOLCHAIN_CI.md`](../roadmap/SOMEDAY_TOOLCHAIN_CI.md).

## Boundary

A Scaffold Package is a local developer instruction and generation-declaration package. It may describe instructions, generators, defaults, and namespaced commands. It does not prove that generated output is correct.

No package may define or override `oclive ci`, workflows, validator coordinates, runners, secrets, caches, concurrency, timeouts, gate strength, the impact algorithm, or job-skipping policy. CI always re-analyzes generated files. Stage 2A does not execute third-party commands and has no marketplace, network installation, or composition runtime.

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
