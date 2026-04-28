export type PluginReviewEntryDtoLike = {
  id: string;
  plugin_id: string;
  pubkey_id?: string | null;
  version?: string | null;
  rating: number;
  title?: string | null;
  body?: string | null;
  created_at: string;
  author_github?: string | null;
};

export type ReviewPreview = {
  id: string;
  rating: number;
  title?: string | null;
  body?: string | null;
  created_at: string;
  pubkey_id?: string | null;
  version?: string | null;
  author_github?: string | null;
};

export function buildReviewJsonTemplate(params: {
  pluginId: string;
  pubkeyId?: string | null;
  version?: string | null;
}): string {
  const now = new Date().toISOString();
  const pid = params.pluginId.trim();
  const pk = (params.pubkeyId ?? "").trim();
  const v = (params.version ?? "").trim();
  const obj = {
    id: `r-${now.replace(/[-:]/g, "").slice(0, 15)}-yourid`,
    pluginId: pid,
    ...(pk ? { pubkeyId: pk } : {}),
    ...(v ? { version: v } : {}),
    rating: 5,
    title: "一句话短评（可选）",
    body: "详细说明（可选）",
    createdAt: now,
    author: { github: "your-github-id" },
  };
  return JSON.stringify(obj, null, 2);
}

export function normalizeStars(rating: number): number {
  const x = Number(rating);
  if (!Number.isFinite(x)) return 0;
  return Math.max(1, Math.min(5, Math.round(x)));
}

export function ratingStars(avg: number): string {
  const n = Math.max(0, Math.min(5, Math.round(avg)));
  return "★★★★★".slice(0, n) + "☆☆☆☆☆".slice(0, 5 - n);
}

export function getRecentReviews(
  reviews: PluginReviewEntryDtoLike[],
  params: {
    pluginId: string;
    pubkeyId?: string | null;
    version?: string | null;
    limit?: number;
  },
): ReviewPreview[] {
  const pid = params.pluginId.trim();
  const pk = (params.pubkeyId ?? "").trim();
  const v = (params.version ?? "").trim();
  const limit = Math.max(1, Math.min(20, params.limit ?? 3));
  return (reviews ?? [])
    .filter((r) => {
      if ((r.plugin_id ?? "").trim() !== pid) return false;
      if (pk && (r.pubkey_id ?? "").trim() !== pk) return false;
      if (v && (r.version ?? "").trim() !== v) return false;
      return true;
    })
    .slice()
    .sort((a, b) => String(b.created_at ?? "").localeCompare(String(a.created_at ?? "")))
    .slice(0, limit)
    .map((r) => ({
      id: r.id,
      rating: normalizeStars(r.rating),
      title: r.title ?? null,
      body: r.body ?? null,
      created_at: r.created_at,
      pubkey_id: r.pubkey_id ?? null,
      version: r.version ?? null,
      author_github: r.author_github ?? null,
    }));
}

export function renderReviewLine(r: ReviewPreview): string {
  const title = (r.title ?? "").trim();
  const body = (r.body ?? "").trim();
  const text = title || body || "（无内容）";
  const who = (r.author_github ?? "").trim();
  const whoText = who ? ` @${who}` : "";
  return `${ratingStars(r.rating)} ${text}${whoText}`;
}

