/** roleRuntime — zh. */
export default {
  roleRuntime: {
    personalityProfile: '档案（可变正文由对话维护）',
    personalityVector: '七维向量',
    profileHint1:
      '人格来源为 profile：运行时以核心性格档案与数据库中的「可变性格档案」为准；界面七维多为从正文归纳的视图。',
    profileHint2:
      '与 vector 模式（七维直接参与事件演化）不同；设计说明见仓库 docs/personality-archive-notes.md。',
    vectorHint1:
      '人格来源为 vector：事件与情绪按七维精细化调整；与 settings 中 evolution.personality_source 一致。',
    versionAuthor: '版本 {version} · 作者 {author}',
    personalitySource: '人格来源：',
    backendHintBefore: '对话模型与 LLM 后端（本会话覆盖）请打开',
    modelManagerLink: '模型管理',
    backendHintAfter: '（Ctrl+Shift+M）',
    relation: '关系',
    eventImpact: '事件影响',
  }
}
