/**
 * Node 18+：通过 fetch 调用 /health 与 /chat（零 npm 依赖）。
 *
 *   node client.mjs --role-path "D:/oclivenewnew/roles/mumu"
 */
const args = process.argv.slice(2);

function getArg(name, def = undefined) {
  const i = args.indexOf(name);
  if (i === -1) return def;
  return args[i + 1] ?? def;
}

const baseUrl = (getArg("--base-url", "http://127.0.0.1:48888") ?? "").replace(/\/$/, "");
const rolePath = getArg("--role-path");
const message = getArg("--message", "你好，请用一句话自我介绍。");
const sessionId = getArg("--session-id");
const sceneId = getArg("--scene-id");
const timeoutMs = Math.floor(Number(getArg("--timeout", "120")) * 1000);

if (!rolePath) {
  console.error("用法: node client.mjs --role-path <角色目录绝对路径> [--base-url URL] [--message 文本]");
  process.exit(1);
}

async function withTimeout(promise, ms, label) {
  let t;
  const timeout = new Promise((_, rej) => {
    t = setTimeout(() => rej(new Error(`${label} 超时（>${ms / 1000}s）`)), ms);
  });
  try {
    return await Promise.race([promise, timeout]);
  } finally {
    clearTimeout(t);
  }
}

async function main() {
  const healthUrl = `${baseUrl}/health`;
  let healthText;
  try {
    const r = await withTimeout(fetch(healthUrl, { method: "GET" }), Math.min(10_000, timeoutMs), "GET /health");
    healthText = await r.text();
    if (!r.ok) {
      console.error(`[health] HTTP ${r.status}: ${healthText}`);
      process.exit(1);
    }
  } catch (e) {
    console.error(`[health] 失败（内核是否在跑？）:`, e.message ?? e);
    process.exit(1);
  }
  console.log(`[health] ${healthUrl} -> ${JSON.stringify(healthText)}`);
  if (healthText.trim() !== "ok") {
    console.error("[health] 预期响应纯文本 ok");
    process.exit(1);
  }

  const chatUrl = `${baseUrl}/chat`;
  const body = {
    role_path: rolePath,
    message,
    session_id: sessionId ?? null,
    scene_id: sceneId ?? null,
  };

  try {
    const r = await withTimeout(
      fetch(chatUrl, {
        method: "POST",
        headers: { "Content-Type": "application/json; charset=utf-8" },
        body: JSON.stringify(body),
      }),
      timeoutMs,
      "POST /chat"
    );
    const text = await r.text();
    if (!r.ok) {
      console.error(`[chat] HTTP ${r.status}`);
      try {
        console.error(JSON.stringify(JSON.parse(text), null, 2));
      } catch {
        console.error(text);
      }
      process.exit(1);
    }
    const obj = JSON.parse(text);
    if (obj.reply == null) {
      console.log(JSON.stringify(obj, null, 2));
      console.error("[chat] 响应中无 reply 字段");
      process.exit(1);
    }
    console.log("[chat] reply:");
    console.log(obj.reply);
  } catch (e) {
    console.error(`[chat] 失败:`, e.message ?? e);
    process.exit(1);
  }
}

main();
