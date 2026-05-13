import type { Component } from "vue";
import { compileScript, compileStyleAsync, compileTemplate, parse } from "@vue/compiler-sfc";
import { transform as sucraseTransform } from "sucrase";
import { readPluginAssetText } from "./tauri-api";

const SCHEME = "oclive-plugin://";

/** 目录插件 `.vue` 编译失败时的可读错误（供插槽 UI 展示）。 */
export class PluginVueCompileError extends Error {
  readonly pluginId: string;
  readonly componentPath: string;
  readonly friendlyMessage: string;
  readonly rawMessage: string;

  constructor(
    pluginId: string,
    componentPath: string,
    friendlyMessage: string,
    rawMessage: string,
  ) {
    super(friendlyMessage);
    this.name = "PluginVueCompileError";
    this.pluginId = pluginId;
    this.componentPath = componentPath;
    this.friendlyMessage = friendlyMessage;
    this.rawMessage = rawMessage;
  }
}

function uri(pluginId: string, rel: string): string {
  const r = rel.replace(/\\/g, "/").replace(/^\/+/, "");
  return `${SCHEME}${pluginId}/${r}`;
}

function dirname(rel: string): string {
  const i = rel.lastIndexOf("/");
  return i === -1 ? "" : rel.slice(0, i);
}

function joinUnder(baseDir: string, rel: string): string {
  const parts = `${baseDir}/${rel}`.split("/").filter(Boolean);
  const stack: string[] = [];
  for (const p of parts) {
    if (p === "..") stack.pop();
    else if (p !== ".") stack.push(p);
  }
  return stack.join("/");
}

function buildCompileError(
  pluginId: string,
  vueRel: string,
  err: unknown,
): PluginVueCompileError {
  const raw =
    err instanceof Error ? err.stack || err.message : String(err ?? "unknown error");
  const short = err instanceof Error ? err.message : String(err ?? "");
  const lineHint =
    short.match(/\((\d+),(\d+)\)|:(\d+):(\d+)|line\s*(\d+)/i)?.[0] ?? short.slice(0, 240);
  const friendly = `插件 ${pluginId} 的 Vue 组件编译失败，请检查语法。组件路径：${vueRel}。错误详情：${lineHint}`;
  return new PluginVueCompileError(pluginId, vueRel, friendly, raw);
}

/** 稳定 scopeId（与 compileScript / compileStyleAsync 共用）。 */
function scopeIdFor(pluginId: string, vueRel: string): string {
  const s = `${pluginId}\0${vueRel}`;
  let h = 2166136261 >>> 0;
  for (let i = 0; i < s.length; i += 1) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 16777619) >>> 0;
  }
  return `v${h.toString(16).padStart(8, "0")}`;
}

function stripTypeScript(source: string, filePath: string): string {
  try {
    return sucraseTransform(source, {
      transforms: ["typescript"],
      filePath,
    }).code;
  } catch (e) {
    throw new Error(e instanceof Error ? e.message : String(e));
  }
}

export type LoadPluginVueOptions = {
  /** 入口 `.vue` 已读入的源码（如安全扫描后），避免对同一文件二次 `read_plugin_asset_text`。 */
  preloadedEntrySource?: string;
};

/**
 * 从目录插件根编译并加载 `.vue`（`@vue/compiler-sfc` + sucrase，无 `vue-template-compiler` 依赖链）。
 * 编译失败抛出 {@link PluginVueCompileError}；读盘失败返回 `null` 以便回退 iframe。
 */
