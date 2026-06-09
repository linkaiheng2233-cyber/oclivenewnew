/**
 * Reply post-process polish — directory plugin (rule-gated LLM polish).
 *
 * Enable in role pack config.json:
 *   "reply_post_processor": { "enabled": true, "backend": "directory",
 *     "directory": { "plugin_id": "reply-post-process-polish" } }
 */
import http from "node:http";
import path from "node:path";
import { getPresetForRole } from "./preset_cache.mjs";
import {
  buildPolishUserBlock,
  polishWithOllama,
  readPolishModel,
} from "./ollama_client.mjs";
import { shouldPolish } from "./polish_rules.mjs";

const PROTOCOL_HEADER = "x-oclive-remote-protocol";
const PROTOCOL_VALUE = "oclive-remote-jsonrpc-v1";

function jsonRpcResult(id, result) {
  return JSON.stringify({ jsonrpc: "2.0", id, result });
}

function jsonRpcError(id, code, message) {
  return JSON.stringify({
    jsonrpc: "2.0",
    id,
    error: { code, message },
  });
}

function readRolesDir() {
  const fromEnv = process.env.OCLIVE_ROLES_DIR?.trim();
  if (fromEnv) {
    return fromEnv;
  }
  return path.resolve(process.cwd(), "..", "..", "roles");
}

/**
 * @param {Record<string, unknown> | null | undefined} params
 * @returns {Promise<{ display_reply: string, diagnostic?: string }>}
 */
async function polishReply(params) {
  const raw = params && typeof params.raw_reply === "string" ? params.raw_reply : "";
  const userMessage =
    params && typeof params.user_message === "string" ? params.user_message : "";
  const roleId = params && typeof params.role_id === "string" ? params.role_id : "";
  const locale = params && typeof params.locale === "string" ? params.locale : "zh";

  if (!shouldPolish(raw, userMessage)) {
    return {
      display_reply: raw,
      diagnostic: "reply-post-process-polish:skip:rules",
    };
  }

  if (!readPolishModel()) {
    return {
      display_reply: raw,
      diagnostic: "reply-post-process-polish:skip:no-model",
    };
  }

  const rolesDir = readRolesDir();
  const preset = getPresetForRole(rolesDir, roleId);
  const userBlock = buildPolishUserBlock(userMessage, raw, locale);

  try {
    const display_reply = await polishWithOllama({
      system: preset,
      user: userBlock,
    });
    return {
      display_reply,
      diagnostic: "reply-post-process-polish:polished",
    };
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    process.stderr.write(`[reply-post-process-polish] ollama fallback: ${msg}\n`);
    return {
      display_reply: raw,
      diagnostic: `reply-post-process-polish:fallback:${msg}`,
    };
  }
}

const server = http.createServer((req, res) => {
  if (req.method !== "POST" || !req.url || !req.url.startsWith("/rpc")) {
    res.writeHead(404, { "Content-Type": "text/plain; charset=utf-8" });
    res.end("not found");
    return;
  }
  const chunks = [];
  req.on("data", (c) => chunks.push(c));
  req.on("end", () => {
    void (async () => {
      const rawBody = Buffer.concat(chunks).toString("utf8");
      let msg;
      try {
        msg = JSON.parse(rawBody);
      } catch {
        res.writeHead(400, { "Content-Type": "application/json; charset=utf-8" });
        res.end(jsonRpcError(null, -32700, "parse error"));
        return;
      }
      const id = msg.id ?? null;
      if (msg.jsonrpc !== "2.0" || typeof msg.method !== "string") {
        res.writeHead(400, { "Content-Type": "application/json; charset=utf-8" });
        res.end(jsonRpcError(id, -32600, "invalid request"));
        return;
      }
      res.setHeader("Content-Type", "application/json; charset=utf-8");
      res.setHeader(PROTOCOL_HEADER, PROTOCOL_VALUE);
      if (msg.method === "reply_post_process.process") {
        try {
          const result = await polishReply(msg.params);
          res.writeHead(200);
          res.end(jsonRpcResult(id, result));
        } catch (err) {
          const message = err instanceof Error ? err.message : String(err);
          res.writeHead(200);
          res.end(jsonRpcError(id, -32603, message));
        }
        return;
      }
      res.writeHead(200);
      res.end(jsonRpcError(id, -32601, `method not found: ${msg.method}`));
    })();
  });
});

server.listen(0, "127.0.0.1", () => {
  const addr = server.address();
  const port = typeof addr === "object" && addr ? addr.port : 0;
  const url = `http://127.0.0.1:${port}/rpc`;
  process.stdout.write(`OCLIVE_READY ${url}\n`);
});

process.on("SIGTERM", () => server.close());
process.on("SIGINT", () => server.close());
