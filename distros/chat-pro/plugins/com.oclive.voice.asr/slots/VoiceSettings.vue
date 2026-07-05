<script setup lang="ts">
import { inject, onMounted, ref } from "vue";

type OcliveApi = {
  invoke(command: string, params?: unknown): Promise<unknown>;
};

type ProfileRow = {
  id: string;
  label: string;
  engine: string;
  kind?: string;
  platform_ready?: boolean;
};

const PLUGIN_ID = "com.oclive.voice.asr";

const oclive = inject<OcliveApi | null>("oclive", null);
const asrProfiles = ref<ProfileRow[]>([]);
const ttsProfiles = ref<ProfileRow[]>([]);
const directorProfiles = ref<ProfileRow[]>([]);
const probe = ref<Record<string, unknown> | null>(null);
const errText = ref("");
const submitMode = ref<"send" | "fill">("send");
const autoTts = ref(false);
const asrProfile = ref("sherpa-paraformer-zh-small");
const ttsProfile = ref("sherpa-piper-zh");
const directorProfile = ref("rules-v1");
const importPath = ref("");
const importKind = ref<"asr" | "tts">("asr");
const saving = ref(false);

async function rpc(method: string, params: Record<string, unknown> = {}) {
  if (!oclive) throw new Error("oclive bridge missing");
  return oclive.invoke("plugin_rpc_invoke", { method, params });
}

async function loadConfig(): Promise<void> {
  if (!oclive) return;
  try {
    const ui = (await oclive.invoke("get_plugin_settings_ui", {
      pluginId: PLUGIN_ID,
    })) as { config?: Record<string, unknown> };
    const cfg = ui.config || {};
    submitMode.value = cfg.submit_mode === "fill" ? "fill" : "send";
    autoTts.value = cfg.auto_tts === true;
    if (typeof cfg.asr_profile === "string") asrProfile.value = cfg.asr_profile;
    if (typeof cfg.tts_profile === "string") ttsProfile.value = cfg.tts_profile;
    if (typeof cfg.director_profile === "string") {
      directorProfile.value = cfg.director_profile || "none";
    }
  } catch {
    /* optional */
  }
}

async function saveConfig(): Promise<void> {
  if (!oclive) return;
  saving.value = true;
  errText.value = "";
  try {
    await oclive.invoke("set_plugin_settings_config", {
      pluginId: PLUGIN_ID,
      config: {
        submit_mode: submitMode.value,
        auto_tts: autoTts.value,
        asr_profile: asrProfile.value,
        tts_profile: ttsProfile.value,
        director_profile: directorProfile.value,
      },
    });
    await reload();
  } catch (e) {
    errText.value = e instanceof Error ? e.message : String(e);
  } finally {
    saving.value = false;
  }
}

function byKind(rows: ProfileRow[], kind: string): ProfileRow[] {
  return rows.filter((p) => (p.kind || "asr") === kind);
}

async function reload(): Promise<void> {
  if (!oclive) return;
  errText.value = "";
  try {
    const list = (await rpc("voice.list_profiles", {})) as {
      profiles?: ProfileRow[];
    };
    const all = Array.isArray(list.profiles) ? list.profiles : [];
    asrProfiles.value = byKind(all, "asr");
    ttsProfiles.value = byKind(all, "tts");
    directorProfiles.value = byKind(all, "director");
    probe.value = (await rpc("voice.probe", { profile: asrProfile.value })) as Record<
      string,
      unknown
    >;
  } catch (e) {
    errText.value = e instanceof Error ? e.message : String(e);
  }
}

async function importModel(): Promise<void> {
  const src = importPath.value.trim();
  if (!src) {
    errText.value = "请填写模型目录路径";
    return;
  }
  errText.value = "";
  const profile = importKind.value === "tts" ? ttsProfile.value : asrProfile.value;
  try {
    const res = (await rpc("voice.import_model", {
      src_path: src,
      profile,
      kind: importKind.value,
    })) as { ok?: boolean; reason?: string; dest?: string };
    if (!res.ok) {
      errText.value = res.reason || "导入失败";
      return;
    }
    importPath.value = "";
    await reload();
  } catch (e) {
    errText.value = e instanceof Error ? e.message : String(e);
  }
}

onMounted(() => {
  void loadConfig().then(() => reload());
});
</script>