export async function loadPluginVueComponent(
  pluginId: string,
  vueRel: string,
  opts?: LoadPluginVueOptions,
): Promise<Component | null> {
  const rel0 = vueRel.replace(/\\/g, "/").replace(/^\/+/, "");
  const entry = uri(pluginId, rel0);
  const pre = opts?.preloadedEntrySource;

  const getFileText = async (fullUri: string): Promise<string> => {
    const body = fullUri.slice(SCHEME.length);
    const slash = body.indexOf("/");
    const pid = body.slice(0, slash);
    const rel = body.slice(slash + 1);
    if (pid !== pluginId) {
      throw new Error(`cross-plugin import denied: ${fullUri}`);
    }
    if (pre !== undefined && pre.length > 0 && fullUri === entry) {
      return pre;
    }
    return readPluginAssetText(pid, rel);
  };

  const addStyle = (styleText: string): void => {
    const el = document.createElement("style");
    el.textContent = styleText;
    document.head.appendChild(el);
  };

  let source: string;
  try {
    source = await getFileText(entry);
  } catch (e) {
    console.warn("[loadPluginVueComponent] read failed", pluginId, vueRel, e);
    return null;
  }

  try {
    const scopeId = scopeIdFor(pluginId, rel0);
    const { descriptor, errors } = parse(source, { filename: rel0 });
    const parseErr = [...(errors ?? []), ...(descriptor.errors ?? [])]
      .map((x) => (typeof x === "string" ? x : (x as { message?: string }).message ?? String(x)))
      .filter(Boolean);
    if (parseErr.length) {
      throw new Error(parseErr.join("\n"));
    }

    const hasScript = Boolean(descriptor.script || descriptor.scriptSetup);
    let moduleJs: string;

    if (hasScript) {
      const compiledScript = compileScript(descriptor, {
        id: scopeId,
        inlineTemplate: true,
      });
      const scriptErrs = (compiledScript as { errors?: { message: string }[] }).errors;
      if (scriptErrs?.length) {
        throw new Error(scriptErrs.map((x) => x.message).join("\n"));
      }
      moduleJs = stripTypeScript(compiledScript.content, `${rel0}.tsx`);
    } else if (descriptor.template) {
      const tmpl = compileTemplate({
        source: descriptor.template.content,
        id: scopeId,
        filename: rel0,
        compilerOptions: { bindingMetadata: {} },
      });
      if (tmpl.errors?.length) {
        throw new Error(tmpl.errors.map((e) => e.message).join("\n"));
      }
      const baseName = rel0.split("/").pop()?.replace(/\.vue$/i, "") || "Anonymous";
      moduleJs = `import { defineComponent } from "vue";\n${tmpl.code}\nexport default defineComponent({ name: ${JSON.stringify(
        baseName,
      )}, render });\n`;
    } else {
      throw new Error("SFC 缺少 <script> 与 <template>，无法编译。");
    }

    for (const style of descriptor.styles) {
      const res = await compileStyleAsync({
        source: style.content,
        filename: rel0,
        id: scopeId,
        scoped: style.scoped,
        trim: true,
      });
      if (res.errors?.length) {
        throw new Error(res.errors.map((e) => e.message).join("\n"));
      }
      if (res.code?.trim()) addStyle(res.code);
    }

    // Blob-module `import()` cannot resolve bare specifiers like `from "vue"`.
    // Resolve the app bundle's Vue ESM URL (Vite `?url`) and rewrite imports.
    const needsVueRewrite = /\bfrom\s*["']vue["']/.test(moduleJs);
    let vueModuleUrl = "";
    if (needsVueRewrite) {
      try {
        const mod = (await import("vue?url")) as { default?: string };
        const raw = mod.default ?? "";
        vueModuleUrl = raw ? new URL(raw, window.location.href).href : "";
      } catch {
        vueModuleUrl = "";
      }
      if (!vueModuleUrl) {
        throw new Error(
          "无法解析 Vue 模块地址（vue?url）。请确认以 Vite 构建/开发环境运行；否则目录插件 .vue 无法在浏览器中动态加载。",
        );
      }
      moduleJs = moduleJs.replace(/\bfrom\s*["']vue["']/g, `from ${JSON.stringify(vueModuleUrl)}`);
    }

    const blob = new Blob([moduleJs], { type: "text/javascript" });
    const url = URL.createObjectURL(blob);
    try {
      const mod = (await import(/* @vite-ignore */ url)) as { default?: Component };
      return (mod.default ?? (mod as unknown as Component)) ?? null;
    } finally {
      URL.revokeObjectURL(url);
    }
  } catch (e) {
    throw buildCompileError(pluginId, rel0, e);
  }
}
