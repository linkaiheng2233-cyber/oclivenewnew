import * as Vue from "vue";
import type { Component } from "vue";
import { readPluginAssetText } from "./tauri-api";

const SCHEME = "oclive-plugin://";

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

export type LoadPluginVueOptions = {
  /** 入口 `.vue` 已读入的源码（如安全扫描后），避免对同一文件二次 `read_plugin_asset_text`。 */
  preloadedEntrySource?: string;
};

/**
 * 从目录插件根编译并加载 `.vue`（运行时 `vue3-sfc-loader`）；失败返回 `null` 以便回退 iframe。
 */
export async function loadPluginVueComponent(
  pluginId: string,
  vueRel: string,
  opts?: LoadPluginVueOptions,
): Promise<Component | null> {
  const rel0 = vueRel.replace(/\\/g, "/").replace(/^\/+/, "");
  const entry = uri(pluginId, rel0);
  const pre = opts?.preloadedEntrySource;

  const moduleCache = Object.assign(Object.create(null), {
    vue: Vue,
  });

  const getFile = async (path: { toString(): string }) => {
    const p = String(path);
    let full: string;
    if (p.startsWith(SCHEME)) {
      full = p;
    } else {
      full = uri(pluginId, joinUnder(dirname(rel0), p));
    }
    const body = full.slice(SCHEME.length);
    const slash = body.indexOf("/");
    const pid = body.slice(0, slash);
    const rel = body.slice(slash + 1);
    if (pid !== pluginId) {
      throw new Error(`cross-plugin import denied: ${p}`);
    }
    if (pre !== undefined && pre.length > 0 && full === entry) {
      const text = pre;
      return {
        getContentData: (asBinary: boolean) =>
          asBinary
            ? Promise.resolve(new TextEncoder().encode(text).buffer)
            : Promise.resolve(text),
      };
    }
    const text = await readPluginAssetText(pid, rel);
    return {
      getContentData: (asBinary: boolean) =>
        asBinary
          ? Promise.resolve(new TextEncoder().encode(text).buffer)
          : Promise.resolve(text),
    };
  };

  try {
    const { loadModule } = await import("vue3-sfc-loader");
    const mod = await loadModule(entry, {
      moduleCache,
      getFile,
      addStyle(styleText: string) {
        const el = document.createElement("style");
        el.textContent = styleText;
        document.head.appendChild(el);
      },
    } as never);
    const m = mod as { default?: Component };
    return (m.default ?? (mod as Component)) ?? null;
  } catch (e) {
    console.warn("[loadPluginVueComponent]", pluginId, vueRel, e);
    return null;
  }
}
