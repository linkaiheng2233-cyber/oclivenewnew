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
const COSYVOICE_RESOURCE_ADAPTER_ID = "builtin.voice.cosyvoice2";
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PLATFORM = process.platform;
const DEFAULT_TTS_PROFILE = "bundled-cosyvoice2-zh";
/** CosyVoice2 first inference can exceed 2 min when ONNX runs on CPU. */
const COSYVOICE_SYNTH_TIMEOUT_MS = 600_000;
const COSYVOICE_WARM_TIMEOUT_MS = 900_000;

function readInitialPluginConfig() {
  const raw = String(
    process.env.OCLIVE_PLUGIN_CONFIG || process.env.OCLIVE_DEBUG_PLUGIN_CONFIG || "",
  ).trim();
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw);
    if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) return parsed;
    console.error("[PLUGIN_CONFIG_INVALID] startup config must be a JSON object");
  } catch (error) {
    console.error(
      `[PLUGIN_CONFIG_INVALID] startup config parse failed: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
  return null;
}

/** @type {Record<string, unknown> | null} */
let pluginConfig = readInitialPluginConfig();

/** @type {import("node:child_process").ChildProcess | null} */
let cosyvoiceSidecarChild = null;
/** @type {string | null} */
let cosyvoiceSidecarUrl = null;
let cosyvoiceSidecarWarmed = false;
/** @type {Promise<Record<string, unknown>> | null} */
let cosyvoiceWarmInFlight = null;
/** @type {Map<string, Promise<Record<string, unknown>>>} */
const inFlightSpeakByKey = new Map();
let localSpeakLane = Promise.resolve();

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
      default_tts_profile: DEFAULT_TTS_PROFILE,
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

const RULES_V1_EMO_TEXT = {
  shy: "用害羞轻柔的语气",
  happy: "用开心明亮的语气",
  sad: "用低沉难过的语气",
  angry: "用严厉激动的语气",
  fearful: "用紧张不安的语气",
  surprised: "用惊讶的语气",
  disgusted: "用嫌弃的语气",
  neutral: "用自然平静的语气",
  excited: "用兴奋活泼的语气",
  confused: "用困惑犹豫的语气",
};

const DEFAULT_COSYVOICE_EMO_TEXT = RULES_V1_EMO_TEXT.neutral;

const MODEL_PACK_FILENAME = "voice_model_pack.json";
const ADAPTER_PACK_FILENAME = "tts_adapter_pack.json";

function resolveRoleAssetPath(rolePath, relPath) {
  const base = String(rolePath || "").trim();
  const rel = String(relPath || "").trim();
  if (!base || !rel) return "";
  const joined = path.join(base, rel);
  return fs.existsSync(joined) ? joined : "";
}

function resolveRefAudio(rolePath, roleVoice, emotion) {
  if (!roleVoice || typeof roleVoice !== "object") return "";
  const refMap = roleVoice.ref_map;
  if (refMap && typeof refMap === "object") {
    const mapped = refMap[emotion] || refMap.neutral;
    if (mapped) {
      const resolved = resolveRoleAssetPath(rolePath, mapped);
      if (resolved) return resolved;
    }
  }
  if (roleVoice.ref_default) {
    return resolveRoleAssetPath(rolePath, roleVoice.ref_default);
  }
  return "";
}

function synthRoutingFromConfig(profileRec) {
  const cfg = pluginConfig || {};
  const profile = profileRec?.profile || {};
  const provider = String(
    profile.synth_provider || cfg.synth_provider || "bundled",
  ).trim();
  const profileEndpoint = String(profile.sidecar_endpoint || "").trim();
  const localEndpoint = String(
    profileEndpoint || cfg.local_synth_endpoint || "",
  ).trim();
  const cloudUrl = String(cfg.cloud_tts_url || "").trim();
  const cloudToken = String(cfg.cloud_tts_token || "").trim();
  const cloudVoiceId = String(
    profile.voice || cfg.cloud_tts_voice_id || "",
  ).trim();
  const cloudModel = String(cfg.cloud_tts_model || "").trim();
  const engine = String(profile.engine || "").trim();
  return {
    provider,
    localEndpoint,
    cloudUrl,
    cloudToken,
    cloudVoiceId,
    cloudModel,
    engine,
  };
}

function profileEngine(profileRec) {
  return String(profileRec?.profile?.engine || "").trim();
}

function shouldRunBundledSidecar(profileRec) {
  const cfg = pluginConfig || {};
  if (cfg.tts_expansion_enabled !== true) return false;
  const routing = synthRoutingFromConfig(profileRec);
  const engine = routing.engine || profileEngine(profileRec) || "cosyvoice2";
  if (engine !== "cosyvoice2") return false;
  return routing.provider === "bundled" || routing.provider === "";
}

function engineSupportsWarm(engine) {
  return String(engine || "").trim() === "cosyvoice2";
}

function engineSupportsStream(engine, provider) {
  return (
    String(engine || "").trim() === "cosyvoice2" &&
    (provider === "bundled" || provider === "")
  );
}

function findCosyvoicePython(engineRoot) {
  const envPy = process.env.OCLIVE_COSYVOICE_PYTHON?.trim();
  if (envPy && fs.existsSync(envPy)) return envPy;
  if (engineRoot) {
    const venvNames =
      PLATFORM === "win32"
        ? [".venv-cosyvoice/Scripts/python.exe", ".venv/Scripts/python.exe"]
        : [".venv-cosyvoice/bin/python3", ".venv/bin/python3"];
    for (const rel of venvNames) {
      const candidate = path.join(engineRoot, rel);
      if (fs.existsSync(candidate)) return candidate;
    }
  }
  return findPythonExecutable(engineRoot);
}

function stopCosyvoiceSidecar() {
  if (cosyvoiceSidecarChild) {
    try {
      cosyvoiceSidecarChild.kill("SIGTERM");
    } catch {
      /* ignore */
    }
    cosyvoiceSidecarChild = null;
    cosyvoiceSidecarUrl = null;
    cosyvoiceSidecarWarmed = false;
  }
}

function waitForChildClose(child, timeoutMs = 5000) {
  return new Promise((resolve) => {
    if (!child || child.exitCode !== null || child.signalCode !== null) {
      resolve(true);
      return;
    }
    let settled = false;
    let timer = null;
    const finish = (closed) => {
      if (settled) return;
      settled = true;
      if (timer) clearTimeout(timer);
      child.off("close", onClose);
      resolve(closed);
    };
    const onClose = () => finish(true);
    child.once("close", onClose);
    timer = setTimeout(() => finish(false), timeoutMs);
  });
}

async function releaseCosyvoiceSidecar(profileId) {
  const modelDir = resolveTtsModelDir(profileId);
  const profileRec = resolveTtsProfileRecord(profileId);
  const port = Number(profileRec.profile?.sidecar_port || 50000) || 50000;
  const endpoint = cosyvoiceSidecarUrl || `http://127.0.0.1:${port}`;
  let remoteRelease = null;
  let endpointRefusedConnection = false;
  try {
    const response = await fetch(`${endpoint.replace(/\/+$/, "")}/unload`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ model_dir: modelDir }),
      signal: AbortSignal.timeout(15_000),
    });
    remoteRelease = response.ok ? await response.json() : {
      ok: false,
      released: false,
      reason: `unload_http_${response.status}`,
    };
  } catch (error) {
    const causeCode =
      error && typeof error === "object" && error.cause && typeof error.cause === "object"
        ? String(error.cause.code || "")
        : "";
    endpointRefusedConnection = causeCode === "ECONNREFUSED";
    remoteRelease = {
      ok: false,
      released: false,
      reason: "unload_unreachable",
      message: error instanceof Error ? error.message : String(error),
    };
  }

  const child = cosyvoiceSidecarChild;
  let managedChildStopped = false;
  if (child) {
    try {
      const signaled = child.kill("SIGTERM");
      managedChildStopped = signaled && await waitForChildClose(child);
    } catch {
      managedChildStopped = false;
    }
  }
  const alreadyStopped = !child && endpointRefusedConnection;
  if (remoteRelease?.released === true || managedChildStopped || alreadyStopped) {
    if (managedChildStopped || alreadyStopped) {
      cosyvoiceSidecarChild = null;
      cosyvoiceSidecarUrl = null;
    }
    cosyvoiceSidecarWarmed = false;
    return {
      ok: true,
      released: true,
      release_mode: managedChildStopped
        ? "managed_process_stopped"
        : alreadyStopped
          ? "already_stopped"
          : "model_unloaded",
      sidecar_endpoint: endpoint,
      model_dir: modelDir,
    };
  }
  return {
    ok: false,
    released: false,
    reason: remoteRelease?.reason || "resource_release_unconfirmed",
    message: remoteRelease?.message || "CosyVoice2 resource release was not confirmed",
    sidecar_endpoint: endpoint,
    model_dir: modelDir,
  };
}

