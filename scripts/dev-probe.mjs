const url = process.env.OCLIVE_DEV_PROBE_URL ?? "http://127.0.0.1:1420/";
const timeoutMs = Number(process.env.OCLIVE_DEV_PROBE_TIMEOUT_MS ?? "8000");

async function main() {
  const ac = new AbortController();
  const t = setTimeout(() => ac.abort(), timeoutMs);
  try {
    const res = await fetch(url, { signal: ac.signal, headers: { Accept: "text/html" } });
    if (!res.ok) {
      console.error(`[dev-probe] HTTP ${res.status} <- ${url}`);
      process.exit(1);
    }
    console.info(`[dev-probe] OK  ${url}`);
    process.exit(0);
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    console.error(`[dev-probe] cannot reach ${url} (${msg})`);
    process.exit(1);
  } finally {
    clearTimeout(t);
  }
}

void main();
