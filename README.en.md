# A.I.Live (OCLive)

**A composable AI character runtime: role packs describe the character; a Host connects the capabilities.**

[简体中文](README.md) · [Five-minute introduction](human-docs/START_HERE.md) · [User guide](creator-docs-en/getting-started/USER_MANUAL.md) · [Public test releases](https://github.com/linkaiheng2233-cyber/oclivenewnew/releases)

[![CI](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/workflows/ci.yml/badge.svg)](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/workflows/ci.yml)
[![Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

A.I.Live lets creators write personas, portraits, scenes and identity templates, and lets users run characters locally. Developers can also select the contracts and implementations they need to build their own Host.

**The project is in development and testing.** Public installers, default-branch source and the new frontend candidate are separate snapshots. A source screenshot does not mean an older installer includes that interface. Recent Chat Pro work is in [candidate PR #225](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/225); consult the [project status](creator-docs-en/getting-started/PROJECT_CURRENT_STATUS.md) and each release's actual assets for versions and acceptance boundaries.

## Choose an entry point

| Goal | Start here |
|---|---|
| Run a character, import a role pack and chat | [User guide](creator-docs-en/getting-started/USER_MANUAL.md) · [Public test releases](https://github.com/linkaiheng2233-cyber/oclivenewnew/releases) |
| Create or convert a role pack | [Role-pack editor](https://github.com/linkaiheng2233-cyber/oclive-pack-editor) · [Creator path](creator-docs-en/getting-started/CREATOR_GOLDEN_PATH.md) |
| Use the small kernel, implement a module or build a Host | [Independent OCLive Kernel](https://github.com/linkaiheng2233-cyber/oclive-kernel) · [Source SDK / Memory recipe](examples/developer-sdk-memory/README.md) |
| Write a plugin or extend the UI | [Plugin author path](creator-docs-en/plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md) |
| Contribute to this repository | [Developer entry](human-docs-en/README.md) · [Contributing](CONTRIBUTING.md) |

## Characters, capabilities and runtime

- **Role packs** carry persona content and assets. The editor supports creation and supported import/conversion formats. Mumu is an example character.
- **Local first**: the reference runtime stores conversations and character state on the user's machine. A configured local model or a selected remote backend supplies inference; software and external model/service terms are separate.
- **Replaceable capabilities**: Memory, Prompt, LLM and other capabilities connect through contracts. The Host selects implementations and the call sequence. The six capability categories are not six mandatory steps.
- **Chat Pro** is the desktop reference Host in this repository. Other entry points, including VS Code, have their own supported scope.

![Preview of the candidate Chat Pro character workspace](creator-docs/assets/chat-pro-chat.jpg)

*Candidate frontend preview; actual UI and behavior depend on the version being used.*

## Developers: choose an integration layer

The [independent kernel](https://github.com/linkaiheng2233-cyber/oclive-kernel) contains six Rust Base contracts and a default-on, optional character profile. Its [source prerelease](https://github.com/linkaiheng2233-cyber/oclive-kernel/releases/tag/v0.1.0-preview.1) is available for developers starting from a small interface surface.

This repository also contains a fuller **reference runtime**: concrete implementations, state application, persistence, permissions and plugin wiring, HTTP/Tauri adapters and distribution UIs. Reusing it does not make its complete orchestration mandatory for every Host.

Responsibilities and implementation mappings are maintained in the [module and boundary document](handoff/MODULE_MAP_AND_HANDOFF.md). Follow the [integration path](creator-docs-en/getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md) and [compatibility notes](creator-docs-en/COMPATIBILITY.md) for the actual integration contract.

## Start the desktop reference app from source

Prepare Node.js 22, Rust and platform desktop build dependencies using the [developer entry](human-docs-en/README.md), then run:

```bash
git clone https://github.com/linkaiheng2233-cyber/oclivenewnew.git
cd oclivenewnew
npm ci
npm run tauri:dev
```

Before chatting, prepare a working local model backend or configure a remote backend as described in the [user guide](creator-docs-en/getting-started/USER_MANUAL.md). A source checkout or lightweight installer does not imply that model files are included.

## Related projects

| Project | Purpose |
|---|---|
| [oclive-kernel](https://github.com/linkaiheng2233-cyber/oclive-kernel) | Independent Rust Base library and example Hosts |
| [oclive-pack-editor](https://github.com/linkaiheng2233-cyber/oclive-pack-editor) | Role-pack creation, conversion, validation and export |
| [oclive-vscode](https://github.com/linkaiheng2233-cyber/oclive-vscode) | VS Code character-chat Host; diaries and letters are optional plugins |
| [Role directory](https://github.com/linkaiheng2233-cyber/awesome-oclive-roles) · [Plugin directory](https://github.com/linkaiheng2233-cyber/awesome-oclive-plugins) | Discover entries and sources; compatibility belongs to the publisher and selected Host |
| [Physical character-Host project](https://github.com/linkaiheng2233-cyber/oclive-skippy-plan) | P0 hardware research for device events and small-screen character feedback, not a finished device |

The historical launcher and paused community-site prototype are separate projects, not required Chat Pro components. See the [documentation index](creator-docs-en/getting-started/DOCUMENTATION_INDEX.md) for further material.

## Feedback, contributions and license

This is a continuing hobby project. Suggestions are welcome through [Issues](https://github.com/linkaiheng2233-cyber/oclivenewnew/issues) or **ocliveproject@163.com**. Interesting ideas may be adopted; implementation is not guaranteed.

See [CONTRIBUTING](CONTRIBUTING.md) for code contributions and [AGENTS](AGENTS.md) for task-specific AI reading paths. Report security concerns privately using [SECURITY](SECURITY.md).

The main repository uses [Apache-2.0](LICENSE) with [NOTICE](NOTICE). Models, third-party plugins and role assets retain their own source terms; see the [license policy](creator-docs-en/LICENSE_POLICY.md).


<!-- Preserve previously published section bookmarks. -->
<a id="ailive--pluggable-role-artery-loom"></a>
<a id="start-here"></a>
<a id="what-is-this"></a>
<a id="four-quick-examples-30-seconds"></a>
<a id="example-1--creator-a-talkable-oc"></a>
<a id="example-2--developer-swap-llm-only"></a>
<a id="example-3--integrator-one-pack-many-hosts"></a>
<a id="example-4--module-author-new-capability-only"></a>
<a id="how-is-this-different"></a>
<a id="three-distros-one-kernel--different-hostprofile"></a>
<a id="architecture-one-paragraph-version"></a>
<a id="30-minute-contributor-path"></a>
<a id="ecosystem-sister-repos"></a>
<a id="ai--agent-onboarding"></a>
<a id="support--license"></a>

Previous section bookmarks are preserved here. The [earlier homepage](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/fb361ff5b878c633ef95dc66d717105d5b5074c0/README.en.md) remains available as historical documentation; use this page and its linked sources for current entry points.
