<script setup lang="ts">
import { inject, onBeforeUnmount, onMounted, ref } from "vue";

type OcliveApi = {
  invoke(command: string, params?: unknown): Promise<unknown>;
  events: {
    emit(event: string, data?: unknown): void;
    on(event: string, handler: (data: unknown) => void): void;
    off(event: string, handler: (data: unknown) => void): void;
  };
};

// Inlined from slots/audioCapture.ts — vue3-sfc-loader cannot load sibling .ts in ui_slots.
const TARGET_SAMPLE_RATE = 16000;
const MIN_RECORD_MS = 350;
const MIC_CONSTRAINTS: MediaTrackConstraints = {
  echoCancellation: true,
  noiseSuppression: true,
  autoGainControl: true,
  channelCount: 1,
};

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  const chunk = 0x8000;
  for (let i = 0; i < bytes.length; i += chunk) {
    binary += String.fromCharCode(...bytes.subarray(i, i + chunk));
  }
  return btoa(binary);
}

function encodeWavPcm16(samples: Float32Array, sampleRate: number): Uint8Array {
  const dataSize = samples.length * 2;
  const buffer = new ArrayBuffer(44 + dataSize);
  const view = new DataView(buffer);
  const writeAscii = (offset: number, text: string) => {
    for (let i = 0; i < text.length; i += 1)
      view.setUint8(offset + i, text.charCodeAt(i));
  };
  writeAscii(0, "RIFF");
  view.setUint32(4, 36 + dataSize, true);
  writeAscii(8, "WAVE");
  writeAscii(12, "fmt ");
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, 1, true);
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, sampleRate * 2, true);
  view.setUint16(32, 2, true);
  view.setUint16(34, 16, true);
  writeAscii(36, "data");
  view.setUint32(40, dataSize, true);
  let offset = 44;
  for (let i = 0; i < samples.length; i += 1) {
    const clamped = Math.max(-1, Math.min(1, samples[i]));
    view.setInt16(offset, clamped < 0 ? clamped * 0x8000 : clamped * 0x7fff, true);
    offset += 2;
  }
  return new Uint8Array(buffer);
}

function mixToMono(decoded: AudioBuffer): Float32Array {
  const length = decoded.length;
  const mono = new Float32Array(length);
  const ch0 = decoded.getChannelData(0);
  if (decoded.numberOfChannels === 1) {
    mono.set(ch0);
    return mono;
  }
  for (let i = 0; i < length; i += 1) {
    let sum = ch0[i];
    for (let c = 1; c < decoded.numberOfChannels; c += 1)
      sum += decoded.getChannelData(c)[i];
    mono[i] = sum / decoded.numberOfChannels;
  }
  return mono;
}

async function resampleTo16kMono(
  mono: Float32Array,
  srcRate: number,
  durationSec: number,
): Promise<Float32Array> {
  const offline = new OfflineAudioContext(
    1,
    Math.max(1, Math.ceil(durationSec * TARGET_SAMPLE_RATE)),
    TARGET_SAMPLE_RATE,
  );
  const monoBuffer = offline.createBuffer(1, mono.length, srcRate);
  monoBuffer.copyToChannel(mono, 0);
  const source = offline.createBufferSource();
  source.buffer = monoBuffer;
  source.connect(offline.destination);
  source.start(0);
  const rendered = await offline.startRendering();
  return rendered.getChannelData(0);
}

async function blobToWav16kMonoBase64(blob: Blob): Promise<string> {
  if (!blob.size) throw new Error("录音为空");
  const arrayBuffer = await blob.arrayBuffer();
  if (arrayBuffer.byteLength >= 4) {
    const head = new Uint8Array(arrayBuffer, 0, 4);
    if (head[0] === 0x52 && head[1] === 0x49 && head[2] === 0x46 && head[3] === 0x46)
      return bytesToBase64(new Uint8Array(arrayBuffer));
  }
  const ctx = new AudioContext();
  try {
    const decoded = await ctx.decodeAudioData(arrayBuffer.slice(0));
    if (decoded.duration * 1000 < MIN_RECORD_MS)
      throw new Error("录音太短，请按住多说一会");
    const mono = mixToMono(decoded);
    const pcm = await resampleTo16kMono(mono, decoded.sampleRate, decoded.duration);
    return bytesToBase64(encodeWavPcm16(pcm, TARGET_SAMPLE_RATE));
  } finally {
    await ctx.close();
  }
}

function pickMediaRecorderMime(): string {
  const candidates = [
    "audio/webm;codecs=opus",
    "audio/webm",
    "audio/ogg;codecs=opus",
    "audio/mp4",
  ];
  for (const mime of candidates) {
    if (MediaRecorder.isTypeSupported(mime)) return mime;
  }
  return "";
}
const EVT_SUBMIT = "com.oclive.voice.asr:submit";
const EVT_HOLD = "com.oclive.voice.asr:hold";
const PLUGIN_ID = "com.oclive.voice.asr";

