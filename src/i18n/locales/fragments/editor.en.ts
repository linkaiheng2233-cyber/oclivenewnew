/** editor — en. */
export default {
  editor: {
    personalityTrait: {
      stubbornness: 'Stubbornness',
      clinginess: 'Clinginess',
      sensitivity: 'Sensitivity',
      assertiveness: 'Assertiveness',
      forgiveness: 'Forgiveness',
      talkativeness: 'Talkativeness',
      warmth: 'Warmth',
    },
    chatExport: {
      allRoles: 'Export all roles',
      pluginDebug: 'Include plugin resolution debug (single role)',
      exportJson: 'Export JSON',
      exportTxt: 'Export TXT',
      downloaded: 'Downloaded {name}',
      success: 'Export saved',
      saveCancelled: 'Save cancelled',
    },
    debug: {
      monologueInserted: 'Monologue inserted',
      monologuePrefix: '[Monologue] ',
      title: '🎛️ Debug panel',
      hint1:
        'Inspect favorability, traits, recent events and memory; reload policy, generate monologue, import/manage packs.',
      hint2:
        'Ctrl+Shift+D toggles this panel; Esc also closes it. Under top bar “More”, use “Open debug panel”.',
      dockSlotAria: 'Debug dock slot',
      insertMonoGenerating: 'Generating…',
      insertMono: 'Insert monologue',
      knowledgeTitle: 'World knowledge',
      knowledgeIndexed: 'Pack index:',
      knowledgeLoaded: 'loaded',
      knowledgeNotLoaded: 'not loaded',
      knowledgeChunks: ' · {n} chunks',
      knowledgeLastPrompt: 'Last prompt injection:',
      knowledgeChunksUnit: 'chunks',
      knowledgeLastPromptLine: 'Last prompt injection: {n} chunks',
      knowledgePresenceInline: '({label})',
      knowledgeHint:
        'Updates after you send; click “Refresh debug data” to sync chunk counts (call load_role after disk changes).',
      favorability: 'Favorability',
      personalityVector: 'Personality vector',
      personalityProfileHelp:
        'Profile-sourced pack: these seven values are mostly derived from archives for readability, not the sole source.',
      metaCounts: 'Events: {events} · Memories: {memories}',
      recentEvents: 'Recent events',
      recentMemories: 'Recent memories',
      refresh: 'Refresh debug data',
      reloadPolicy: 'Reload policy',
      footer: '💡 Ctrl+Shift+D toggles panel · packs & monologue live here',
      fav80: '💖 Very close!',
      fav60: '💕 Going strong',
      fav40: '👍 Okay',
      fav20: '🤝 Getting to know',
      fav0: '😶 Still distant',
      presenceCoPresent: 'Co-present',
      presenceRemoteStub: 'Remote placeholder',
      presenceRemoteLife: 'Remote inner voice',
    },
  }
}
