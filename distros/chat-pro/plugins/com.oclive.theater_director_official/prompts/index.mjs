/**
 * Theater prompt pack v0.2 — mode router (official plugin SSOT).
 */
import { buildCastAdapt } from "./modes/cast_adapt.mjs";
import { buildCastRewrite } from "./modes/cast_rewrite.mjs";
import { buildCastRewriteMinimal } from "./modes/cast_rewrite_minimal.mjs";
import { buildPatch } from "./modes/patch.mjs";
import { buildRipple } from "./modes/ripple.mjs";

export { DRIFT_MARKERS } from "./drama_guardrails.mjs";

export function buildTheaterPrompt(input) {
  const mode = (input.mode || "ripple").trim();
  switch (mode) {
    case "patch":
      return buildPatch(input);
    case "cast_adapt":
      return buildCastAdapt(input);
    case "cast_rewrite":
      return buildCastRewrite(input);
    case "cast_rewrite_minimal":
      return buildCastRewriteMinimal(input);
    case "ripple":
    default:
      return buildRipple(input);
  }
}
