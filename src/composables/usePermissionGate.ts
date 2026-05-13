import { ref } from "vue";
import { useI18n } from "vue-i18n";
import {
  getPluginPermissionGrants,
  listPermissionTokens,
  setPluginPermissionGrant,
  type PermissionTokenInfoDto,
} from "../utils/tauri-api";
import { appConfirm } from "../utils/confirmDialog";
import { useAppToast } from "./useAppToast";

type EnsurePermissionParams = {
  subjectId: string;
  required: string[];
  title: string;
  detailLines?: string[];
};

export function usePermissionGate() {
  const { t } = useI18n();
  const { showToast } = useAppToast();

  const tokenInfoLoading = ref(false);
  const tokenInfoMap = ref<Map<string, PermissionTokenInfoDto>>(new Map());

  async function ensureTokenRegistry(): Promise<void> {
    if (tokenInfoMap.value.size > 0) return;
    tokenInfoLoading.value = true;
    try {
      const res = await listPermissionTokens();
      const map = new Map<string, PermissionTokenInfoDto>();
      for (const x of res.tokens ?? []) {
        if (!x?.token) continue;
        map.set(x.token, x);
      }
      tokenInfoMap.value = map;
    } finally {
      tokenInfoLoading.value = false;
    }
  }

  function permissionPlainText(token: string): string {
    const info = tokenInfoMap.value.get(token);
    if (!info) {
      if (token === "process:spawn") return String(t("permissionGate.plain.processSpawn"));
      if (token === "network:*") return String(t("permissionGate.plain.networkAll"));
      return token;
    }
    return `${info.title}（${info.token}）`;
  }

  async function ensurePermissionsOrCancel(
    p: EnsurePermissionParams,
  ): Promise<{ ok: boolean; missing: string[] }> {
    const subjectId = p.subjectId.trim();
    const required = (p.required ?? []).map((x) => String(x ?? "").trim()).filter(Boolean);
    if (!subjectId || required.length === 0) return { ok: true, missing: [] };

    await ensureTokenRegistry().catch(() => {
      // fall back to raw tokens
    });

    let grants: { permission: string; enabled: boolean }[] = [];
    try {
      const res = await getPluginPermissionGrants(subjectId);
      grants = res.grants ?? [];
    } catch (e) {
      showToast("error", e instanceof Error ? e.message : String(e));
      return { ok: false, missing: required };
    }

    const enabled = new Set(
      (grants ?? [])
        .filter((g) => g?.enabled === true)
        .map((g) => String(g?.permission ?? "").trim())
        .filter(Boolean),
    );
    const missing = required.filter((x) => !enabled.has(x));
    if (missing.length === 0) return { ok: true, missing: [] };

    const lines = [
      String(t("permissionGate.confirm.header", { title: p.title })),
      String(t("permissionGate.confirm.subject", { id: subjectId })),
      String(t("permissionGate.confirm.permsHeader")),
      ...missing.map((tok) => `- ${permissionPlainText(tok)}`),
      ...(p.detailLines?.length ? ["", ...p.detailLines] : []),
      "",
      String(t("permissionGate.confirm.cta")),
    ];
    const ok = await appConfirm(lines.join("\n"), { title: p.title, type: "warning" });
    if (!ok) {
      showToast("info", String(t("permissionGate.toast.cancelled")));
      return { ok: false, missing };
    }

    for (const tok of missing) {
      try {
        await setPluginPermissionGrant(subjectId, tok, true);
      } catch (e) {
        showToast("error", e instanceof Error ? e.message : String(e));
        return { ok: false, missing };
      }
    }
    return { ok: true, missing: [] };
  }

  return {
    tokenInfoLoading,
    ensurePermissionsOrCancel,
  };
}

