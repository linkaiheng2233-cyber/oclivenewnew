/**
 * Split infrastructure/db.rs into db/*.rs submodules.
 */
import fs from 'node:fs'
import path from 'node:path'

const root = path.resolve(import.meta.dirname, '..')
const srcPath = path.join(root, 'src-tauri/src/infrastructure/db.rs')
const dbDir = path.join(root, 'src-tauri/src/infrastructure/db')

const source = fs.readFileSync(srcPath, 'utf8')
const testIdx = source.indexOf('#[cfg(test)]')
const main = testIdx >= 0 ? source.slice(0, testIdx) : source
const tests = testIdx >= 0 ? source.slice(testIdx) : ''

const implIdx = main.indexOf('impl DbManager {')
if (implIdx < 0) throw new Error('impl DbManager not found')
const header = main.slice(0, implIdx)
const implBody = main.slice(implIdx + 'impl DbManager {'.length)

const fnGroups = {
  mod: new Set([
    'health_ping', 'save_memory_and_event_atomic', 'apply_chat_turn_atomic',
    'delete_all_data_for_manifest_role', 'role_runtime_exists',
  ]),
  long_term_memory: new Set([
    'save_memory', 'load_memories', 'count_memories', 'load_memories_paged',
    'get_latest_memory_created_at', 'delete_memory', 'delete_memory_for_role',
  ]),
  role_runtime: new Set([
    'ensure_role_runtime', 'save_favorability', 'get_favorability',
    'favorability_for_identity_with_runtime_fallback', 'apply_favorability_delta',
    'get_current_emotion', 'set_current_emotion', 'get_relation_state',
    'get_current_scene', 'set_current_scene', 'get_user_presence_scene',
    'set_user_presence_scene', 'get_virtual_time_ms', 'set_virtual_time_ms',
    'ensure_interaction_mode_seeded', 'get_interaction_mode', 'set_interaction_mode_for_role',
    'get_remote_life_enabled', 'set_remote_life_enabled', 'get_use_manifest_default',
    'set_use_manifest_default', 'get_event_impact_factor', 'set_event_impact_factor',
    'get_core_delta_personality_json', 'set_core_delta_personality_json',
    'get_mutable_personality', 'set_mutable_personality',
    'save_personality_vector', 'get_latest_personality_vector',
    'save_event', 'count_events', 'list_events_paged', 'insert_manual_event', 'get_events',
  ]),
  relation_state: new Set([
    'get_user_relation', 'set_user_relation', 'get_user_relation_for_scene',
    'set_user_relation_for_scene', 'clear_user_relation_for_scene',
    'clear_all_scene_identities_for_role', 'get_favorability_for_identity',
    'get_relation_state_for_identity', 'ensure_identity_stats_row',
    'set_identity_favorability_value', 'mirror_runtime_from_identity',
  ]),
  session_state: new Set([
    'list_short_term_recent_turns', 'list_short_term_turns', 'list_conversation_sessions',
  ]),
  plugin_state: new Set(['upsert_app_setting', 'get_app_setting']),
}

function splitImplMethods(body) {
  const methods = []
  let i = 0
  while (i < body.length) {
    const fnMatch = body.slice(i).match(/\n    pub async fn (\w+)/)
    if (!fnMatch) break
    const start = i + fnMatch.index + 1
    const name = fnMatch[1]
  let j = start
  let depth = 0
  let started = false
  for (; j < body.length; j += 1) {
    const c = body[j]
    if (c === '{') {
      depth += 1
      started = true
    }
    else if (c === '}') {
      depth -= 1
      if (started && depth === 0) {
        j += 1
        break
      }
    }
  }
    methods.push({ name, text: body.slice(start, j).trimEnd() })
    i = j
  }
  return methods
}

const methods = splitImplMethods(implBody)
const buckets = Object.fromEntries(Object.keys(fnGroups).map(k => [k, []]))
let unassigned = []
for (const m of methods) {
  let placed = false
  for (const [bucket, set] of Object.entries(fnGroups)) {
    if (set.has(m.name)) {
      buckets[bucket].push(m.text)
      placed = true
      break
    }
  }
  if (!placed) unassigned.push(m.name)
}
if (unassigned.length) {
  console.warn('unassigned methods -> mod:', unassigned)
  for (const m of methods.filter(x => unassigned.includes(x.name))) {
    buckets.mod.push(m.text)
  }
}

const submodulePreamble = `use super::{DbManager, log_txn_finish, parse_memory_created_at, ChatTurnTxInput, EventListRow, SHORT_TERM_FIFO_LIMIT, TX_ERROR_MS, TX_WARN_MS};
use crate::error::{AppError, Result};
use crate::models::*;
use chrono::{DateTime, Utc};
use sqlx::Row;
use std::time::Instant;

impl DbManager {
`

const headerFixed = header.trimEnd() + `

pub(crate) fn log_txn_finish(tx_name: &str, role_id: &str, elapsed_ms: u128) {
    if elapsed_ms >= TX_ERROR_MS {
        tracing::error!(
            "tx slow code=TXN_SLOW_CRITICAL tx_name={} role_id={} elapsed_ms={}",
            tx_name,
            role_id,
            elapsed_ms
        );
    } else if elapsed_ms >= TX_WARN_MS {
        tracing::warn!(
            "tx slow code=TXN_SLOW_WARN tx_name={} role_id={} elapsed_ms={}",
            tx_name,
            role_id,
            elapsed_ms
        );
    } else {
        tracing::info!(
            "tx finish tx_name={} role_id={} elapsed_ms={}",
            tx_name,
            role_id,
            elapsed_ms
        );
    }
}
`

fs.mkdirSync(dbDir, { recursive: true })
const submods = []
for (const [name, parts] of Object.entries(buckets)) {
  if (name === 'mod' || parts.length === 0) continue
  submods.push(name)
  const content = `//! \`${name.replace('_', ' ')}\` 相关 [\`DbManager\`](super::DbManager) 方法。\n\n${submodulePreamble}${parts.join('\n\n')}\n}\n`
  fs.writeFileSync(path.join(dbDir, `${name}.rs`), content, 'utf8')
  console.log(name, parts.length, 'methods')
}

const modImpl = buckets.mod.join('\n\n')
const modRs = `${headerFixed}

impl DbManager {
${modImpl}
}

${submods.map(n => `mod ${n};`).join('\n')}

${tests}`
fs.writeFileSync(path.join(dbDir, 'mod.rs'), modRs, 'utf8')
fs.unlinkSync(srcPath)
console.log('done')
