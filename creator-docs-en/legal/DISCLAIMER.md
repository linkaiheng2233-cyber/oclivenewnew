# Disclaimer (models, plugins, and data)

This document clarifies how the **oclive desktop host** positions **model weights**, **third-party plugins**, and **user data on disk**. It answers: **where is my data**, **who is responsible for plugin safety**, and **who must comply with model licenses**. This is not legal advice.

---

## 1. Models and inference services

- **oclive does not ship, host, or redistribute large model weights.** If you use local or cloud models, you must **obtain** the appropriate rights and comply with each model’s license (including commercial restrictions and attribution).  
- **Built-in integration** may target **Ollama** on the user’s machine by convention (env vars, ports, model names). **Ollama itself and any models you pull are not part of this repository**; their licensing and updates are governed by **Ollama** and **your local install**.  
- If you configure **Remote** LLMs, HTTP sidecars, or other cloud endpoints, **egress traffic, secrets, and upstream terms** are between **you and the operator** of those endpoints — see [SIDECAR_LLM_USER_GUIDE.md](../getting-started/SIDECAR_LLM_USER_GUIDE.md) and [LICENSE_POLICY.md](../LICENSE_POLICY.md).

---

## 2. Plugins and the open ecosystem

- **Marketplaces / indices / community feeds** surface an **open ecosystem**: each plugin remains the responsibility of **its author** for code, updates, and declared capabilities. **Maintainers do not warrant** that third-party plugins are defect-free, benign, or compatible with your environment.  
- **Before install**: review the plugin’s **`manifest.json`** (permissions, process, network, etc.).  
- **High-risk capabilities** (e.g. **`process:spawn`**, **`network:*`**, MCP transports) require **explicit host grants**; without grants features degrade with visible prompts — see [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md), [LICENSE_POLICY.md](../LICENSE_POLICY.md), and in-app **Agent debug / grants** UI.

---

## 3. Data storage and telemetry

- **Default local storage**: chat history, pack content, memory, and runtime state primarily live in **local** **SQLite** and **`{app_data}`** (see [CONFIGURATION_FILES.md](../guides/CONFIGURATION_FILES.md)). **Under default settings, the host does not upload that primary corpus to an oclive-operated cloud.**  
- **Exceptions**: if you configure a slot to use a **Remote** HTTP backend, or enable directory plugins / MCP that initiate outbound calls, data may leave the machine via **URLs you configure** — responsibility lies with **those endpoints**.  
- **Sentry**: when a DSN is baked into the build and the user has not opted out, **uncaught Vue errors** may be reported (defaults avoid personally identifying data; see README and Settings). **Users can disable** this in Settings.

---

## See also

- [LICENSE](../../LICENSE) · [LICENSE_POLICY.md](../LICENSE_POLICY.md)  
- [SECURITY.md](../../SECURITY.md) · [SECURITY_AUDIT_SCOPE.md](../security/SECURITY_AUDIT_SCOPE.md)

[中文](../../creator-docs/legal/DISCLAIMER.md)