const oclive = inject<OcliveApi | null>("oclive", null);
const ready = ref(false);
const statusText = ref("");
const busy = ref(false);
const recording = ref(false);
const errText = ref("");
const submitMode = ref<"send" | "fill">("send");
const autoTts = ref(false);
const asrProfile = ref("sherpa-paraformer-zh-small");
const ttsProfile = ref("sherpa-piper-zh");
const directorProfile = ref("rules-v1");

let mediaStream: MediaStream | null = null;
let mediaRecorder: MediaRecorder | null = null;
let audioChunks: Blob[] = [];
let recordStartedAt = 0;
let pendingTts = false;
let audioEl: HTMLAudioElement | null = null;

async function rpc(method: string, params: Record<string, unknown> = {}) {
  if (!oclive) throw new Error("oclive bridge missing");
  return oclive.invoke("plugin_rpc_invoke", { method, params });
}

async function loadPluginConfig(): Promise<void> {
  if (!oclive) return;
  try {
    const ui = (await oclive.invoke("get_plugin_settings_ui", {
      pluginId: PLUGIN_ID,
    })) as { config?: Record<string, unknown> };
    const cfg = ui.config || {};
    submitMode.value = cfg.submit_mode === "fill" ? "fill" : "send";
    autoTts.value = cfg.auto_tts === true;
    if (typeof cfg.asr_profile === "string" && cfg.asr_profile.trim()) {
      asrProfile.value = cfg.asr_profile.trim();
    }
    if (typeof cfg.tts_profile === "string" && cfg.tts_profile.trim()) {
      ttsProfile.value = cfg.tts_profile.trim();
    }
    if (typeof cfg.director_profile === "string") {
      directorProfile.value = cfg.director_profile.trim() || "none";
    }
  } catch {
    /* settings optional */
  }
}

async function refreshProbe(): Promise<void> {
  if (!oclive) return;
  try {
    const probe = (await rpc("voice.probe", { profile: asrProfile.value })) as {
      ok?: boolean;
      message?: string;
      reason?: string;
    };
    ready.value = probe.ok === true;
    statusText.value =
      probe.message || (probe.ok ? "就绪" : probe.reason || "未就绪");
    errText.value = "";
  } catch (e) {
    ready.value = false;
    statusText.value = "";
    errText.value = e instanceof Error ? e.message : String(e);
  }
}

async function ensureMic(): Promise<MediaStream> {
  if (!navigator.mediaDevices?.getUserMedia) {
    throw new Error("此环境不支持麦克风（需 HTTPS 或 Tauri WebView）");
  }
  return navigator.mediaDevices.getUserMedia({ audio: MIC_CONSTRAINTS });
}

