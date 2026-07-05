/**
 * Official voice.asr directory plugin — JSON-RPC sidecar.
 * Node gateway spawns Python sherpa-onnx ASR/TTS (SSOT: examples/voice-loop-minimal/).
 * Startup line: OCLIVE_READY http://127.0.0.1:<port>/rpc
 */
import { spawn } from "node:child_process";
import fs from "node:fs";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const PROTOCOL_HEADER = "x-oclive-remote-protocol";
const PROTOCOL_VALUE = "oclive-remote-jsonrpc-v1";
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PLATFORM = process.platform;

/** @type {Record<string, unknown> | null} */
let pluginConfig = null;

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

function readProfiles() {
  const p = path.join(__dirname, "asr_profiles.json");
  try {
    return JSON.parse(fs.readFileSync(p, "utf8"));
  } catch {
    return {
      default_profile: "sherpa-paraformer-zh-small",
      default_tts_profile: "sherpa-piper-zh",
      default_director_profile: "rules-v1",
      profiles: {},
    };
  }
}

const RULES_V1_SPEED = {
  shy: 0.85,
  happy: 1.1,
  sad: 0.75,
  angry: 1.15,
  fearful: 0.8,
  surprised: 1.05,
  disgusted: 0.9,
  neutral: 1.0,
};

function loadVoiceProfileFromRole(rolePath) {
  const base = String(rolePath || "").trim();
  if (!base) return null;
  const file = path.join(base, "voice_profile.json");
  if (!fs.existsSync(file)) return null;
  try {
    return JSON.parse(fs.readFileSync(file, "utf8"));
  } catch {
    return null;
  }
}

function validateDirective(directive) {
  if (!directive || typeof directive !== "object") return false;
  if (directive.schema_version !== 1) return false;
  const required = ["emotion_tag", "speed", "energy", "emo_text", "synth_profile"];
  return required.every((k) => Object.prototype.hasOwnProperty.call(directive, k));
}

function userModelsRoot() {
  const env = process.env.OCLIVE_VOICE_MODELS_DIR?.trim();
  if (env) return env;
  if (PLATFORM === "win32") {
    const appData = process.env.APPDATA || path.join(os.homedir(), "AppData", "Roaming");
    return path.join(appData, "OCLive", "models");
  }
  if (PLATFORM === "darwin") {
    return path.join(os.homedir(), "Library", "Application Support", "OCLive", "models");
  }
  const xdg = process.env.XDG_DATA_HOME || path.join(os.homedir(), ".local", "share");
  return path.join(xdg, "OCLive", "models");
}

