<script setup lang="ts">
import { inject, onMounted, ref } from "vue";

type OcliveApi = {
  invoke(command: string, params?: unknown): Promise<unknown>;
  events: {
    emit(event: string, data?: unknown): void;
  };
};

type ProfileRow = {
  id: string;
  label: string;
  engine: string;
  kind?: string;
  platform_ready?: boolean;
  requires_pack?: string;
  min_vram_gb_recommended?: number | null;
  synth_provider?: string;
};

type ModelPackRow = {
  pack_id: string;
  profile_id: string;
  label: string;
  engine: string;
  min_vram_gb_recommended?: number | null;
  installed: boolean;
  model_dir: string;
  download_url?: string;
};

const PLUGIN_ID = "com.oclive.voice.asr";
const EVT_CONFIG_UPDATED = "com.oclive.voice.asr:config-updated";
const DEFAULT_TTS = "bundled-cosyvoice2-zh";

const oclive = inject<OcliveApi | null>("oclive", null);
const asrProfiles = ref<ProfileRow[]>([]);
const ttsProfiles = ref<ProfileRow[]>([]);
const directorProfiles = ref<ProfileRow[]>([]);
const modelPacks = ref<ModelPackRow[]>([]);
const asrProbe = ref<Record<string, unknown> | null>(null);
const ttsProbe = ref<Record<string, unknown> | null>(null);
const errText = ref("");
const submitMode = ref<"send" | "fill">("send");
const ttsExpansionEnabled = ref(false);
const autoTts = ref(false);
const asrProfile = ref("sherpa-paraformer-zh-small");
const ttsProfile = ref(DEFAULT_TTS);
const directorProfile = ref("rules-v1");
const synthProvider = ref<"bundled" | "local_http" | "cloud">("bundled");
const localSynthEndpoint = ref("http://127.0.0.1:50000");
const cloudTtsUrl = ref("");
const cloudTtsToken = ref("");
const cloudTtsVoiceId = ref("");
const cloudTtsModel = ref("tts-1");
const importPath = ref("");
const importKind = ref<"asr" | "tts">("asr");
const saving = ref(false);
const warming = ref(false);

async function rpc(method: string, params: Record<string, unknown> = {}) {
  if (!oclive) throw new Error("oclive bridge missing");
  return oclive.invoke("plugin_rpc_invoke", { method, params });
}

async function pushConfigToSidecar(): Promise<void> {
  await rpc("config_updated", {
    config: {
      submit_mode: submitMode.value,
      tts_expansion_enabled: ttsExpansionEnabled.value,
      auto_tts: autoTts.value,
      asr_profile: asrProfile.value,
      tts_profile: ttsProfile.value,
      director_profile: directorProfile.value,
      synth_provider: synthProvider.value,
      local_synth_endpoint: localSynthEndpoint.value,
      cloud_tts_url: cloudTtsUrl.value,
      cloud_tts_token: cloudTtsToken.value,
      cloud_tts_voice_id: cloudTtsVoiceId.value,
      cloud_tts_model: cloudTtsModel.value,
    },
  });
}

async function loadConfig(): Promise<void> {
  if (!oclive) return;
  try {
    const ui = (await oclive.invoke("get_plugin_settings_ui", {
      pluginId: PLUGIN_ID,
    })) as { config?: Record<string, unknown> };
    const cfg = ui.config || {};
    submitMode.value = cfg.submit_mode === "fill" ? "fill" : "send";
    ttsExpansionEnabled.value = cfg.tts_expansion_enabled === true;
    autoTts.value = cfg.auto_tts === true;
    if (typeof cfg.asr_profile === "string") asrProfile.value = cfg.asr_profile;
    if (typeof cfg.tts_profile === "string") ttsProfile.value = cfg.tts_profile;
    if (typeof cfg.director_profile === "string") {
      directorProfile.value = cfg.director_profile || "none";
    }
    if (cfg.synth_provider === "local_http" || cfg.synth_provider === "cloud" || cfg.synth_provider === "bundled") {
      synthProvider.value = cfg.synth_provider;
    }
    if (typeof cfg.local_synth_endpoint === "string" && cfg.local_synth_endpoint.trim()) {
      localSynthEndpoint.value = cfg.local_synth_endpoint.trim();
    }
    if (typeof cfg.cloud_tts_url === "string") cloudTtsUrl.value = cfg.cloud_tts_url;
    if (typeof cfg.cloud_tts_token === "string") cloudTtsToken.value = cfg.cloud_tts_token;
    if (typeof cfg.cloud_tts_voice_id === "string") cloudTtsVoiceId.value = cfg.cloud_tts_voice_id;
    if (typeof cfg.cloud_tts_model === "string" && cfg.cloud_tts_model.trim()) {
      cloudTtsModel.value = cfg.cloud_tts_model.trim();
    }
    await pushConfigToSidecar();
  } catch {
    /* optional */
  }
}

