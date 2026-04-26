#!/usr/bin/env npx tsx
// OOCP WebSocket Smoke Test
// 用法: npx tsx tools/oocp-client/examples/smoke.ts [url]
//   默认 ws://127.0.0.1:48888/oocp
//
// 验收标准:
//   1. 连接成功 → 打印 capabilities.version
//   2. role.list 返回成功或明确的错误码（core 未启动/角色目录为空都算可观测）
//   3. 未连接/超时 → 给出清晰错误提示

import { connectOocp, OocpCapabilities, OocpResponse } from "../src/index.js";

const URL = process.argv[2] || "ws://127.0.0.1:48888/oocp";

async function main() {
  console.log(`\n[smoke] OOCP smoke test → ${URL}`);
  console.log(
    "[smoke] 确保先启动 core：在项目根运行 `npm run oocp:serve` (另开终端)",
  );
  console.log("");

  let caps: OocpCapabilities;

  try {
    caps = await connect();
  } catch (e: any) {
    console.error(
      `\n[FAIL] 无法连接到 OOCP 服务端: ${e.message}`,
    );
    console.error("[smoke] 请确认 core 已启动: npm run oocp:serve");
    process.exit(1);
  }

  console.log(`[PASS] capabilities.version = ${caps.version}`);
  console.log(
    `[PASS] methods (${caps.methods.length}): ${caps.methods.join(", ")}`,
  );
  console.log(`[PASS] auth_required = ${caps.auth_required}`);
  console.log(
    `[PASS] limits.max_message_chars = ${caps.limits.max_message_chars}`,
  );
  console.log("");

  // 尝试 role.list（可能返回空列表，只要不 panic 就行）
  try {
    const resp = await call("role.list");
    printResult("role.list", resp);
  } catch (e: any) {
    console.log(
      `[INFO] role.list 出错（可能角色目录为空或 core 不支持）: ${e.message}`,
    );
  }

  // 尝试 session.create（需要有效的 role 信息，不要求成功）
  try {
    const resp = await call("session.create", {});
    printResult("session.create", resp);
  } catch (e: any) {
    console.log(
      `[INFO] session.create 出错（预期可能无有效 role）: ${e.message}`,
    );
  }

  console.log("");
  console.log("[smoke] smoke test 完成（以上错误如为预期内则不影响验收）");
  process.exit(0);
}

async function connect(): Promise<OocpCapabilities> {
  const client = connectOocp(
    { url: URL, timeoutMs: 5000 },
    {
      onConnected: () => {
        console.log("[smoke] WS connected, waiting for capabilities...");
      },
      onError: (e) => {
        console.error(`[smoke] connection error: ${e.message}`);
      },
    },
  );

  const caps = await client.connect();
  client.close();
  return caps;
}

async function call(
  method: string,
  params: Record<string, unknown>,
): Promise<OocpResponse> {
  // 每次调用新建连接，模拟 smoke 场景
  const client = connectOocp({ url: URL, timeoutMs: 5000 });
  const caps = await client.connect();
  console.log(`[smoke] (reconnected, version=${caps.version})`);
  try {
    const resp = await client.call(method, params);
    client.close();
    return resp;
  } catch (e) {
    client.close();
    throw e;
  }
}

function printResult(method: string, resp: OocpResponse) {
  const result = resp.result;
  if (result && result.reply) {
    console.log(`[PASS] ${method} → reply: "${result.reply}"`);
  } else if (Array.isArray(result)) {
    console.log(`[PASS] ${method} → ${result.length} item(s)`);
  } else {
    console.log(`[PASS] ${method} → ${JSON.stringify(result)}`);
  }
}

main();