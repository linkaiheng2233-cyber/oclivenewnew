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

const EVT_SUBMIT = "com.oclive.voice.asr:submit";
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

let mediaStream: MediaStream | null = null;
let mediaRecorder: MediaRecorder | null = null;
let audioChunks: Blob[] = [];
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

function blobToBase64(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onloadend = () => {
      const raw = String(reader.result || "");
      const comma = raw.indexOf(",");
      resolve(comma >= 0 ? raw.slice(comma + 1) : raw);
    };
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(blob);
  });
}

async function ensureMic(): Promise<MediaStream> {
  if (!navigator.mediaDevices?.getUserMedia) {
    throw new Error("此环境不支持麦克风（需 HTTPS 或 Tauri WebView）");
  }
  return navigator.mediaDevices.getUserMedia({ audio: true });
}

async function transcribeBlob(blob: Blob): Promise<void> {
  if (!oclive || busy.value) return;
  busy.value = true;
  errText.value = "";
  try {
    const audioBase64 = await blobToBase64(blob);
    const res = (await rpc("voice.transcribe", {
      profile: asrProfile.value,
      audio_base64: audioBase64,
      sample_rate: 16000,
    })) as { ok?: boolean; text?: string; reason?: string; message?: string };
    const text = String(res.text || "").trim();
    if (!res.ok || !text) {
      errText.value = res.reason || res.message || "识别无结果";
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

async function playTts(text: string): Promise<void> {
  const cleaned = text.trim();
  if (!cleaned || !oclive) return;
  try {
    const res = (await rpc("voice.speak", { text: cleaned })) as {
      ok?: boolean;
      audio_base64?: string;
      reason?: string;
    };
    if (!res.ok || !res.audio_base64) return;
    if (!audioEl) audioEl = new Audio();
    audioEl.src = `data:audio/wav;base64,${res.audio_base64}`;
    await audioEl.play();
  } catch {
    /* TTS failure is silent per plan */
  }
}

function onMessageSent(payload: unknown): void {
  if (!pendingTts && !autoTts.value) return;
  const reply = (payload as { reply?: string } | null)?.reply?.trim();
  pendingTts = false;
  if (!reply) return;
  void playTts(reply);
}

async function startRecording(): Promise<void> {
  if (!oclive || busy.value || recording.value) return;
  errText.value = "";
  try {
    mediaStream = await ensureMic();
    audioChunks = [];
    const mime =
      MediaRecorder.isTypeSupported("audio/webm;codecs=opus")
        ? "audio/webm;codecs=opus"
        : MediaRecorder.isTypeSupported("audio/webm")
          ? "audio/webm"
          : "";
    mediaRecorder = mime
      ? new MediaRecorder(mediaStream, { mimeType: mime })
      : new MediaRecorder(mediaStream);
    mediaRecorder.ondataavailable = (ev) => {
      if (ev.data.size > 0) audioChunks.push(ev.data);
    };
    mediaRecorder.onstop = () => {
      const blob = new Blob(audioChunks, {
        type: mediaRecorder?.mimeType || "audio/webm",
      });
      void transcribeBlob(blob);
      mediaStream?.getTracks().forEach((t) => t.stop());
      mediaStream = null;
      mediaRecorder = null;
      recording.value = false;
    };
    mediaRecorder.start();
    recording.value = true;
    statusText.value = "录音中…";
  } catch (e) {
    errText.value = e instanceof Error ? e.message : String(e);
    recording.value = false;
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
});

onBeforeUnmount(() => {
  oclive?.events.off("oclive:message:sent", onMessageSent);
  stopRecording();
  mediaStream?.getTracks().forEach((t) => t.stop());
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
