import type { ExpertGraph, PromptStyleOverride } from "../utils/tauri-api";

export const OCLEXPERT_FORMAT = "oclexpert" as const;
export const OCLEXPERT_FILE_VERSION = 1 as const;

const ALLOWED_NODE_TYPES = new Set([
  "base_model",
  "lora_adapter",
  "prompt_style",
  "cloud_model",
  "event_trigger",
]);

export interface OclexpertFileV1 {
  format: typeof OCLEXPERT_FORMAT;
  fileVersion: typeof OCLEXPERT_FILE_VERSION;
  /** Optional human label for save dialogs */
  name?: string;
  graph: ExpertGraph;
  promptStyle?: PromptStyleOverride | null;
}

export function buildOclexpertPayload(
  graph: ExpertGraph,
  promptStyle: PromptStyleOverride | null | undefined,
  name?: string,
): OclexpertFileV1 {
  return {
    format: OCLEXPERT_FORMAT,
    fileVersion: OCLEXPERT_FILE_VERSION,
    name: name?.trim() || undefined,
    graph: JSON.parse(JSON.stringify(graph)) as ExpertGraph,
    promptStyle: promptStyle ? { ...promptStyle } : null,
  };
}

export class OclexpertImportError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "OclexpertImportError";
  }
}

function isRecord(x: unknown): x is Record<string, unknown> {
  return typeof x === "object" && x !== null && !Array.isArray(x);
}

/** Validate graph nodes have known `type` tags (serde snake_case). */
export function validateExpertGraphNodes(graph: ExpertGraph): void {
  const nodes = graph.nodes ?? [];
  for (let i = 0; i < nodes.length; i += 1) {
    const n = nodes[i] as { type?: string };
    const t = String(n?.type ?? "").trim();
    if (!t || !ALLOWED_NODE_TYPES.has(t)) {
      throw new OclexpertImportError(
        `Invalid or unknown expert node type at index ${i}: ${t || "(missing)"}`,
      );
    }
  }
}

/**
 * Parse `.oclexpert` / shareable JSON: wrapped v1 file or bare `ExpertGraph`.
 */
export function parseOclexpertJson(raw: string): {
  graph: ExpertGraph;
  promptStyle: PromptStyleOverride | null;
  suggestedName?: string;
} {
  let data: unknown;
  try {
    data = JSON.parse(raw);
  } catch {
    throw new OclexpertImportError("File is not valid JSON.");
  }
  if (!isRecord(data)) {
    throw new OclexpertImportError("Root JSON must be an object.");
  }

  if (data.format === OCLEXPERT_FORMAT) {
    const fv = data.fileVersion;
    if (fv !== 1) {
      throw new OclexpertImportError(
        `Unsupported .oclexpert fileVersion: ${String(fv)} (this app supports version 1).`,
      );
    }
    const graph = data.graph as ExpertGraph | undefined;
    if (!graph || !Array.isArray(graph.nodes)) {
      throw new OclexpertImportError("Missing graph.nodes in .oclexpert file.");
    }
    validateExpertGraphNodes(graph);
    const name = typeof data.name === "string" ? data.name.trim() : "";
    const ps = data.promptStyle;
    const promptStyle =
      ps && typeof ps === "object" ? (ps as PromptStyleOverride) : null;
    return {
      graph: JSON.parse(JSON.stringify(graph)) as ExpertGraph,
      promptStyle,
      suggestedName: name || undefined,
    };
  }

  // Bare ExpertGraph (e.g. old workflow export)
  if (Array.isArray(data.nodes)) {
    const graph = data as unknown as ExpertGraph;
    validateExpertGraphNodes(graph);
    return {
      graph: JSON.parse(JSON.stringify(graph)) as ExpertGraph,
      promptStyle: null,
    };
  }

  throw new OclexpertImportError(
    'Unrecognized file: expected format "oclexpert" or a bare ExpertGraph with nodes[].',
  );
}
