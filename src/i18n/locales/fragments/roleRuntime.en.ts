/** roleRuntime — en. */
export default {
  roleRuntime: {
    personalityProfile: 'Profile (mutable text maintained by chat)',
    personalityVector: '7D vector',
    profileHint1:
      'Profile source: runtime uses core + mutable personality archives; the seven dimensions here are mostly a summarized view.',
    profileHint2:
      'Unlike vector mode (dimensions drive events); see docs/personality-archive-notes.md.',
    vectorHint1:
      'Vector source: events and mood adjust per dimension; matches evolution.personality_source in settings.',
    versionAuthor: 'Version {version} · Author {author}',
    personalitySource: 'Personality source:',
    backendHintBefore: 'Reply model and LLM backend (session override) — open',
    modelManagerLink: 'Model management',
    backendHintAfter: '(Ctrl+Shift+M)',
    relation: 'Relation',
    eventImpact: 'Event impact',
  }
}
