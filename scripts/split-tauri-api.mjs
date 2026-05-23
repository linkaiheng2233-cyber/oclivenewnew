/**
 * One-shot splitter: src/utils/tauri-api.ts → src/api/*
 * Run: node scripts/split-tauri-api.mjs
 */
import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "..");
const srcPath = path.join(root, "src/utils/tauri-api.ts");
const apiDir = path.join(root, "src/api");
const src = fs.readFileSync(srcPath, "utf8");
const lines = src.split("\n");

const helpersHeader = `import { invoke } from '@tauri-apps/api/tauri'

import { i18n } from '../i18n/index'

`;

// lines 0-182 (1-indexed 1-183) are helpers through invokeWithFriendlyError
const helpersBody = lines.slice(0, 183).join("\n").replace(/^import.*\n\nimport.*\n\n/s, "");

const toCamelPayload = `
/** snake_case → camelCase for a single key (Tauri IPC top-level args). */
export function snakeToCamelKey(key: string): string {
  return key.replace(/_([a-z0-9])/g, (_, c) => c.toUpperCase())
}

/** Shallow-recursive camelCase key transform for invoke payloads. */
export function toCamelPayload(value: unknown): unknown {
  if (value === null || value === undefined)
    return value
  if (Array.isArray(value))
    return value.map(v => toCamelPayload(v))
  if (typeof value !== 'object')
    return value
  const out: Record<string, unknown> = {}
  for (const [k, v] of Object.entries(value as Record<string, unknown>)) {
    out[snakeToCamelKey(k)] = toCamelPayload(v)
  }
  return out
}
`;

const helpersImports = helpersHeader;
const helpersContent =
  helpersImports +
  helpersBody +
  toCamelPayload +
  "\n\nexport { invokeWithFriendlyError }\n";

const domainImports = `import { invoke } from '@tauri-apps/api/tauri'
import { invokeWithFriendlyError } from './helpers'

`;

// Remaining body starts at line 184 (index 184) - types and functions
const rest = lines.slice(184).join("\n");

// Split rest into blocks by export keyword at column 0
const blocks = [];
let current = [];
for (const line of rest.split("\n")) {
  if (line.startsWith("export ") && current.length > 0) {
    blocks.push(current.join("\n"));
    current = [line];
  } else {
    current.push(line);
  }
}
if (current.length) blocks.push(current.join("\n"));

const AGENT = new Set([
  "listMcpServers", "listMcpTools", "callMcpTool", "getAgentDebugTraces",
  "clearAgentDebugTraces", "listHighRiskGrants", "grantHighRiskCapability",
  "revokeHighRiskCapability",
]);
const AGENT_TYPES = /McpTool|McpServer|AgentDebug|AgentToolCall|HighRiskGrant/;

const DIAG = new Set([
  "runEnvironmentDiagnostics", "getRemoteFallbackAppSettings", "setRemoteFallbackToBuiltin",
]);
const DIAG_TYPES = /EnvironmentDiagnostics|RemoteFallbackAppSettings/;

const SETTINGS = new Set([
  "setSessionPluginBackend", "setSessionSlotOverride", "clearSessionSlotOverride",
  "clearAllSessionSlotOverrides", "saveRoleSlotRegistry", "applyAuthorSuggestedPluginBackends",
  "getPluginResolutionDebug", "getHotkeyBindings", "saveHotkeyBindings",
  "setRemoteFallbackToBuiltin",
]);
const SETTINGS_TYPES =
  /PluginBackends|PluginBackendSource|PluginResolutionDebug|DirectoryPluginSlots|Hotkey/;

const CHAT = new Set([
  "sendMessage", "queryMemories", "queryEvents", "createEvent", "switchScene",
  "setUserPresenceScene", "getTimeState", "jumpTime", "generateMonologue",
  "exportChatLogs", "reloadPolicyPlugins",
]);
const CHAT_TYPES =
  /SendMessage|EmotionDto|DetectedEvent|PresenceMode|QueryMemor|MemoryItem|QueryEvent|EventItem|CreateEvent|TimeState|JumpTime|ExportChatLogs|SwitchScene/;

const ROLE = new Set([
  "loadRole", "resolveRoleAssetPath", "getRoleInfo", "listRoles", "switchRole",
  "setUserRelation", "setEvolutionFactor", "setRemoteLifeEnabled", "setRoleInteractionMode",
  "setSceneUserRelation", "clearSceneUserRelation", "exportRolePack", "peekRolePack",
  "importRolePack",
]);
const ROLE_TYPES =
  /RoleData|RoleInfo|RolePack|UserRelation|LifeState|SceneLabel|PersonalitySource|PackUi|AuthorRecommended|AuthorPack|OCLIVE_DEFAULT|normalizePackUi|emptyPackUi/;

function blockDomain(block) {
  const fn = block.match(/^export async function (\w+)/)?.[1];
  if (fn) {
    if (AGENT.has(fn)) return "agent";
    if (DIAG.has(fn)) return "diagnostics";
    if (SETTINGS.has(fn)) return "settings";
    if (CHAT.has(fn)) return "chat";
    if (ROLE.has(fn)) return "role";
    return "plugin";
  }
  const iface = block.match(/^export interface (\w+)/)?.[1] ?? "";
  const type = block.match(/^export type (\w+)/)?.[1] ?? "";
  const name = iface || type;
  if (AGENT_TYPES.test(name)) return "agent";
  if (DIAG_TYPES.test(name)) return "diagnostics";
  if (SETTINGS_TYPES.test(name)) return "settings";
  if (CHAT_TYPES.test(name)) return "chat";
  if (ROLE_TYPES.test(name)) return "role";
  if (/PluginBridge/.test(name)) return "plugin";
  return "plugin";
}

const buckets = { agent: [], chat: [], role: [], settings: [], plugin: [], diagnostics: [] };
for (const b of blocks) {
  const d = blockDomain(b);
  buckets[d].push(b);
}

// directory bootstrap inflight lives in plugin block - ensure const at top of plugin
fs.mkdirSync(apiDir, { recursive: true });
fs.writeFileSync(path.join(apiDir, "helpers.ts"), helpersContent);

for (const [name, parts] of Object.entries(buckets)) {
  const content = domainImports + parts.join("\n\n") + "\n";
  fs.writeFileSync(path.join(apiDir, `${name}.ts`), content);
}

const index = `/** Domain-split Tauri invoke API (replaces utils/tauri-api.ts). */
export * from './helpers'
export * from './chat'
export * from './role'
export * from './settings'
export * from './plugin'
export * from './agent'
export * from './diagnostics'
`;
fs.writeFileSync(path.join(apiDir, "index.ts"), index);

console.log(
  "Wrote api modules:",
  Object.fromEntries(Object.entries(buckets).map(([k, v]) => [k, v.length])),
);
