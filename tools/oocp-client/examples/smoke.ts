#!/usr/bin/env npx tsx
// OOCP WebSocket Smoke Test
// 用法: npx tsx tools/oocp-client/examples/smoke.ts [url]
//   默认 ws://127.0.0.1:48888/oocp
//
// PASS/FAIL 规则:
//   连接成功且拿到 capabilities → PASS
//   连不上 WS / capabilities 超时 → FAIL
//   后续 role/session/chat 按环境选择性 PASS/INFO

import { connectOocp, OocpCapabilities, OocpClient } from "../src/index.js";

const URL = process.argv[2] || "ws://127.0.0.1:48888/oocp";

async function main() {
  console.log(`\n[smoke] OOCP smoke test → ${URL}`);
  console.log(
    "[smoke] 确保先启动 core：在项目根运行 `npm run oocp:kernel:serve` (另开终端)",
  );
  console.log("");

  let client: OocpClient;

  try {
    client = connectOocp(
      { url: URL, timeoutMs: 5000 },
      {
        onError: (e) => {
          console.error(`[smoke] connection error: ${e.message}`);
        },
      },
    );
    const caps = await client.connect();
    console.log(`[PASS] capabilities.version = ${caps.version}`);
    console.log(
      `[PASS] methods (${caps.methods.length}): ${caps.methods.join(", ")}`,
    );
    console.log(`[PASS] auth_required = ${caps.auth_required}`);
    console.log(
      `[PASS] limits.max_message_chars = ${caps.limits.max_message_chars}`,
    );
    console.log("");
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    console.error(`\n[FAIL] 无法连接到 OOCP 服务端: ${msg}`);
    console.error("[smoke] 请确认 core 已启动: npm run oocp:kernel:serve");
    process.exit(1);
  }

  // 尝试 role.list → session.create → chat.send_message
  try {
    const roleResp = await client.call("role.list", {});
    const result = roleResp.result;

    // 兼容多种返回结构
    let roles: unknown[] = [];
    if (Array.isArray(result)) {
      roles = result;
    } else if (
      result &&
      typeof result === "object" &&
      "roles" in result &&
      Array.isArray((result as Record<string, unknown>).roles)
    ) {
      roles = (result as Record<string, unknown>).roles as unknown[];
    }

    if (roles.length === 0) {
      console.log(
        "[INFO] no roles found, skipping chat.send_message (smoke still PASS)",
      );
      console.log(`[PASS] role.list → 0 items`);
    } else {
      console.log(`[PASS] role.list → ${roles.length} item(s)`);

      // 取第一个角色的 role_id
      const firstRole = roles[0] as Record<string, unknown>;
      const roleId: string =
        (firstRole.role_id as string) ||
        (firstRole.manifestId as string) ||
        (firstRole.id as string) ||
        "";

      if (!roleId) {
        console.log(
          "[INFO] role.list returned items but cannot extract role_id, " +
            "skipping session/chat. First item keys: " +
            Object.keys(firstRole).join(", "),
        );
      } else {
        console.log(`[PASS] role_id = ${roleId}`);

        // 创建会话
        const sessionResp = await client.call("session.create", {
          role_id: roleId,
        });
        const sessResult = sessionResp.result;
        let sessionNs = "";

        if (
          sessResult &&
          typeof sessResult === "object" &&
          "session_ns" in sessResult
        ) {
          sessionNs = String(
            (sessResult as Record<string, unknown>).session_ns,
          );
        } else if (
          sessResult &&
          typeof sessResult === "object" &&
          "id" in sessResult
        ) {
          sessionNs = String((sessResult as Record<string, unknown>).id);
        } else {
          sessionNs = JSON.stringify(sessResult);
        }

        console.log(`[PASS] session.create → session_ns = ${sessionNs}`);

        // 发送消息
        const chatResp = await client.call("chat.send_message", {
          session_ns: sessionNs,
          user_message: "hello",
        });
        const chatResult = chatResp.result;
        if (
          chatResult &&
          typeof chatResult === "object" &&
          "reply" in chatResult
        ) {
          console.log(
            `[PASS] chat.send_message → reply: "${String((chatResult as Record<string, unknown>).reply)}"`,
          );
        } else {
          console.log(
            `[PASS] chat.send_message → ${JSON.stringify(chatResult)}`,
          );
        }
      }
    }
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    console.log(
      `[INFO] role/session/chat 链出错（可选，核心连通性 PASS）: ${msg}`,
    );
  }

  client.close();

  console.log("");
  console.log("[smoke] smoke test PASS");
  process.exit(0);
}

main();