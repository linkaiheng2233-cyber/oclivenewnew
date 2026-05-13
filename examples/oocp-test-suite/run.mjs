#!/usr/bin/env node
/**
 * OOCP v0.1 executable checks — see creator-docs/oocp/OOCP_TEST_SUITE.md
 */
import { connectOocp } from "@oclive/oocp-client";

const HTTP_BASE = (process.env.OOCP_HTTP_BASE || "http://127.0.0.1:48888").replace(
  /\/+$/,
  "",
);
const WS_URL =
  process.env.OOCP_WS_URL ||
  HTTP_BASE.replace(/^http/i, (m) => (m.toLowerCase() === "https" ? "wss" : "ws")) +
    "/oocp";
const TOKEN = process.env.OOCP_API_TOKEN || "";

function fail(msg) {
  console.error(`[oocp-test-suite] FAIL: ${msg}`);
  process.exit(1);
}

function assert(cond, msg) {
  if (!cond) fail(msg);
}

async function httpHealth() {
  const url = `${HTTP_BASE}/health`;
  const res = await fetch(url, { method: "GET" });
  assert(res.ok, `GET /health HTTP ${res.status} @ ${url}`);
  const text = await res.text();
  assert(text.includes("ok"), `GET /health body should contain "ok", got: ${text.slice(0, 200)}`);
  console.log("[oocp-test-suite] S0 http_health_plain OK");
}

async function main() {
  await httpHealth();

  const client = connectOocp(
    { url: WS_URL, token: TOKEN, timeoutMs: 30_000 },
    {
      onError: (e) => console.warn("[oocp-test-suite] ws lifecycle error:", e.message),
    },
  );

  const caps = await client.connect();
  assert(caps && caps.type === "capabilities", "capabilities first frame");
  assert(caps.version === "0.1.0", `capabilities.version expected 0.1.0, got ${caps.version}`);
  assert(Array.isArray(caps.methods) && caps.methods.includes("role.list"), "methods whitelist");
  console.log("[oocp-test-suite] S1 oocp_capabilities_first_frame OK");

  const rl = await client.call("role.list", {});
  assert(rl.type === "response", "role.list response type");
  const rolesRaw = rl.result;
  const roles = Array.isArray(rolesRaw) ? rolesRaw : null;
  assert(roles && roles.length > 0, "role.list should return non-empty array");
  console.log(`[oocp-test-suite] S2 role_list OK (${roles.length} roles)`);

  let roleId = (process.env.OOCP_TEST_ROLE_ID || "").trim();
  if (!roleId) {
    const preferred = roles.find((r) => r.role_id === "mumu" || r.id === "mumu");
    const pick = preferred || roles[0];
    roleId = String(pick.role_id || pick.id || pick.manifestId || "");
  }
  assert(roleId.length > 0, "could not pick role_id");

  const gi = await client.call("role.get_info", { role_id: roleId });
  assert(gi.type === "response", "role.get_info response type");
  const info = gi.result;
  assert(info && typeof info === "object", "role.get_info result object");
  assert(info.role_id === roleId, `role.get_info.role_id (${info.role_id}) vs ${roleId}`);
  const scenes = Array.isArray(info.scenes) ? info.scenes : [];
  assert(scenes.length > 0, "role.get_info.scenes non-empty");
  console.log(`[oocp-test-suite] S3 role_get_info OK (role=${roleId})`);

  const sc = await client.call("session.create", { role_id: roleId });
  assert(sc.type === "response", "session.create response type");
  const sessionNs = sc.result && String(sc.result.session_ns || "");
  assert(sessionNs.length > 0, "session.create.session_ns");
  console.log(`[oocp-test-suite] S4 session_create OK (session_ns=${sessionNs})`);

  const sceneForSwitch = scenes.includes("home") ? "home" : scenes[0];
  const sw = await client.call("session.switch_scene", {
    session_ns: sessionNs,
    scene_id: sceneForSwitch,
  });
  assert(sw.type === "response", "session.switch_scene response type");
  assert(
    sw.result && String(sw.result.scene_id) === sceneForSwitch,
    `session.switch_scene.scene_id (${sw.result?.scene_id})`,
  );
  console.log(`[oocp-test-suite] S5 session_switch_scene OK (scene=${sceneForSwitch})`);

  const chat = await client.call("chat.send_message", {
    session_ns: sessionNs,
    user_message: "Hello from oocp-test-suite",
    scene_id: sceneForSwitch,
  });
  assert(chat.type === "response", "chat.send_message response type");
  const reply = chat.result && chat.result.reply;
  assert(typeof reply === "string" && reply.length > 0, "chat.send_message.reply non-empty string");
  console.log("[oocp-test-suite] S6 chat_send_message OK");

  client.close();
  console.log("[oocp-test-suite] PASS");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
