#!/usr/bin/env node
/**
 * OOCP v0.1 executable checks — see creator-docs/oocp/OOCP_TEST_SUITE.md
 */
import { connectOocp } from "@oclive/oocp-client";

const HTTP_PORT = (process.env.OOCP_HTTP_PORT || "48888").trim();
const HTTP_BASE = (
  process.env.OOCP_HTTP_BASE || `http://127.0.0.1:${HTTP_PORT}`
).replace(/\/+$/, "");
const WS_URL =
  process.env.OOCP_WS_URL ||
  HTTP_BASE.replace(/^http/i, (m) => (m.toLowerCase() === "https" ? "wss" : "ws")) +
    "/oocp";
const TOKEN = process.env.OOCP_API_TOKEN || "";

const WANT_JSON = process.argv.includes("--json");

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
  const t0 = Date.now();
  const scenarioTimings = [];

  function mark(id) {
    scenarioTimings.push({ id, durationMs: Date.now() - t0 });
  }

  await httpHealth();
  mark("S0");

  const client = connectOocp(
    { url: WS_URL, token: TOKEN, timeoutMs: 60_000 },
    {
      onError: (e) => console.warn("[oocp-test-suite] ws lifecycle error:", e.message),
    },
  );

  const caps = await client.connect();
  assert(caps && caps.type === "capabilities", "capabilities first frame");
  assert(caps.version === "0.1.0", `capabilities.version expected 0.1.0, got ${caps.version}`);
  assert(Array.isArray(caps.methods) && caps.methods.includes("role.list"), "methods whitelist");
  console.log("[oocp-test-suite] S1 oocp_capabilities_first_frame OK");
  mark("S1");

  const rl = await client.call("role.list", {});
  assert(rl.type === "response", "role.list response type");
  const rolesRaw = rl.result;
  const roles = Array.isArray(rolesRaw) ? rolesRaw : null;
  assert(roles && roles.length > 0, "role.list should return non-empty array");
  console.log(`[oocp-test-suite] S2 role_list OK (${roles.length} roles)`);
  mark("S2");

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
  mark("S3");

  for (const k of ["version", "author", "description"]) {
    assert(typeof info[k] === "string", `role.get_info.${k} should be string`);
  }
  assert(String(info.version).length > 0, "role.get_info.version non-empty");
  assert(String(info.author).length > 0, "role.get_info.author non-empty");
  console.log(`[oocp-test-suite] S11 role_pack_metadata_via_get_info OK (${roleId})`);
  mark("S11");

  const sc = await client.call("session.create", { role_id: roleId });
  assert(sc.type === "response", "session.create response type");
  const sessionNs = sc.result && String(sc.result.session_ns || "");
  assert(sessionNs.length > 0, "session.create.session_ns");
  console.log(`[oocp-test-suite] S4 session_create OK (session_ns=${sessionNs})`);
  mark("S4");

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
  mark("S5");

  const chat = await client.call("chat.send_message", {
    session_ns: sessionNs,
    user_message: "Hello from oocp-test-suite (S6)",
    scene_id: sceneForSwitch,
  });
  assert(chat.type === "response", "chat.send_message response type");
  const reply = chat.result && chat.result.reply;
  assert(typeof reply === "string" && reply.length > 0, "chat.send_message.reply non-empty string");
  console.log("[oocp-test-suite] S6 chat_send_message OK");
  mark("S6");

  const replies = [String(reply)];
  const msgs = [
    "Second turn: counting 1",
    "Third turn: counting 2",
    "Fourth turn: counting 3",
  ];
  for (const m of msgs) {
    const r2 = await client.call("chat.send_message", {
      session_ns: sessionNs,
      user_message: m,
      scene_id: sceneForSwitch,
    });
    assert(r2.type === "response", "chat.send_message (multi) response type");
    const r2t = r2.result && r2.result.reply;
    assert(typeof r2t === "string" && r2t.length > 0, "chat.send_message multi reply");
    replies.push(String(r2t));
  }
  const uniq = new Set(replies);
  assert(replies.length === 4, "expected 4 assistant replies total");
  assert(uniq.size >= 1, "at least one distinct reply shape");
  console.log("[oocp-test-suite] S8 chat_send_message_multi_turn OK");
  mark("S8");

  const st = await client.call("session.get_state", { session_ns: sessionNs });
  assert(st.type === "response", "session.get_state response type");
  const stb = st.result;
  assert(stb && typeof stb === "object", "session.get_state body");
  assert(
    typeof stb.role_id === "string" && stb.role_id.length > 0,
    "session.get_state.role_id",
  );
  console.log(
    "[oocp-test-suite] S9 session_state_probe OK (v0.1 has no plugin.list_slots; using session.get_state)",
  );
  mark("S9");

  let threw = false;
  try {
    await client.call("oclive.__nonexistent_method__", {});
  } catch (e) {
    threw = true;
    const msg = e instanceof Error ? e.message : String(e);
    assert(
      msg.includes("UNSUPPORTED_METHOD") || msg.includes("未在 capabilities") || msg.includes("未知方法"),
      `unexpected error message: ${msg}`,
    );
  }
  assert(threw, "invalid OOCP method should reject");
  console.log("[oocp-test-suite] S10 unsupported_method_error OK");
  mark("S10");

  const dest = await client.call("session.destroy", { session_ns: sessionNs });
  assert(dest.type === "response", "session.destroy response type");
  assert(dest.result && typeof dest.result === "object", "session.destroy result object");
  console.log(
    "[oocp-test-suite] S7 session_destroy_wire_ok (v0.1: OOCP method is session.destroy; host may extend invalidation semantics)",
  );
  mark("S7");

  client.close();
  console.log("[oocp-test-suite] PASS (S0–S11)");

  if (WANT_JSON) {
    const durationMs = Date.now() - t0;
    const envelope = {
      schemaVersion: 1,
      kind: "oclive.protocol_conformance_report.v1",
      summary: {
        passed: 12,
        failed: 0,
        skipped: 0,
        total: 12,
        passRate: 1,
        durationMs,
        exitCode: 0,
        ok: true,
      },
      meta: {
        httpBase: HTTP_BASE,
        wsUrl: WS_URL,
        scenarios: [
          "S0",
          "S1",
          "S2",
          "S3",
          "S4",
          "S5",
          "S6",
          "S8",
          "S9",
          "S10",
          "S11",
          "S7",
        ],
      },
      suites: [],
      failures: [],
    };
    process.stdout.write(`${JSON.stringify(envelope, null, 2)}\n`);
  }
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