async function ensureCosyvoiceSidecarWarmed(
  profileId,
  sidecarEndpoint,
  directive = null,
  hostResourceAdmission = null,
) {
  const hasRolePrompt = directiveHasCosyvoiceInput(directive);
  if (cosyvoiceSidecarWarmed && !hasRolePrompt) {
    return { ok: true, already_warmed: true, warmed: true };
  }
  const modelDir = resolveTtsModelDir(profileId);
  const health = await probeSidecarEndpoint(sidecarEndpoint, modelDir);
  if (health.ok && health.warmed && !hasRolePrompt) {
    cosyvoiceSidecarWarmed = true;
    return {
      ok: true,
      already_warmed: true,
      warmed: true,
      sidecar_endpoint: sidecarEndpoint,
      model_dir: modelDir,
    };
  }
  return runCosyvoiceWarmSerialized(
    profileId,
    sidecarEndpoint,
    modelDir,
    directive,
    hostResourceAdmission,
  );
}

async function runCosyvoiceWarm(
  profileId,
  sidecarEndpoint,
  modelDir,
  directive = null,
  hostResourceAdmission = null,
) {
  const warm = await spawnPythonJson(
    "tts.synthesize",
    {
      warm: true,
      prime: true,
      model_dir: modelDir,
      engine: "cosyvoice2",
      sidecar_endpoint: sidecarEndpoint,
      ...(hostResourceAdmission && typeof hostResourceAdmission === "object"
        ? { host_resource_admission: hostResourceAdmission }
        : {}),
      ...(directiveHasCosyvoiceInput(directive) ? { directive } : {}),
    },
    COSYVOICE_WARM_TIMEOUT_MS,
  );
  if (warm.ok) {
    cosyvoiceSidecarWarmed = true;
  }
  return {
    ...warm,
    sidecar_endpoint: sidecarEndpoint,
    model_dir: modelDir,
    profile: profileId,
  };
}

async function runCosyvoiceWarmSerialized(
  profileId,
  sidecarEndpoint,
  modelDir,
  directive = null,
  hostResourceAdmission = null,
) {
  const previous = cosyvoiceWarmInFlight;
  const promise = (previous ? previous.catch(() => {}) : Promise.resolve())
    .then(() =>
      runCosyvoiceWarm(
        profileId,
        sidecarEndpoint,
        modelDir,
        directive,
        hostResourceAdmission,
      ),
    );
  cosyvoiceWarmInFlight = promise;
  try {
    return await promise;
  } finally {
    if (cosyvoiceWarmInFlight === promise) {
      cosyvoiceWarmInFlight = null;
    }
  }
}