function findEngineRoot() {
  const env = process.env.OCLIVE_VOICE_ENGINE_ROOT?.trim();
  if (env && fs.existsSync(path.join(env, "asr", "transcribe.py"))) return env;
  let dir = __dirname;
  for (let i = 0; i < 10; i += 1) {
    const candidate = path.join(dir, "examples", "voice-loop-minimal");
    if (fs.existsSync(path.join(candidate, "asr", "transcribe.py"))) return candidate;
    const parent = path.dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  const local = path.join(__dirname, "engine");
  if (fs.existsSync(path.join(local, "asr", "transcribe.py"))) return local;
  return null;
}

function findPythonExecutable(engineRoot) {
  const candidates = [];
  const envPy = process.env.OCLIVE_VOICE_PYTHON?.trim();
  if (envPy) candidates.push(envPy);
  if (engineRoot) {
    const venvNames =
      PLATFORM === "win32"
        ? [".venv/Scripts/python.exe", "venv/Scripts/python.exe"]
        : [".venv/bin/python3", ".venv/bin/python", "venv/bin/python3"];
    for (const rel of venvNames) {
      candidates.push(path.join(engineRoot, rel));
    }
  }
  candidates.push(PLATFORM === "win32" ? "py" : "python3");
  candidates.push("python");
  for (const cmd of candidates) {
    if (cmd.includes(path.sep) && !fs.existsSync(cmd)) continue;
    return cmd;
  }
  return "python";
}

function resolvePlatformProfile(profile) {
  const cfg = readProfiles();
  const id = (profile?.profile || profile || cfg.default_profile || "").trim();
  const profiles = cfg.profiles || {};
  const base = profiles[id];
  if (!base) {
    return { id, ok: false, reason: "profile_not_found" };
  }
  const platforms = base.platforms || {};
  const slot = platforms[PLATFORM];
  if (slot?.unsupported) {
    return {
      id,
      ok: false,
      reason: "unsupported_platform",
      profile: base,
      platform: PLATFORM,
      message: slot.message || `${PLATFORM} ASR not yet supported`,
    };
  }
  const merged = {
    ...base,
    ...(slot || {}),
    id,
  };
  return { id, ok: true, profile: merged };
}

function resolveModelDir(profileRec) {
  const profile = profileRec.profile;
  const profileId = profileRec.id;
  const rel = profile.model_dir || `models/asr/${profileId}`;
  const candidates = [
    path.join(userModelsRoot(), "asr", profileId),
    path.join(__dirname, rel),
    path.join(findEngineRoot() || "", "models", "asr", profileId),
  ];
  for (const dir of candidates) {
    if (dir && fs.existsSync(dir)) return dir;
  }
  return candidates[0];
}

function resolveTtsModelDir(profileId) {
  const id = (profileId || pluginConfig?.tts_profile || readProfiles().default_tts_profile || "sherpa-piper-zh").trim();
  const resolved = resolvePlatformProfile(id);
  const profile = resolved.ok ? resolved.profile : null;
  const rel = profile?.model_dir || `models/tts/${id}`;
  const candidates = [
    path.join(userModelsRoot(), "tts", id),
    path.join(__dirname, rel),
    path.join(findEngineRoot() || "", "models", "tts", id),
  ];
  for (const dir of candidates) {
    if (dir && fs.existsSync(dir)) return dir;
  }
  return candidates[0];
}

function resolveTtsProfileRecord(profileId) {
  const id = (profileId || pluginConfig?.tts_profile || readProfiles().default_tts_profile || "sherpa-piper-zh").trim();
  const resolved = resolvePlatformProfile(id);
  if (!resolved.ok) {
    return { id, ok: false, profile: null };
  }
  return { id, ok: true, profile: resolved.profile };
}

function spawnPythonJson(moduleName, payload, timeoutMs = 120_000) {
  const engineRoot = findEngineRoot();
  if (!engineRoot) {
    return Promise.resolve({
      ok: false,
      reason: "engine_root_missing",
      message: "voice-loop-minimal/asr not found; set OCLIVE_VOICE_ENGINE_ROOT",
    });
  }
  const python = findPythonExecutable(engineRoot);
  const args =
    python === "py" && PLATFORM === "win32"
      ? ["-3", "-m", moduleName]
      : ["-m", moduleName];
  return new Promise((resolve) => {
    const child = spawn(python, args, {
      cwd: engineRoot,
      env: {
        ...process.env,
        PYTHONPATH: engineRoot,
        PYTHONIOENCODING: "utf-8",
        PYTHONUTF8: "1",
      },
      stdio: ["pipe", "pipe", "pipe"],
    });
    let stdout = "";
    let stderr = "";
    const timer = setTimeout(() => {
      child.kill("SIGTERM");
      resolve({ ok: false, reason: "engine_timeout", message: `${moduleName} timed out` });
    }, timeoutMs);
    child.stdout.on("data", (chunk) => {
      stdout += chunk.toString("utf8");
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk.toString("utf8");
    });
    child.on("error", (err) => {
      clearTimeout(timer);
      resolve({
        ok: false,
        reason: "engine_spawn_failed",
        message: err instanceof Error ? err.message : String(err),
      });
    });
    child.on("close", (code) => {
      clearTimeout(timer);
      const line = stdout.trim().split("\n").filter(Boolean).pop() || "";
      if (!line) {
        resolve({
          ok: false,
          reason: "engine_empty_output",
          message: stderr.trim() || `exit ${code}`,
        });
        return;
      }
      try {
        const parsed = JSON.parse(line);
        resolve(parsed);
      } catch {
        resolve({
          ok: false,
          reason: "engine_bad_json",
          message: line.slice(0, 200),
        });
      }
    });
    child.stdin.write(JSON.stringify(payload));
    child.stdin.end();
  });
}

function copyDirRecursive(src, dest) {
  fs.mkdirSync(dest, { recursive: true });
  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    const from = path.join(src, entry.name);
    const to = path.join(dest, entry.name);
    if (entry.isDirectory()) copyDirRecursive(from, to);
    else fs.copyFileSync(from, to);
  }
}