async function saveConfig(): Promise<void> {
  if (!oclive) return;
  saving.value = true;
  errText.value = "";
  try {
    const config = {
      submit_mode: submitMode.value,
      tts_expansion_enabled: ttsExpansionEnabled.value,
      auto_tts: ttsExpansionEnabled.value ? autoTts.value : false,
      asr_profile: asrProfile.value,
      tts_profile: ttsProfile.value,
      director_profile: directorProfile.value,
      synth_provider: synthProvider.value,
      local_synth_endpoint: localSynthEndpoint.value,
      cloud_tts_url: cloudTtsUrl.value,
      cloud_tts_token: cloudTtsToken.value,
      cloud_tts_voice_id: cloudTtsVoiceId.value,
      cloud_tts_model: cloudTtsModel.value,
    };
    await oclive.invoke("set_plugin_settings_config", {
      pluginId: PLUGIN_ID,
      config,
    });
    await pushConfigToSidecar();
    oclive.events.emit(EVT_CONFIG_UPDATED, {});
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
    const packs = (await rpc("voice.list_model_packs", {})) as {
      packs?: ModelPackRow[];
    };
    modelPacks.value = Array.isArray(packs.packs) ? packs.packs : [];
    asrProbe.value = (await rpc("voice.probe", { profile: asrProfile.value })) as Record<
      string,
      unknown
    >;
    if (ttsExpansionEnabled.value) {
      ttsProbe.value = (await rpc("voice.probe_tts", { profile: ttsProfile.value })) as Record<
        string,
        unknown
      >;
    } else {
      ttsProbe.value = null;
    }
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
    })) as { ok?: boolean; reason?: string; dest?: string; message?: string };
    if (!res.ok) {
      errText.value = res.message || res.reason || "导入失败";
      return;
    }
    importPath.value = "";
    await reload();
  } catch (e) {
    errText.value = e instanceof Error ? e.message : String(e);
  }
}