function startCosyvoiceSidecar(modelDir, port = 50000) {
  stopCosyvoiceSidecar();
  const engineRoot = findEngineRoot();
  if (!engineRoot) {
    return Promise.resolve({
      ok: false,
      reason: "engine_root_missing",
      message: "voice-loop-minimal not found",
    });
  }
  const python = findCosyvoicePython(engineRoot);
  const args =
    python === "py" && PLATFORM === "win32"
      ? ["-3", "-m", "tts.cosyvoice_sidecar"]
      : ["-m", "tts.cosyvoice_sidecar"];
  return new Promise((resolve) => {
    const child = spawn(python, args, {
      cwd: engineRoot,
      env: {
        ...process.env,
        PYTHONPATH: engineRoot,
        PYTHONIOENCODING: "utf-8",
        PYTHONUTF8: "1",
        OCLIVE_COSYVOICE_MODEL_DIR: modelDir,
        OCLIVE_COSYVOICE_PORT: String(port),
      },
      stdio: ["ignore", "pipe", "pipe"],
    });
    cosyvoiceSidecarChild = child;
    let settled = false;
    const finish = (result) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      resolve(result);
    };
    const timer = setTimeout(() => {
      finish({
        ok: false,
        reason: "sidecar_start_timeout",
        message: "CosyVoice2 sidecar did not become ready",
      });
    }, 30_000);
    child.stdout.on("data", (chunk) => {
      const text = chunk.toString("utf8");
      const match = text.match(/OCLIVE_SIDECAR_READY\s+(http:\/\/[^\s]+)/);
      if (match) {
        cosyvoiceSidecarUrl = match[1].trim();
        finish({ ok: true, sidecar_endpoint: cosyvoiceSidecarUrl, warmed: false });
      }
    });
    // Surface sidecar stderr (warm/prime/synth timings + tracebacks) and drain the
    // pipe so a full buffer cannot block the Python process.
    child.stderr.on("data", (chunk) => {
      const text = chunk.toString("utf8");
      if (text.trim()) {
        process.stderr.write(`[cosyvoice-sidecar] ${text}`);
      }
    });
    child.on("error", (err) => {
      cosyvoiceSidecarChild = null;
      cosyvoiceSidecarUrl = null;
      cosyvoiceSidecarWarmed = false;
      finish({
        ok: false,
        reason: "sidecar_spawn_failed",
        message: err instanceof Error ? err.message : String(err),
      });
    });
    child.on("close", () => {
      if (cosyvoiceSidecarChild === child) {
        cosyvoiceSidecarChild = null;
        cosyvoiceSidecarUrl = null;
        cosyvoiceSidecarWarmed = false;
      }
      if (!settled) {
        finish({ ok: false, reason: "sidecar_exited", message: "CosyVoice2 sidecar exited" });
      }
    });
  });
}

async function probeSidecarEndpoint(endpoint, expectedModelDir) {
  const base = String(endpoint || "").trim().replace(/\/+$/, "");
  if (!base) {
    return { ok: false, reason: "endpoint_missing" };
  }
  try {
    const res = await fetch(`${base}/health`, { signal: AbortSignal.timeout(3000) });
    if (!res.ok) {
      return { ok: false, reason: "health_http", endpoint: base };
    }
    const body = await res.json();
    const expected = path.resolve(expectedModelDir);
    const actual = path.resolve(String(body.model_dir || ""));
    if (expected.toLowerCase() !== actual.toLowerCase()) {
      return {
        ok: false,
        reason: "sidecar_model_mismatch",
        endpoint: base,
        message: `Sidecar at ${base} uses ${actual}; expected ${expected}`,
      };
    }
    return {
      ok: body.ok === true,
      sidecar_endpoint: base,
      model_dir: actual,
      warmed: body.warmed === true,
      primed: body.primed === true,
      precision_requested: body.precision_requested || "auto",
      precision_active: body.precision_active || "fp32",
      precision_fallback_reason: body.precision_fallback_reason || "",
      load_strategy: body.load_strategy || "unknown",
      load_admission_detail: body.load_admission_detail || "",
      load_vram_probe: body.load_vram_probe || "unavailable",
      load_free_vram_before_mib: Number(body.load_free_vram_before_mib) || 0,
      load_total_vram_mib: Number(body.load_total_vram_mib) || 0,
      load_min_free_vram_mib: Number(body.load_min_free_vram_mib) || 0,
      load_peak_allocated_mib: Number(body.load_peak_allocated_mib) || 0,
      load_peak_reserved_mib: Number(body.load_peak_reserved_mib) || 0,
      retryable: body.retryable === true,
      reason: body.reason || "",
      message: body.message || "",
    };
  } catch (err) {
    return {
      ok: false,
      reason: "health_unreachable",
      endpoint: base,
      message: err instanceof Error ? err.message : String(err),
    };
  }
}

async function ensureCosyvoiceSidecar(profileId) {
  const profileRec = resolveTtsProfileRecord(profileId);
  if (!shouldRunBundledSidecar(profileRec)) {
    return { ok: false, reason: "sidecar_not_applicable" };
  }
  const modelDir = resolveTtsModelDir(profileId);
  const cfg = readProfiles();
  const profile = cfg.profiles?.[profileId] || {};
  const port = Number(profile.sidecar_port || 50000) || 50000;
  if (cosyvoiceSidecarChild && cosyvoiceSidecarUrl) {
    return { ok: true, sidecar_endpoint: cosyvoiceSidecarUrl, model_dir: modelDir };
  }
  const portUrl = `http://127.0.0.1:${port}`;
  const existing = await probeSidecarEndpoint(portUrl, modelDir);
  if (existing.ok) {
    cosyvoiceSidecarUrl = existing.sidecar_endpoint;
    if (existing.warmed) {
      cosyvoiceSidecarWarmed = true;
    }
    return {
      ok: true,
      sidecar_endpoint: cosyvoiceSidecarUrl,
      model_dir: modelDir,
      adopted: true,
    };
  }
  if (existing.reason === "sidecar_model_mismatch") {
    return {
      ok: false,
      reason: existing.reason,
      message: `${existing.message}. Close other CosyVoice sidecar processes, then run Warm TTS again.`,
      sidecar_endpoint: existing.endpoint,
    };
  }
  return startCosyvoiceSidecar(modelDir, port);
}