<template>
  <section class="panel voice-asr-settings" aria-label="语音识别设置">
    <h3 class="title">语音识别（voice.asr）</h3>
    <p class="lede">
      独立通道 · 不进六槽。Windows 首版：模型导入至
      <code>%APPDATA%/OCLive/models/asr/</code> 或 <code>models/tts/</code>。
    </p>

    <label class="field">
      <span class="label">ASR 档案</span>
      <select v-model="asrProfile" class="sel" @change="reload">
        <option v-for="p in asrProfiles" :key="p.id" :value="p.id">
          {{ p.label }} ({{ p.engine }})
        </option>
      </select>
    </label>

    <label class="field">
      <span class="label">TTS 发声 profile</span>
      <select v-model="ttsProfile" class="sel">
        <option v-for="p in ttsProfiles" :key="p.id" :value="p.id">
          {{ p.label }} ({{ p.engine }})
        </option>
      </select>
    </label>

    <label class="field">
      <span class="label">声音导演</span>
      <select v-model="directorProfile" class="sel">
        <option value="none">无（仅 synth）</option>
        <option v-for="p in directorProfiles" :key="p.id" :value="p.id">
          {{ p.label }}
        </option>
      </select>
      <span class="hint">Piper 忽略 emotion_tag，<code>speed</code> 可听出差异</span>
    </label>

    <label class="field">
      <span class="label">识别结果</span>
      <select v-model="submitMode" class="sel">
        <option value="send">直接发送</option>
        <option value="fill">填入输入框</option>
      </select>
    </label>

    <label class="field row">
      <input v-model="autoTts" type="checkbox" />
      <span>自动朗读回复（voice.speak · 键盘发送与语音 send 均生效）</span>
    </label>

    <label class="field">
      <span class="label">导入模型</span>
      <select v-model="importKind" class="sel narrow">
        <option value="asr">ASR</option>
        <option value="tts">TTS</option>
      </select>
      <input
        v-model="importPath"
        class="inp"
        type="text"
        placeholder="D:\models\sherpa-paraformer-zh-small"
      />
      <button type="button" class="btn" :disabled="!oclive" @click="importModel">导入</button>
    </label>

    <ul v-if="asrProfiles.length" class="list">
      <li v-for="p in asrProfiles" :key="p.id">
        <strong>{{ p.label }}</strong>
        <span class="meta">
          {{ p.id }} · {{ p.engine }}
          <template v-if="p.platform_ready === false"> · 本平台暂未支持</template>
        </span>
      </li>
    </ul>

    <pre v-if="probe" class="probe">{{ JSON.stringify(probe, null, 2) }}</pre>
    <p v-if="errText" class="err">{{ errText }}</p>

    <div class="actions">
      <button type="button" class="btn" :disabled="!oclive || saving" @click="saveConfig">
        {{ saving ? "保存中…" : "保存设置" }}
      </button>
      <button type="button" class="btn" :disabled="!oclive" @click="reload">重新检测</button>
    </div>
  </section>
</template>

<style scoped>
.panel {
  font-family: var(--font-ui);
  font-size: 0.8125rem;
  line-height: 1.45;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.title {
  margin: 0;
  font-size: 0.9375rem;
}
.lede {
  margin: 0;
  color: var(--text-secondary, #666);
  font-size: 0.75rem;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}
.field.row {
  flex-direction: row;
  align-items: center;
  gap: 0.5rem;
}
.label {
  font-size: 0.75rem;
  color: var(--text-secondary, #666);
}
.hint {
  font-size: 0.6875rem;
  color: var(--text-secondary, #666);
}
.sel,
.inp {
  min-height: 1.875rem;
  padding: 0.25rem 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--border-light, #ccc);
  font-size: 0.8125rem;
}
.sel.narrow {
  max-width: 6rem;
}
.list {
  margin: 0;
  padding-left: 1.1rem;
}
.meta {
  display: block;
  font-size: 0.6875rem;
  color: var(--text-secondary, #666);
}
.probe {
  margin: 0;
  padding: 0.5rem;
  font-size: 0.6875rem;
  background: var(--bg-elevated, #f5f5f5);
  border-radius: 6px;
  overflow: auto;
  max-height: 8rem;
}
.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}
.btn {
  align-self: flex-start;
  min-height: 1.875rem;
  padding: 0.25rem 0.625rem;
  border-radius: 6px;
  border: 1px solid var(--border-light, #ccc);
  cursor: pointer;
}
.err {
  margin: 0;
  color: var(--error, #c00);
  font-size: 0.75rem;
}
</style>