async function warmTts(): Promise<void> {
  warming.value = true;
  errText.value = "";
  try {
    const res = (await rpc("voice.warm", { profile: ttsProfile.value })) as {
      ok?: boolean;
      reason?: string;
      message?: string;
    };
    if (!res.ok) {
      errText.value = res.message || res.reason || "预热失败";
    }
    await reload();
  } catch (e) {
    errText.value = e instanceof Error ? e.message : String(e);
  } finally {
    warming.value = false;
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
      基础包：文字聊天 + 按住说话（ASR）。模型导入至
      <code>%APPDATA%/OCLive/models/asr/</code>。
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
      <span class="label">识别结果</span>
      <select v-model="submitMode" class="sel">
        <option value="send">直接发送</option>
        <option value="fill">填入输入框</option>
      </select>
    </label>

    <label class="field">
      <span class="label">导入 ASR 模型</span>
      <input
        v-model="importPath"
        class="inp"
        type="text"
        placeholder="D:\models\sherpa-paraformer-zh-small"
      />
      <button
        type="button"
        class="btn"
        :disabled="!oclive"
        @click="
          importKind = 'asr';
          importModel();
        "
      >
        导入 ASR
      </button>
    </label>

    <details v-if="asrProbe" class="probe-details">
      <summary>ASR 环境检测</summary>
      <pre class="probe">{{ JSON.stringify(asrProbe, null, 2) }}</pre>
    </details>

    <hr class="sep" />

    <h3 class="title">语音扩展（情感 TTS · 可选）</h3>
    <p class="lede">
      默认关闭。开启后使用 CosyVoice2 本地情感发声；需自备 GPU 与模型包（约 2–4GB）。
      不为发声订阅，算力与模型由用户自担。
    </p>

    <label class="field row">
      <input v-model="ttsExpansionEnabled" type="checkbox" @change="reload" />
      <span>启用语音扩展</span>
    </label>

    <template v-if="ttsExpansionEnabled">
      <label class="field row">
        <input v-model="autoTts" type="checkbox" />
        <span>自动朗读回复（整段 reply 完成后 speak）</span>
      </label>

      <label class="field">
        <span class="label">发声提供方</span>
        <select v-model="synthProvider" class="sel">
          <option value="bundled">本地 bundled（CosyVoice2 侧车）</option>
          <option value="local_http">本地 HTTP（自建 GSVI / CosyVoice）</option>
          <option value="cloud">云端（自填 API · 不经 OCLive 计费）</option>
        </select>
      </label>

      <label v-if="synthProvider === 'local_http'" class="field">
        <span class="label">本地 HTTP endpoint</span>
        <input v-model="localSynthEndpoint" class="inp" type="text" />
      </label>

      <template v-if="synthProvider === 'cloud'">
        <label class="field">
          <span class="label">云端 TTS URL</span>
          <input v-model="cloudTtsUrl" class="inp" type="text" placeholder="https://api.openai.com" />
        </label>
        <label class="field">
          <span class="label">API Token</span>
          <input v-model="cloudTtsToken" class="inp" type="password" autocomplete="off" />
        </label>
        <label class="field">
          <span class="label">Voice ID</span>
          <input v-model="cloudTtsVoiceId" class="inp" type="text" placeholder="alloy" />
        </label>
        <label class="field">
          <span class="label">Model</span>
          <input v-model="cloudTtsModel" class="inp" type="text" />
        </label>
        <p class="hint">也可选 profile <code>edge-tts-zh</code>（无 key，在线）</p>
      </template>

      <label class="field">
        <span class="label">TTS profile</span>
        <select v-model="ttsProfile" class="sel" @change="reload">
          <option v-for="p in ttsProfiles" :key="p.id" :value="p.id">
            {{ p.label }}
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
        <span class="hint">rules-v1 产出 emo_text · 角色包 ref 映射音色</span>
      </label>

      <div v-if="modelPacks.length" class="pack-list">
        <p class="label">模型包（DLC）</p>
        <ul class="list">
          <li v-for="pack in modelPacks" :key="pack.pack_id">
            <strong>{{ pack.label }}</strong>
            <span class="meta">
              {{ pack.pack_id }}
              · {{ pack.installed ? "已安装" : "未安装" }}
              <template v-if="pack.min_vram_gb_recommended">
                · 推荐 {{ pack.min_vram_gb_recommended }}GB+ 显存
              </template>
            </span>
          </li>
        </ul>
      </div>

      <label class="field">
        <span class="label">导入 TTS 模型包目录</span>
        <input
          v-model="importPath"
          class="inp"
          type="text"
          placeholder="解压后的 cosyvoice2-0.5b 目录（含 voice_model_pack.json）"
        />
        <button
          type="button"
          class="btn"
          :disabled="!oclive"
          @click="
            importKind = 'tts';
            importModel();
          "
        >
          导入 TTS 模型
        </button>
      </label>

      <div class="actions inline">
        <button type="button" class="btn" :disabled="!oclive || warming" @click="warmTts">
          {{ warming ? "预热中…" : "预热 TTS 侧车" }}
        </button>
      </div>

      <details v-if="ttsProbe" class="probe-details">
        <summary>TTS 环境检测（voice.probe_tts）</summary>
        <pre class="probe">{{ JSON.stringify(ttsProbe, null, 2) }}</pre>
      </details>
    </template>

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
.sep {
  border: none;
  border-top: 1px solid color-mix(in srgb, var(--border-light, #ccc) 65%, transparent);
  margin: 0.25rem 0;
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
.probe-details {
  margin: 0;
}
.probe-details summary {
  cursor: pointer;
  font-size: 0.75rem;
  color: var(--text-secondary, #666);
  user-select: none;
}
.probe-details[open] summary {
  margin-bottom: 0.25rem;
}
.pack-list .list {
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
  position: sticky;
  bottom: 0;
  z-index: 1;
  margin-top: 0.25rem;
  padding-top: 0.5rem;
  background: inherit;
  border-top: 1px solid color-mix(in srgb, var(--border-light, #ccc) 65%, transparent);
}
.actions.inline {
  position: static;
  border-top: none;
  padding-top: 0;
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