function readModelPackMeta(src) {
  const direct = path.join(src, MODEL_PACK_FILENAME);
  if (fs.existsSync(direct)) {
    try {
      return JSON.parse(fs.readFileSync(direct, "utf8"));
    } catch {
      return null;
    }
  }
  return null;
}
function handleReadRoleProfile(params) {
  const rolePath = String(params?.role_path || "").trim();
  const vp = loadVoiceProfileFromRole(rolePath);
  if (!vp) {
    return { ok: true, profile: null };
  }
  const preferred =
    (typeof vp.preferred_tts_profile === "string" && vp.preferred_tts_profile.trim()) ||
    (typeof vp.synth_profile === "string" && vp.synth_profile.trim()) ||
    null;
  return {
    ok: true,
    profile: {
      preferred_tts_profile: preferred,
      synth_profile:
        typeof vp.synth_profile === "string" ? vp.synth_profile.trim() : null,
      director_profile:
        typeof vp.director_profile === "string" ? vp.director_profile.trim() : null,
    },
  };
}

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

function loadCorePersonalityFromRole(rolePath) {
  const base = String(rolePath || "").trim();
  if (!base) return "";
  const file = path.join(base, "core_personality.txt");
  if (!fs.existsSync(file)) return "";
  try {
    return fs.readFileSync(file, "utf8");
  } catch {
    return "";
  }
}


function countPositiveKeywordMentions(text, keyword) {
  const negRe = new RegExp(
    `(?:不|勿|别|没有|不会|不要|绝不|禁止|勿)[^\\n。；]{0,12}${keyword}`,
  );
  let score = 0;
  for (const line of text.split("\n")) {
    if (!line.includes(keyword)) continue;
    if (negRe.test(line)) continue;
    if (/绝不用/.test(line) && line.includes(keyword)) continue;
    score += line.split(keyword).length - 1;
  }
  return score;
}

function sumKeywordScores(text, keywords) {
  return keywords.reduce((sum, keyword) => sum + countPositiveKeywordMentions(text, keyword), 0);
}

