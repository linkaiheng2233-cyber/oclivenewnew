export class VoiceSpeakDeduper {
  private queuedKeys = new Set<string>()
  private spokenKeys = new Set<string>()

  reset(): void {
    this.queuedKeys.clear()
    this.spokenKeys.clear()
  }

  markQueued(key: string): boolean {
    if (!key || this.queuedKeys.has(key) || this.spokenKeys.has(key))
      return false
    this.queuedKeys.add(key)
    return true
  }

  finish(key: string, spoken: boolean): void {
    if (!key)
      return
    this.queuedKeys.delete(key)
    if (spoken)
      this.spokenKeys.add(key)
  }
}