function handleProbe(params) {
  const resolved = resolvePlatformProfile(params?.profile);
  if (!resolved.ok) {
    return {
      ok: false,
      profile: resolved.id,
      engine: "",
      platform: PLATFORM,
      reason: resolved.reason,
      message:
        resolved.message ||
        "未找到 ASR 档案；请在设置中导入模型或放置 models/ 目录",
    };
  }
  const modelDir = resolveModelDir(resolved);
  const engineRoot = findEngineRoot();
  if (!engineRoot) {
    return {
      ok: false,
      profile: resolved.id,
      engine: resolved.profile.engine || "sherpa-onnx",
      platform: PLATFORM,
      reason: "engine_root_missing",
      model_dir: modelDir,
      message: "Python 引擎路径未找到",
    };
  }
  return spawnPythonJson("asr.transcribe", {
    probe: true,
    model_dir: modelDir,
  }).then((probe) => ({
    ...probe,
    profile: resolved.id,
    platform: PLATFORM,
    model_dir: modelDir,
    engine_root: engineRoot,
    python: findPythonExecutable(engineRoot),
  }));
}

function handleListProfiles() {
  const cfg = readProfiles();
  const profiles = cfg.profiles || {};
  return {
    default_profile: cfg.default_profile || "sherpa-paraformer-zh-small",
    platform: PLATFORM,
    profiles: Object.entries(profiles).map(([id, p]) => ({
      id,
      label: p.label || id,
      engine: p.engine || "",
      model_dir: p.model_dir || "",
      kind: p.kind || (String(p.engine || "").includes("tts") ? "tts" : String(p.engine || "").includes("director") || p.engine === "rules-v1" ? "director" : "asr"),
      platform_ready: !(p.platforms?.[PLATFORM]?.unsupported),
    })),
  };
}

function handleImportModel(params) {
  const src = String(params?.src_path || params?.path || "").trim();
  const profileId = String(params?.profile || "custom").trim();
  const kind = String(params?.kind || "asr").trim();
  if (!src) {
    throw new Error("import_model: src_path required");
  }
  if (!fs.existsSync(src)) {
    return { ok: false, reason: "src_not_found", profile: profileId };
  }
  const dest = path.join(userModelsRoot(), kind, profileId);
  const stat = fs.statSync(src);
  if (stat.isDirectory()) {
    copyDirRecursive(src, dest);
  } else {
    fs.mkdirSync(dest, { recursive: true });
    const base = path.basename(src);
    fs.copyFileSync(src, path.join(dest, base));
  }
  const manifest = {
    id: profileId,
    imported_from: src,
    imported_at: new Date().toISOString(),
    engine: kind === "tts" ? "sherpa-onnx-tts" : "sherpa-onnx",
  };
  fs.writeFileSync(
    path.join(dest, "MANIFEST.json"),
    JSON.stringify(manifest, null, 2),
    "utf8",
  );
  return { ok: true, profile: profileId, dest, kind };
}

async function handleTranscribe(params) {
  const resolved = resolvePlatformProfile(params?.profile);
  if (!resolved.ok) {
    return { ok: false, text: "", reason: resolved.reason || "not_ready" };
  }
  const modelDir = resolveModelDir(resolved);
  const audioBase64 = String(params?.audio_base64 || "").trim();
  const sampleRate = Number(params?.sample_rate || 0) || undefined;
  const sampleText = String(params?.sample_text || "").trim();
  if (sampleText) {
    return {
      ok: true,
      text: sampleText,
      profile: resolved.id,
      engine: resolved.profile.engine || "sherpa-onnx",
      stub: false,
    };
  }
  if (!audioBase64) {
    const probe = await spawnPythonJson("asr.transcribe", {
      probe: true,
      model_dir: modelDir,
    });
    return {
      ok: false,
      text: "",
      reason: probe.ok ? "no_audio" : probe.reason || "not_ready",
      message: probe.message || "需要麦克风录音或 audio_base64",
      profile: resolved.id,
    };
  }
  const result = await spawnPythonJson("asr.transcribe", {
    model_dir: modelDir,
    audio_base64: audioBase64,
    sample_rate: sampleRate,
  });
  return {
    ...result,
    profile: resolved.id,
  };
}