function deriveVoiceStyleFromPersonality(coreText) {
  const text = String(coreText || "").trim();
  if (!text) return null;

  const childlike = sumKeywordScores(text, ["小女孩", "孩子气", "女儿", "学妹"]);
  const feminine = childlike + sumKeywordScores(text, ["女生", "姐姐", "少女"]);
  const masculine = sumKeywordScores(text, ["正太", "少年", "男生"]);

  const cute = sumKeywordScores(text, ["可爱", "软萌", "软软", "软糯", "撒娇"]);
  const gentle = sumKeywordScores(text, ["温柔", "轻轻", "安静", "友善", "心很软", "从容", "不急不缓"]);
  const lively = sumKeywordScores(text, ["活泼", "元气", "兴奋", "蹦两下", "脚步会变轻"]);
  const shy = sumKeywordScores(text, ["害羞", "脸红", "声音越来越小", "耳朵红", "结巴"]);
  const tsundere = sumKeywordScores(text, ["傲娇", "口嫌体正直", "别扭", "嘴硬"]);
  const sharp = sumKeywordScores(text, ["毒舌", "带刺", "烦人", "差劲", "变态", "挖苦"]);
  const caring = sumKeywordScores(text, ["关心", "照顾", "体贴", "护短", "陪着", "安抚"]);

  const childDominant = childlike >= 2;
  const sharpDominant = sharp >= 2 || (sharp >= 1 && tsundere >= 1);
  const gentleDominant = gentle >= 3 && !sharpDominant;

  const styleCandidates = [];
  const pushStyle = (bit, priority) => {
    if (!styleCandidates.some(item => item.bit === bit)) {
      styleCandidates.push({ bit, priority });
    }
  };

  if (childDominant && cute > 0) pushStyle("软萌可爱", 10);
  if (masculine > 0 && cute > 0 && !childDominant) pushStyle("温和软萌", 8);
  if (gentle > 0 && !sharpDominant) pushStyle("温柔轻软", gentleDominant ? 8 : 5);
  if (lively > 0 && (childDominant || !sharpDominant)) pushStyle("自然有活力", 6);
  if (sharpDominant) pushStyle("带一点冷淡锋利感", 9);
  if (tsundere > 0) {
    pushStyle(
      childDominant ? "偶尔带一点点嘴硬撒娇" : "偶尔嘴硬掩饰真实情绪",
      7,
    );
  }
  if (shy > 0) pushStyle("害羞时会轻轻放小声音", 5);
  if (caring > 0) pushStyle("会把关心感放进语气细节里", sharpDominant ? 4 : 6);
  if (styleCandidates.length === 0) pushStyle("自然生动", 1);

  const selectedStyleBits = styleCandidates
    .sort((a, b) => b.priority - a.priority)
    .slice(0, 3)
    .map(item => item.bit);

  let speakerStyle = "角色感声线";
  if (childDominant) speakerStyle = "小女孩嗓音";
  else if (masculine > feminine) {
    speakerStyle = gentleDominant ? "温和从容的少年感男声" : "自然的少年感男声";
  } else if (feminine > 0) {
    if (sharpDominant) speakerStyle = "带一点清冷感的少女声线";
    else if (gentleDominant) speakerStyle = "温柔自然的少女声线";
    else speakerStyle = "自然的少女声线";
  }

  const speed = childDominant && lively > 0 ? 1.0 : sharpDominant ? 0.98 : gentleDominant ? 0.95 : 0.98;
  const energy = sharpDominant ? "normal" : childDominant && lively > 0 ? "normal" : "soft";
  const emoTextTemplate
    = `用${selectedStyleBits.join("、")}的${speakerStyle}说话，语气自然有起伏，不要平铺直叙，也不要太像播报；{tone}`;

  return {
    speed,
    energy,
    emoTextTemplate,
  };
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

function resolveTtsAdapterDir(adapterId) {
  const id = String(adapterId || "").trim();
  if (!id) return "";
  const candidates = [
    path.join(userModelsRoot(), "tts_adapters", id),
    path.join(__dirname, "models", "tts_adapters", id),
  ];
  for (const dir of candidates) {
    if (dir && fs.existsSync(dir)) return dir;
  }
  return candidates[0];
}

function resolveTtsModelDir(profileId) {
  const id = (profileId || pluginConfig?.tts_profile || readProfiles().default_tts_profile || DEFAULT_TTS_PROFILE).trim();
  const resolved = resolvePlatformProfile(id);
  const profile = resolved.ok ? resolved.profile : null;
  if (profile?.engine === "generic-http-adapter" && profile.adapter_id) {
    return resolveTtsAdapterDir(profile.adapter_id);
  }
  const rel = profile?.model_dir || `models/tts/${id}`;
  const folderName = rel ? path.basename(rel.replace(/\\/g, "/")) : id;
  const candidates = [
    path.join(userModelsRoot(), "tts", folderName),
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
  const id = (profileId || pluginConfig?.tts_profile || readProfiles().default_tts_profile || DEFAULT_TTS_PROFILE).trim();
  const resolved = resolvePlatformProfile(id);
  if (!resolved.ok) {
    return { id, ok: false, profile: null };
  }
  return { id, ok: true, profile: resolved.profile };
}

function stableJson(value) {
  if (Array.isArray(value)) {
    return `[${value.map(stableJson).join(",")}]`;
  }
  if (value && typeof value === "object") {
    return `{${Object.keys(value)
      .sort()
      .map(key => `${JSON.stringify(key)}:${stableJson(value[key])}`)
      .join(",")}}`;
  }
  return JSON.stringify(value);
}

function speakRequestKey(payload) {
  return stableJson({
    text: String(payload?.text || "").trim(),
    engine: String(payload?.engine || "").trim(),
    model_dir: String(payload?.model_dir || "").trim(),
    sidecar_endpoint: String(payload?.sidecar_endpoint || "").trim(),
    voice: String(payload?.voice || "").trim(),
    cloud_url: String(payload?.cloud_url || "").trim(),
    cloud_voice_id: String(payload?.cloud_voice_id || "").trim(),
    cloud_model: String(payload?.cloud_model || "").trim(),
    directive: payload?.directive && typeof payload.directive === "object" ? payload.directive : null,
  });
}

function shouldSerializeLocalSpeak(routing, engine) {
  const provider = String(routing?.provider || "").trim();
  const id = String(engine || "").trim();
  if (!id) return false;
  if (provider === "cloud" || id === "edge-tts") return false;
  return true;
}

function runInLocalSpeakLane(task) {
  const next = localSpeakLane.then(task, task);
  localSpeakLane = next.finally(() => {
    if (localSpeakLane === next) {
      localSpeakLane = Promise.resolve();
    }
  });
  return next;
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
    default_tts_profile: cfg.default_tts_profile || DEFAULT_TTS_PROFILE,
    platform: PLATFORM,
    profiles: Object.entries(profiles).map(([id, p]) => ({
      id,
      label: p.label || id,
      engine: p.engine || "",
      model_dir: p.model_dir || "",
      kind:
        p.kind ||
        (String(p.engine || "").includes("tts")
          ? "tts"
          : String(p.engine || "").includes("director") || p.engine === "rules-v1"
            ? "director"
            : "asr"),
      platform_ready: !(p.platforms?.[PLATFORM]?.unsupported),
      requires_pack: p.requires_pack || "",
      min_vram_gb_recommended: p.min_vram_gb_recommended ?? null,
      synth_provider: p.synth_provider || "",
      adapter_id: p.adapter_id || "",
      sidecar_endpoint: p.sidecar_endpoint || "",
    })),
  };
}

function handleListModelPacks() {
  const cfg = readProfiles();
  const packs = [];
  for (const [id, p] of Object.entries(cfg.profiles || {})) {
    if (p.kind !== "tts" || !p.requires_pack) continue;
    const modelDir = resolveTtsModelDir(id);
    const ready = fs.existsSync(path.join(modelDir, "MANIFEST.json"));
    packs.push({
      pack_id: p.requires_pack,
      profile_id: id,
      label: p.label || id,
      engine: p.engine || "",
      min_vram_gb_recommended: p.min_vram_gb_recommended ?? null,
      installed: ready,
      model_dir: modelDir,
      download_url: p.download_url || "",
    });
  }
  return { ok: true, packs };
}

function handleImportModel(params) {
  const src = String(params?.src_path || params?.path || "").trim();
  let profileId = String(params?.profile || "custom").trim();
  let kind = String(params?.kind || "asr").trim();
  if (!src) {
    throw new Error("import_model: src_path required");
  }
  if (!fs.existsSync(src)) {
    return { ok: false, reason: "src_not_found", profile: profileId };
  }

  const packMeta = readModelPackMeta(src);
  if (packMeta) {
    profileId = String(packMeta.install_profile || packMeta.pack_id || profileId).trim();
    kind = "tts";
    const installName =
      String(packMeta.install_dir || "")
        .split(/[/\\]/)
        .filter(Boolean)
        .pop() || "cosyvoice2-0.5b";
    profileId = installName.includes("cosyvoice") ? "cosyvoice2-0.5b" : profileId;
  }

  const dest = path.join(userModelsRoot(), kind, profileId);
  const stat = fs.statSync(src);
  if (stat.isDirectory()) {
    copyDirRecursive(src, dest);
  } else if (src.toLowerCase().endsWith(".zip")) {
    return {
      ok: false,
      reason: "zip_extract_not_supported",
      message: "Extract the zip to a folder, then import the folder path",
      profile: profileId,
    };
  } else {
    fs.mkdirSync(dest, { recursive: true });
    const base = path.basename(src);
    fs.copyFileSync(src, path.join(dest, base));
  }

  const engine =
    packMeta?.engine ||
    (kind === "tts" ? "cosyvoice2" : "sherpa-onnx");
  const manifest = {
    id: profileId,
    imported_from: src,
    imported_at: new Date().toISOString(),
    engine,
    ...(packMeta?.pack_id ? { pack_id: packMeta.pack_id } : {}),
  };
  fs.writeFileSync(
    path.join(dest, "MANIFEST.json"),
    JSON.stringify(manifest, null, 2),
    "utf8",
  );
  if (packMeta) {
    fs.writeFileSync(
      path.join(dest, MODEL_PACK_FILENAME),
      JSON.stringify(packMeta, null, 2),
      "utf8",
    );
  }
  return { ok: true, profile: profileId, dest, kind, pack: packMeta || undefined };
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

function directiveHasCosyvoiceInput(directive) {
  if (!directive || typeof directive !== "object") return false;
  if (String(directive.emo_text || "").trim()) return true;
  const ref = String(directive.ref_audio || "").trim();
  return ref.length > 0 && fs.existsSync(ref);
}

function ensureCosyvoiceSpeakDirective(directive, params) {
  if (directiveHasCosyvoiceInput(directive)) return directive;
  const built = handleBuildDirective({
    bot_emotion: params?.bot_emotion || directive?.emotion_tag || "neutral",
    role_path: String(params?.role_path || ""),
    profile: pluginConfig?.director_profile,
  });
  if (built.ok && built.directive && directiveHasCosyvoiceInput(built.directive)) {
    return built.directive;
  }
  const emotion = String(params?.bot_emotion || directive?.emotion_tag || "neutral")
    .trim()
    .toLowerCase();
  return {
    schema_version: 1,
    emotion_tag: emotion,
    speed: Number(directive?.speed ?? 1.0) || 1.0,
    energy: directive?.energy || "normal",
    emo_text: RULES_V1_EMO_TEXT[emotion] || DEFAULT_COSYVOICE_EMO_TEXT,
    synth_profile:
      directive?.synth_profile ||
      pluginConfig?.tts_profile ||
      readProfiles().default_tts_profile ||
      DEFAULT_TTS_PROFILE,
    ...(directive?.ref_audio ? { ref_audio: directive.ref_audio } : {}),
    ...(directive?.ref_text ? { ref_text: directive.ref_text } : {}),
  };
}

async function handleSpeak(params) {
  const text = String(params?.text || "").trim();
  if (!text) {
    return { ok: false, reason: "empty_text", audio_base64: "" };
  }
  if (pluginConfig?.tts_expansion_enabled !== true) {
    return {
      ok: false,
      reason: "tts_expansion_disabled",
      message: "Enable voice expansion in settings to use emotional TTS",
      audio_base64: "",
    };
  }
  let directive =
    params?.directive && typeof params.directive === "object" ? params.directive : null;
  directive = ensureCosyvoiceSpeakDirective(directive, params);
  const profileId =
    directive?.synth_profile ||
    params?.profile ||
    pluginConfig?.tts_profile ||
    readProfiles().default_tts_profile ||
    DEFAULT_TTS_PROFILE;
  const profileRec = resolveTtsProfileRecord(profileId);
  const routing = synthRoutingFromConfig(profileRec);
  const engine = routing.engine || profileRec.profile?.engine;
  let sidecarEndpoint = routing.localEndpoint;
  if (shouldRunBundledSidecar(profileRec)) {
    const sidecar = await ensureCosyvoiceSidecar(profileId);
    if (!sidecar.ok) {
      return {
        ok: false,
        reason: sidecar.reason || "sidecar_not_ready",
        message: sidecar.message || "Start CosyVoice2 sidecar and import model pack",
        audio_base64: "",
      };
    }
    sidecarEndpoint = sidecar.sidecar_endpoint || sidecarEndpoint;
    const warm = await ensureCosyvoiceSidecarWarmed(
      profileId,
      sidecarEndpoint,
      null,
      params?._oclive_resource_admission,
    );
    if (!warm.ok) {
      return {
        ok: false,
        reason: warm.reason || "not_warmed",
        message: warm.message || "CosyVoice2 sidecar warm failed — try 预热 TTS 侧车 in settings",
        audio_base64: "",
      };
    }
  }
  const modelDir = resolveTtsModelDir(profileId);
  const payload = {
    model_dir: modelDir,
    text,
    speed: directive?.speed,
    directive,
    engine,
    voice: profileRec.profile?.voice,
    sidecar_endpoint: sidecarEndpoint,
    cloud_url: routing.cloudUrl,
    cloud_token: routing.cloudToken,
    cloud_voice_id: routing.cloudVoiceId,
    cloud_model: routing.cloudModel,
  };
  if (routing.provider === "cloud" && engine !== "edge-tts") {
    payload.engine = "cloud-tts-openai";
  }
  const finalizeSpeak = () => spawnPythonJson(
    "tts.synthesize",
    payload,
    engine === "cosyvoice2" ? COSYVOICE_SYNTH_TIMEOUT_MS : undefined,
  ).then((result) => ({
    ...result,
    profile: profileId,
    engine: payload.engine || engine,
    directive: directive || undefined,
  }));
  const key = speakRequestKey(payload);
  const existing = inFlightSpeakByKey.get(key);
  if (existing) {
    return existing;
  }
  const promise = shouldSerializeLocalSpeak(routing, payload.engine || engine)
    ? runInLocalSpeakLane(finalizeSpeak)
    : finalizeSpeak();
  inFlightSpeakByKey.set(key, promise);
  try {
    return await promise;
  } finally {
    if (inFlightSpeakByKey.get(key) === promise) {
      inFlightSpeakByKey.delete(key);
    }
  }
}

async function handleProbeTts(params) {
  const profileId =
    params?.profile ||
    pluginConfig?.tts_profile ||
    readProfiles().default_tts_profile ||
    DEFAULT_TTS_PROFILE;
  const profileRec = resolveTtsProfileRecord(profileId);
  const routing = synthRoutingFromConfig(profileRec);
  let sidecarEndpoint = routing.localEndpoint;
  let sidecarReady = false;
  if (shouldRunBundledSidecar(profileRec) && pluginConfig?.tts_expansion_enabled === true) {
    const sidecar = await ensureCosyvoiceSidecar(profileId);
    if (sidecar.ok) {
      sidecarEndpoint = sidecar.sidecar_endpoint || sidecarEndpoint;
      sidecarReady = true;
    }
  }
  const modelDir = resolveTtsModelDir(profileId);
  const engineRoot = findEngineRoot();
  if (!engineRoot) {
    return {
      ok: false,
      profile: profileId,
      reason: "engine_root_missing",
      message: "Python engine path not found",
    };
  }
  const engine = routing.engine || profileRec.profile?.engine;
  const probePayload = {
    probe: true,
    model_dir: modelDir,
    engine: routing.provider === "cloud" && engine !== "edge-tts" ? "cloud-tts-openai" : engine,
    sidecar_endpoint: sidecarEndpoint,
  };
  const probe = await spawnPythonJson("tts.synthesize", probePayload);
  return {
    ...probe,
    profile: profileId,
    platform: PLATFORM,
    model_dir: modelDir,
    engine: probe.engine || engine,
    synth_provider: routing.provider,
    supports_stream: engineSupportsStream(engine, routing.provider),
    supports_warm: engineSupportsWarm(engine),
    expansion_enabled: pluginConfig?.tts_expansion_enabled === true,
    sidecar_ready: sidecarReady || probe.ok === true,
  };
}

async function handleWarm(params) {
  const profileId =
    params?.profile ||
    pluginConfig?.tts_profile ||
    readProfiles().default_tts_profile ||
    DEFAULT_TTS_PROFILE;
  const profileRec = resolveTtsProfileRecord(profileId);
  const routing = synthRoutingFromConfig(profileRec);
  const engine = routing.engine || profileRec.profile?.engine;
  const directive =
    params?.directive && typeof params.directive === "object"
      ? ensureCosyvoiceSpeakDirective(params.directive, params)
      : null;
  if (!engineSupportsWarm(engine)) {
    return {
      ok: true,
      skipped: true,
      engine,
      profile: profileId,
      message: `${engine || "engine"} does not require CosyVoice sidecar warm`,
    };
  }
  if (!shouldRunBundledSidecar(profileRec)) {
    return {
      ok: true,
      skipped: true,
      engine,
      profile: profileId,
      message: "Warm applies only to bundled CosyVoice2 profile",
    };
  }
  const sidecar = await ensureCosyvoiceSidecar(profileId);
  if (!sidecar.ok) {
    return sidecar;
  }
  const modelDir = resolveTtsModelDir(profileId);
  const endpoint = sidecar.sidecar_endpoint;
  if (cosyvoiceSidecarWarmed && !directiveHasCosyvoiceInput(directive)) {
    return {
      ok: true,
      already_warmed: true,
      warmed: true,
      engine: "cosyvoice2",
      model_dir: modelDir,
      sidecar_endpoint: endpoint,
      message: "CosyVoice2 sidecar already warmed",
    };
  }
  const health = await probeSidecarEndpoint(endpoint, modelDir);
  if (health.ok && health.warmed && !directiveHasCosyvoiceInput(directive)) {
    cosyvoiceSidecarWarmed = true;
    return {
      ok: true,
      already_warmed: true,
      warmed: true,
      engine: "cosyvoice2",
      model_dir: modelDir,
      sidecar_endpoint: endpoint,
      message: "CosyVoice2 sidecar already warmed",
    };
  }
  return runCosyvoiceWarmSerialized(
    profileId,
    endpoint,
    modelDir,
    directive,
    params?._oclive_resource_admission,
  );
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
  const personalityVoice = deriveVoiceStyleFromPersonality(loadCorePersonalityFromRole(rolePath));
  const synthProfile =
    roleVoice?.synth_profile ||
    pluginConfig?.tts_profile ||
    cfg.default_tts_profile ||
    DEFAULT_TTS_PROFILE;
  const baselineSpeed = Number(roleVoice?.speed ?? personalityVoice?.speed ?? 1.0) || 1.0;

  const refAudio = resolveRefAudio(rolePath, roleVoice, botEmotion);
  const refText = String(roleVoice?.ref_text || "").trim();
  const emoFromRole =
    roleVoice?.emo_text_template && typeof roleVoice.emo_text_template === "string"
      ? roleVoice.emo_text_template.replace("{tone}", RULES_V1_EMO_TEXT[botEmotion] || "")
      : personalityVoice?.emoTextTemplate
        ? personalityVoice.emoTextTemplate.replace("{tone}", RULES_V1_EMO_TEXT[botEmotion] || "")
      : "";
  const emoText =
    emoFromRole || RULES_V1_EMO_TEXT[botEmotion] || DEFAULT_COSYVOICE_EMO_TEXT;

  if (!directorId || directorId === "none") {
    const directive = {
      schema_version: 1,
      emotion_tag: botEmotion,
      speed: baselineSpeed,
      energy: roleVoice?.energy || personalityVoice?.energy || "normal",
      emo_text: emoText,
      synth_profile: synthProfile,
      ...(refAudio ? { ref_audio: refAudio } : {}),
      ...(refText ? { ref_text: refText } : {}),
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
    personalityVoice?.energy ||
    (botEmotion === "shy" || botEmotion === "sad" ? "soft" : "normal");
  const directive = {
    schema_version: 1,
    emotion_tag: botEmotion,
    speed,
    energy,
    emo_text: emoText,
    synth_profile: roleVoice?.synth_profile || synthProfile,
    ...(refAudio ? { ref_audio: refAudio } : {}),
    ...(refText ? { ref_text: refText } : {}),
  };
  if (!validateDirective(directive)) {
    return { ok: false, reason: "invalid_directive", director_profile: directorId };
  }
  return { ok: true, directive, director_profile: directorId };
}

function readAdapterPackMeta(src) {
  const direct = path.join(src, ADAPTER_PACK_FILENAME);
  if (fs.existsSync(direct)) {
    try {
      return JSON.parse(fs.readFileSync(direct, "utf8"));
    } catch {
      return null;
    }
  }
  return null;
}

function handleListTtsAdapters() {
  const root = path.join(userModelsRoot(), "tts_adapters");
  const adapters = [];
  if (!fs.existsSync(root)) {
    return { ok: true, adapters };
  }
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    const dir = path.join(root, entry.name);
    const pack = readAdapterPackMeta(dir);
    if (!pack) continue;
    adapters.push({
      adapter_id: pack.adapter_id || entry.name,
      label: pack.label || entry.name,
      api_style: pack.api_style || "",
      base_url: pack.base_url || "",
      path: dir,
    });
  }
  return { ok: true, adapters };
}

function handleImportTtsAdapter(params) {
  const src = String(params?.src_path || params?.path || "").trim();
  if (!src) {
    throw new Error("import_tts_adapter: src_path required");
  }
  if (!fs.existsSync(src)) {
    return { ok: false, reason: "src_not_found" };
  }
  const packMeta = readAdapterPackMeta(src);
  if (!packMeta) {
    return {
      ok: false,
      reason: "adapter_pack_missing",
      message: `Directory must contain ${ADAPTER_PACK_FILENAME}`,
    };
  }
  const adapterId = String(packMeta.adapter_id || path.basename(src)).trim();
  const dest = path.join(userModelsRoot(), "tts_adapters", adapterId);
  const stat = fs.statSync(src);
  if (stat.isDirectory()) {
    copyDirRecursive(src, dest);
  } else {
    return {
      ok: false,
      reason: "src_must_be_directory",
      message: "Import the folder containing tts_adapter_pack.json",
    };
  }
  return {
    ok: true,
    adapter_id: adapterId,
    dest,
    pack: packMeta,
  };
}

async function handleConfigUpdated(params) {
  let resourceTransition = null;
  if (params?.config && typeof params.config === "object") {
    const prev = pluginConfig;
    pluginConfig = params.config;
    const profileId =
      pluginConfig.tts_profile || readProfiles().default_tts_profile || DEFAULT_TTS_PROFILE;
    const profileRec = resolveTtsProfileRecord(profileId);
    const requestedTransition =
      params?.resource_transition &&
      typeof params.resource_transition === "object" &&
      params.resource_transition.adapter_id === COSYVOICE_RESOURCE_ADAPTER_ID &&
      params.resource_transition.operation === "unload";
    const leavingBundledRuntime =
      pluginConfig.tts_expansion_enabled !== true || !shouldRunBundledSidecar(profileRec);
    if (
      leavingBundledRuntime &&
      (requestedTransition || prev?.tts_expansion_enabled === true)
    ) {
      const releaseProfileId = requestedTransition
        ? String(params.resource_transition.runtime_profile_id || DEFAULT_TTS_PROFILE)
        : String(prev?.tts_profile || DEFAULT_TTS_PROFILE);
      resourceTransition = {
        adapter_id: COSYVOICE_RESOURCE_ADAPTER_ID,
        operation: "unload",
        ...(await releaseCosyvoiceSidecar(releaseProfileId)),
      };
    }
  }
  return {
    ok: true,
    ...(resourceTransition ? { resource_transition: resourceTransition } : {}),
  };
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
        else if (method === "voice.probe_tts") result = await handleProbeTts(params);
        else if (method === "voice.warm") result = await handleWarm(params);
        else if (method === "voice.list_model_packs") result = handleListModelPacks();
        else if (method === "voice.list_tts_adapters") result = handleListTtsAdapters();
        else if (method === "voice.import_tts_adapter") result = handleImportTtsAdapter(params);
        else if (method === "voice.build_directive") result = handleBuildDirective(params);
        else if (method === "voice.read_role_profile") result = handleReadRoleProfile(params);
        else if (method === "config_updated") result = await handleConfigUpdated(params);
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

process.on("SIGTERM", () => {
  stopCosyvoiceSidecar();
  server.close();
});
process.on("SIGINT", () => {
  stopCosyvoiceSidecar();
  server.close();
});
