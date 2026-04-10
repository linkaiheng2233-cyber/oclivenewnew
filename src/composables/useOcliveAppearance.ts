import { computed, onMounted, onUnmounted, ref, watch } from "vue";

const THEME_STORAGE_KEY = "oclive-runtime-theme";
const SCALE_STORAGE_KEY = "oclive-runtime-ui-scale";

export type ThemePreference = "light" | "dark" | "system";

/** 与 pack-editor / launcher 相同的字号档位 */
const UI_SCALE_STEPS = [0.8, 0.88, 0.96, 1, 1.08, 1.16, 1.24] as const;

function resolveEffectiveTheme(pref: ThemePreference): "light" | "dark" {
  if (pref === "light") return "light";
  if (pref === "dark") return "dark";
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

function readStoredTheme(): ThemePreference {
  try {
    const raw = localStorage.getItem(THEME_STORAGE_KEY);
    if (raw === "light" || raw === "dark" || raw === "system") return raw;
  } catch {
    /* ignore */
  }
  return "system";
}

function nearestScaleIndex(value: number): number {
  let best = 3;
  let bestDiff = Infinity;
  UI_SCALE_STEPS.forEach((s, i) => {
    const d = Math.abs(s - value);
    if (d < bestDiff) {
      bestDiff = d;
      best = i;
    }
  });
  return best;
}

function readStoredScaleIndex(): number {
  try {
    const raw = localStorage.getItem(SCALE_STORAGE_KEY);
    if (raw == null || raw === "") return 3;
    const n = Number(raw);
    if (!Number.isFinite(n)) return 3;
    return nearestScaleIndex(n);
  } catch {
    return 3;
  }
}

export function useOcliveAppearance() {
  const themePreference = ref<ThemePreference>("system");
  const uiScaleIndex = ref(3);

  function applyThemeToDocument() {
    document.documentElement.setAttribute(
      "data-theme",
      resolveEffectiveTheme(themePreference.value),
    );
  }

  function applyScaleToDocument() {
    const scale = UI_SCALE_STEPS[uiScaleIndex.value] ?? 1;
    document.documentElement.style.setProperty(
      "--oclive-ui-scale",
      String(scale),
    );
  }

  let removeSchemeListener: (() => void) | undefined;

  onMounted(() => {
    themePreference.value = readStoredTheme();
    uiScaleIndex.value = readStoredScaleIndex();
    applyThemeToDocument();
    applyScaleToDocument();

    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const onScheme = () => {
      if (themePreference.value === "system") applyThemeToDocument();
    };
    mq.addEventListener("change", onScheme);
    removeSchemeListener = () => mq.removeEventListener("change", onScheme);
  });

  onUnmounted(() => {
    removeSchemeListener?.();
  });

  watch(themePreference, (v) => {
    try {
      localStorage.setItem(THEME_STORAGE_KEY, v);
    } catch {
      /* ignore */
    }
    applyThemeToDocument();
  });

  watch(uiScaleIndex, (i) => {
    const scale = UI_SCALE_STEPS[i] ?? 1;
    try {
      localStorage.setItem(SCALE_STORAGE_KEY, String(scale));
    } catch {
      /* ignore */
    }
    applyScaleToDocument();
  });

  const themeCycleLabel = computed(() => {
    if (themePreference.value === "system") return "跟随系统";
    if (themePreference.value === "dark") return "深色";
    return "浅色";
  });

  function cycleTheme() {
    const order: ThemePreference[] = ["system", "light", "dark"];
    const i = order.indexOf(themePreference.value);
    themePreference.value = order[(i + 1) % order.length]!;
  }

  function bumpScale(delta: number) {
    uiScaleIndex.value = Math.max(
      0,
      Math.min(UI_SCALE_STEPS.length - 1, uiScaleIndex.value + delta),
    );
  }

  const scaleLabel = computed(
    () => `${Math.round((UI_SCALE_STEPS[uiScaleIndex.value] ?? 1) * 100)}%`,
  );

  return {
    themePreference,
    themeCycleLabel,
    cycleTheme,
    bumpScale,
    scaleLabel,
  };
}
