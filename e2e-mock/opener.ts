/**
 * Optional no-op opener mock for Playwright e2e (Tauri 2 plugin-opener).
 */
export async function openPath(_path: string): Promise<void> {}

export async function openUrl(_url: string): Promise<void> {}