async function transcribeBlob(blob: Blob): Promise<void> {
  if (!oclive || busy.value) return;
  busy.value = true;
  errText.value = "";
  try {
    const audioBase64 = await blobToWav16kMonoBase64(blob);
    const res = (await rpc("voice.transcribe", {
      profile: asrProfile.value,
      audio_base64: audioBase64,
    })) as { ok?: boolean; text?: string; reason?: string; message?: string };
    const text = String(res.text || "").trim();
    if (!res.ok || !text) {
      const hint =
        res.reason === "audio_too_quiet"
          ? "声音太小或未检测到语音，请靠近麦克风再说"
          : res.reason === "bad_audio_format"
            ? "音频格式无法识别，请重试"
            : res.reason || res.message || "识别无结果";
      errText.value = hint;
      return;
    }
    if (submitMode.value === "fill") {
      oclive.events.emit(EVT_SUBMIT, { text, mode: "fill" });
    } else {
      pendingTts = autoTts.value;
      oclive.events.emit(EVT_SUBMIT, { text, mode: "send" });
    }
  } catch (e) {
    errText.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}

async function resolveRolePath(roleId: string): Promise<string> {
  if (!oclive || !roleId.trim()) return "";
  try {
    const res = (await oclive.invoke("get_role_pack_path", { roleId: roleId.trim() })) as {
      role_path?: string;
    };
    return String(res.role_path || "").trim();
  } catch {
    return "";
  }
}

async function playTts(text: string, botEmotion?: string, roleId?: string): Promise<void> {
  const cleaned = text.trim();
  if (!cleaned || !oclive) return;
  try {
    let directive: Record<string, unknown> | undefined;
    const director = directorProfile.value.trim();
    if (director && director !== "none") {
      const rolePath = roleId ? await resolveRolePath(roleId) : "";
      const built = (await rpc("voice.build_directive", {
        profile: director,
        bot_emotion: botEmotion || "neutral",
        role_path: rolePath,
      })) as { ok?: boolean; directive?: Record<string, unknown> };
      if (built.ok && built.directive) directive = built.directive;
    }
    const profile =
      (directive?.synth_profile as string | undefined) || ttsProfile.value;
    const res = (await rpc("voice.speak", {
      text: cleaned,
      profile,
      directive,
    })) as {
      ok?: boolean;
      audio_base64?: string;
      audio_mime?: string;
      reason?: string;
    };
    if (!res.ok || !res.audio_base64) return;
    if (!audioEl) audioEl = new Audio();
    const mime = res.audio_mime || "audio/wav";
    audioEl.src = `data:${mime};base64,${res.audio_base64}`;
    await audioEl.play();
  } catch {
    /* TTS failure is silent per plan */
  }
}

function onMessageSent(payload: unknown): void {
  if (!pendingTts && !autoTts.value) return;
  const data = payload as {
    reply?: string;
    bot_emotion?: string;
    role_id?: string;
  } | null;
  const reply = data?.reply?.trim();
  pendingTts = false;
  if (!reply) return;
  void playTts(reply, data?.bot_emotion, data?.role_id);
}

function onHoldEvent(payload: unknown): void {
  const phase = (payload as { phase?: string } | null)?.phase;
  if (phase === "start") {
    void startRecording();
    return;
  }
  if (phase === "stop") {
    stopRecording();
  }
}

function cleanupMic(): void {
  mediaStream?.getTracks().forEach((t) => t.stop());
  mediaStream = null;
  mediaRecorder = null;
  recording.value = false;
}

async function startRecording(): Promise<void> {
  if (!oclive || busy.value || recording.value) return;
  errText.value = "";
  try {
    mediaStream = await ensureMic();
    audioChunks = [];
    recordStartedAt = Date.now();
    const mime = pickMediaRecorderMime();
    mediaRecorder = mime
      ? new MediaRecorder(mediaStream, { mimeType: mime })
      : new MediaRecorder(mediaStream);
    mediaRecorder.ondataavailable = (ev) => {
      if (ev.data.size > 0) audioChunks.push(ev.data);
    };
    mediaRecorder.onstop = () => {
      const elapsed = Date.now() - recordStartedAt;
      const blob = new Blob(audioChunks, {
        type: mediaRecorder?.mimeType || "audio/webm",
      });
      cleanupMic();
      if (elapsed < MIN_RECORD_MS) {
        errText.value = "录音太短，请按住多说一会";
        return;
      }
      void transcribeBlob(blob);
    };
    mediaRecorder.start();
    recording.value = true;
    statusText.value = "录音中…";
  } catch (e) {
    errText.value = e instanceof Error ? e.message : String(e);
    cleanupMic();
  }
}

function stopRecording(): void {
  if (mediaRecorder && recording.value) {
    mediaRecorder.stop();
  }
}

function onPointerDown(ev: PointerEvent): void {
  if ((ev.button !== 0 && ev.button !== -1) || !oclive || busy.value) return;
  (ev.currentTarget as HTMLElement)?.setPointerCapture?.(ev.pointerId);
  void startRecording();
}

function onPointerUp(ev: PointerEvent): void {
  (ev.currentTarget as HTMLElement)?.releasePointerCapture?.(ev.pointerId);
  stopRecording();
}

onMounted(() => {
  void loadPluginConfig().then(() => refreshProbe());
  oclive?.events.on("oclive:message:sent", onMessageSent);
  oclive?.events.on(EVT_HOLD, onHoldEvent);
});

onBeforeUnmount(() => {
  oclive?.events.off("oclive:message:sent", onMessageSent);
  oclive?.events.off(EVT_HOLD, onHoldEvent);
  stopRecording();
  cleanupMic();
});
</script>

<template>
  <section class="voice-bar" aria-label="语音输入">
    <button
      type="button"
      class="mic-btn"
      :class="{ recording }"
      :disabled="!oclive || busy"
      :title="statusText || '按住说话'"
      @pointerdown.prevent="onPointerDown"
      @pointerup.prevent="onPointerUp"
      @pointercancel.prevent="stopRecording"
      @pointerleave="stopRecording"
    >
      {{ busy ? "识别中…" : recording ? "录音中…" : "🎤 按住说话" }}
    </button>
    <span v-if="statusText && !errText && !recording" class="hint">{{ statusText }}</span>
    <span v-if="errText" class="err" :title="errText">{{ errText }}</span>
  </section>
</template>

<style scoped>
.voice-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.375rem;
  font-family: var(--font-ui);
  font-size: 0.75rem;
}
.mic-btn {
  min-height: 1.875rem;
  padding: 0.2rem 0.55rem;
  border-radius: var(--radius-btn, 6px);
  border: 1px solid var(--border-light, #ccc);
  background: var(--bg-primary, #fff);
  cursor: pointer;
  touch-action: none;
  user-select: none;
}
.mic-btn.recording {
  border-color: var(--error, #c00);
  background: color-mix(in srgb, var(--error, #c00) 12%, var(--bg-primary, #fff));
}
.mic-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.hint {
  color: var(--text-secondary, #666);
  max-width: 14rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.err {
  color: var(--error, #c00);
  max-width: 16rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
