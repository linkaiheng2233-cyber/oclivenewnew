/**
 * Vue 插槽源码静态扫描（黑名单）。
 * 扩展规则时：在 `scanScriptAst` 的 `simple(...)` 回调中增加分支，并保持文案与 `ScanResult.warnings` 可读。
 */
import * as acorn from "acorn";
import { simple } from "acorn-walk";

export interface ScanResult {
  warnings: string[];
}

function extractScriptBodies(sfc: string): string[] {
  const out: string[] = [];
  const re = /<script\b[^>]*>([\s\S]*?)<\/script>/gi;
  let m: RegExpExecArray | null;
  while ((m = re.exec(sfc)) !== null) {
    const body = m[1]?.trim();
    if (body) out.push(m[1]!);
  }
  return out.length > 0 ? out : [sfc];
}

function pushDedupe(set: Set<string>, list: string[], msg: string): void {
  if (!set.has(msg)) {
    set.add(msg);
    list.push(msg);
  }
}

function scanScriptAst(source: string, dedupe: Set<string>, warnings: string[]): void {
  let ast: acorn.Node;
  try {
    ast = acorn.parse(source, {
      ecmaVersion: 2024,
      sourceType: "module",
    }) as acorn.Node;
  } catch {
    return;
  }

  simple(ast, {
    MemberExpression(node: acorn.Node) {
      const n = node as unknown as {
        object: { type: string; name?: string };
        property: { type: string; name?: string; value?: string };
      };
      if (
        n.object.type === "Identifier" &&
        n.object.name === "window" &&
        n.property.type === "Identifier" &&
        (n.property.name === "__TAURI__" || n.property.name === "tauri")
      ) {
        pushDedupe(
          dedupe,
          warnings,
          "检测到 `window.__TAURI__` / `window.tauri` 访问",
        );
      }
      if (
        n.object.type === "Identifier" &&
        n.object.name === "document" &&
        n.property.type === "Identifier" &&
        n.property.name === "cookie"
      ) {
        pushDedupe(dedupe, warnings, "检测到 `document.cookie`");
      }
      if (
        n.object.type === "Identifier" &&
        n.object.name === "localStorage" &&
        n.property.type === "Identifier" &&
        (n.property.name === "setItem" || n.property.name === "getItem")
      ) {
        pushDedupe(dedupe, warnings, "检测到 `localStorage` 读写");
      }
    },
    CallExpression(node: acorn.Node) {
      const n = node as unknown as {
        callee: { type: string; name?: string };
      };
      if (n.callee.type === "Identifier" && n.callee.name === "fetch") {
        pushDedupe(dedupe, warnings, "检测到 `fetch` 调用");
      }
      if (n.callee.type === "Identifier" && n.callee.name === "eval") {
        pushDedupe(dedupe, warnings, "检测到 `eval` 调用");
      }
    },
    NewExpression(node: acorn.Node) {
      const n = node as unknown as {
        callee: { type: string; name?: string };
      };
      if (n.callee.type === "Identifier" && n.callee.name === "XMLHttpRequest") {
        pushDedupe(dedupe, warnings, "检测到 `XMLHttpRequest`");
      }
    },
  });
}

/** 对 `.vue` 或纯脚本片段做静态扫描（黑名单）；不保证零误报/漏报。 */
export function scanVueComponentSource(source: string): ScanResult {
  const warnings: string[] = [];
  const dedupe = new Set<string>();
  for (const block of extractScriptBodies(source)) {
    scanScriptAst(block, dedupe, warnings);
  }
  return { warnings };
}
