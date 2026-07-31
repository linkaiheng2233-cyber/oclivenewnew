const configuredUrl = process.env.OCLIVE_DEV_PROBE_URL?.trim();
const urls = configuredUrl
  ? [configuredUrl]
  : [
      "http://127.0.0.1:1420/",
      "http://[::1]:1420/",
      "http://localhost:1420/",
    ];
const configuredTimeoutMs = Number(
  process.env.OCLIVE_DEV_PROBE_TIMEOUT_MS ?? "8000",
);
const timeoutMs =
  Number.isFinite(configuredTimeoutMs) && configuredTimeoutMs > 0
    ? configuredTimeoutMs
    : 8000;
const attemptTimeoutMs = configuredUrl
  ? timeoutMs
  : Math.max(250, Math.min(2000, Math.floor(timeoutMs / urls.length)));

async function probe(url) {
  const ac = new AbortController();
  const t = setTimeout(() => ac.abort(), attemptTimeoutMs);
  try {
    const res = await fetch(url, { signal: ac.signal, headers: { Accept: "text/html" } });
    if (!res.ok) {
      return { ok: false, detail: `HTTP ${res.status}` };
    }
    return { ok: true };
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    return { ok: false, detail: msg };
  } finally {
    clearTimeout(t);
  }
}

async function main() {
  const failures = [];
  for (const url of urls) {
    const result = await probe(url);
    if (result.ok) {
      console.info(`[dev-probe] OK  ${url}`);
      return;
    }
    failures.push(`${url} (${result.detail})`);
  }
  console.error(`[dev-probe] cannot reach ${failures.join(" | ")}`);
  process.exitCode = 1;
}

void main();