async function handleSpeak(params) {
  const text = String(params?.text || "").trim();
  if (!text) {
    return { ok: false, reason: "empty_text", audio_base64: "" };
  }
  const directive = params?.directive && typeof params.directive === "object" ? params.directive : null;
  const profileId =
    directive?.synth_profile ||
    params?.profile ||
    pluginConfig?.tts_profile ||
    readProfiles().default_tts_profile ||
    "sherpa-piper-zh";
  const profileRec = resolveTtsProfileRecord(profileId);
  const modelDir = resolveTtsModelDir(profileId);
  const result = await spawnPythonJson("tts.synthesize", {
    model_dir: modelDir,
    text,
    speed: directive?.speed,
    directive,
    engine: profileRec.profile?.engine,
    voice: profileRec.profile?.voice,
  });
  return {
    ...result,
    profile: profileId,
    directive: directive || undefined,
  };
}

function handleBuildDirective(params) {
  const cfg = readProfiles();
  const directorId = String(
    params?.profile ||
      params?.director_profile ||
      pluginConfig?.director_profile ||
      cfg.default_director_profile ||
      "",
  ).trim();
  const botEmotion = String(params?.bot_emotion || "neutral")
    .trim()
    .toLowerCase();
  const rolePath = String(params?.role_path || "").trim();
  const roleVoice = loadVoiceProfileFromRole(rolePath);
  const synthProfile =
    roleVoice?.synth_profile ||
    pluginConfig?.tts_profile ||
    cfg.default_tts_profile ||
    "sherpa-piper-zh";
  const baselineSpeed = Number(roleVoice?.speed ?? 1.0) || 1.0;

  if (!directorId || directorId === "none") {
    const directive = {
      schema_version: 1,
      emotion_tag: botEmotion,
      speed: baselineSpeed,
      energy: roleVoice?.energy || "normal",
      emo_text: "",
      synth_profile: synthProfile,
    };
    return { ok: true, directive, director_profile: null };
  }

  const resolved = resolvePlatformProfile(directorId);
  if (!resolved.ok) {
    return { ok: false, reason: resolved.reason || "director_not_found", director_profile: directorId };
  }
  const engine = resolved.profile.engine || "rules-v1";
  if (engine !== "rules-v1") {
    return { ok: false, reason: "unsupported_director_engine", director_profile: directorId };
  }

  const emotionSpeed = RULES_V1_SPEED[botEmotion] ?? 1.0;
  const speed = Math.round(baselineSpeed * emotionSpeed * 100) / 100;
  const energy =
    roleVoice?.energy ||
    (botEmotion === "shy" || botEmotion === "sad" ? "soft" : "normal");
  const directive = {
    schema_version: 1,
    emotion_tag: botEmotion,
    speed,
    energy,
    emo_text: "",
    synth_profile: roleVoice?.synth_profile || synthProfile,
  };
  if (!validateDirective(directive)) {
    return { ok: false, reason: "invalid_directive", director_profile: directorId };
  }
  return { ok: true, directive, director_profile: directorId };
}

function handleConfigUpdated(params) {
  if (params?.config && typeof params.config === "object") {
    pluginConfig = params.config;
  }
  return { ok: true };
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
      const raw = Buffer.concat(chunks).toString("utf8");
      let msg;
      try {
        msg = JSON.parse(raw);
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
      const method = msg.method;
      const params = msg.params || {};
      try {
        let result;
        if (method === "voice.probe") result = await handleProbe(params);
        else if (method === "voice.list_profiles") result = handleListProfiles();
        else if (method === "voice.import_model") result = handleImportModel(params);
        else if (method === "voice.transcribe") result = await handleTranscribe(params);
        else if (method === "voice.speak") result = await handleSpeak(params);
        else if (method === "voice.build_directive") result = handleBuildDirective(params);
        else if (method === "config_updated") result = handleConfigUpdated(params);
        else {
          res.writeHead(200);
          res.end(jsonRpcError(id, -32601, `method not found: ${method}`));
          return;
        }
        res.writeHead(200);
        res.end(jsonRpcResult(id, result));
      } catch (e) {
        res.writeHead(200);
        res.end(
          jsonRpcError(
            id,
            -32000,
            e instanceof Error ? e.message : "voice.asr failed",
          ),
        );
      }
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
