/** editor — zh. */
export default {
  editor: {
    personalityTrait: {
      stubbornness: '倔强',
      clinginess: '黏人',
      sensitivity: '敏感',
      assertiveness: '强势',
      forgiveness: '宽容',
      talkativeness: '话多',
      warmth: '温暖',
    },
    chatExport: {
      allRoles: '导出全部角色',
      pluginDebug: '附带插件诊断（单角色）',
      exportJson: '导出 JSON',
      exportTxt: '导出 TXT',
      downloaded: '已下载 {name}',
      success: '导出成功',
      saveCancelled: '已取消保存',
    },
    debug: {
      monologueInserted: '已插入独白',
      monologuePrefix: '【独白】',
      title: '🎛️ 开发面板',
      hint1:
        '供开发与排错：查看好感度、性格维度、近期事件与记忆摘要；可重载策略、生成独白、导入或管理角色包等。',
      hint2:
        '快捷键 Ctrl+Shift+D（同时按住 Ctrl、Shift，再按字母 D）可随时打开或关闭本面板；按 Esc 也可关闭。顶栏「更多」里亦可点「打开调试面板」。',
      dockSlotAria: '调试面板扩展槽',
      insertMonoGenerating: '生成中…',
      insertMono: '插入独白',
      knowledgeTitle: '世界观知识',
      knowledgeIndexed: '包内索引：',
      knowledgeLoaded: '已加载',
      knowledgeNotLoaded: '未加载',
      knowledgeChunks: '· 共 {n} 块',
      knowledgeLastPrompt: '上一句注入 Prompt：',
      knowledgeChunksUnit: '块',
      knowledgeLastPromptLine: '上一句注入 Prompt：{n} 块',
      knowledgePresenceInline: '（{label}）',
      knowledgeHint:
        '发话后更新「上一句」；点「刷新调试数据」同步包内块数（改磁盘后请先 load_role）。',
      favorability: '好感度',
      personalityVector: '性格向量',
      personalityProfileHelp:
        '当前包为「档案」人格来源：此处七维多为运行时从核心与可变性格档案归纳的视图，便于理解，不是唯一数据源。',
      metaCounts: '事件数: {events} · 记忆数: {memories}',
      recentEvents: '最近事件',
      recentMemories: '最近记忆',
      refresh: '刷新调试数据',
      reloadPolicy: '重载策略',
      footer: '💡 Ctrl+Shift+D 开关面板 · 角色包与独白已收在此',
      fav80: '💖 超级亲密！',
      fav60: '💕 关系很好~',
      fav40: '👍 还不错',
      fav20: '🤝 慢慢熟悉中',
      fav0: '😶 还有点陌生',
      presenceCoPresent: '共景',
      presenceRemoteStub: '异地占位',
      presenceRemoteLife: '异地心声',
    },
  }
}
