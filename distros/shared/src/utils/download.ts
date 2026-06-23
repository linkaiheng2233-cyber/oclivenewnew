/** Browser download (under Tauri WebView usually lands in Downloads or system save flow); no extra Rust permission. */
export function downloadTextFile(
  filename: string,
  content: string,
  mime: string,
): void {
  const blob = new Blob([content], { type: `${mime};charset=utf-8` })
  triggerBlobDownload(filename, blob)
}

/** Download base64-encoded binary (e.g. ZIP export). */
export function downloadBase64File(
  filename: string,
  base64: string,
  mime: string,
): void {
  const binary = atob(base64)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++)
    bytes[i] = binary.charCodeAt(i)
  const blob = new Blob([bytes], { type: mime })
  triggerBlobDownload(filename, blob)
}

function triggerBlobDownload(filename: string, blob: Blob): void {
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  a.rel = 'noopener'
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}
