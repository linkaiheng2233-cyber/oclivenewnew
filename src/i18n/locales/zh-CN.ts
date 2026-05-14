import apiErrors from "./fragments/apiErrors.zh";
import chat from "./fragments/chat.zh";
import devTools from "./fragments/devTools.zh";
import pluginWorkbench from "./fragments/pluginWorkbench.zh";
import virtualTime from "./fragments/virtualTime.zh";

export default {
  apiErrors,
  chat,
  devTools,
  pluginWorkbench,
  virtualTime,
  app: {
    locale: {
      label: "界面语言",
      system: "跟随系统",
      zhCN: "中文",
      enUS: "English",
    },
    theme: {
      system: "跟随系统",
      light: "浅色",
      dark: "深色",
    },
    defaultRoleName: "沐沐",
    more: {
      collapse: "收起",
      more: "更多",
      ariaMoreFeatures: "更多功能",
      interactionMode: "互动模式",
      interactionImmersiveHint:
        "沉浸：启用虚拟时间、叙事场景、日程推断与位移相关能力。",
      interactionPureChatHint: "纯聊：只保留对话，隐藏场景与时间条，适合日常闲聊。",
      interactionImmersive: "沉浸",
      interactionPureChat: "纯聊",
      identity: "身份",
      identityHelp:
        "与角色相处时的关系身份（如朋友、恋人等），影响对话与关系数值；与包内「核心性格档案」不同，后者写在 core_personality.txt。",
      ui: "界面",
      uiHint1: "字号 A− / A+ 与编写器、启动器使用同一套档位，会保存在本机。",
      uiHint2: "主题为浅色 / 深色 / 跟随系统，亦会记住。",
      appearanceToolbar: "外观与字号",
      scaleGroup: "界面大小",
      shrinkTitle: "缩小",
      shrinkAria: "缩小界面",
      scaleRelativeTitle: "相对默认字号：{label}",
      enlargeTitle: "放大",
      enlargeAria: "放大界面",
      themeTitle: "主题：{label}（点击切换）",
      settingsEntry: "设置入口",
      shortcutHelp: "快捷键说明",
      openSettings: "⚙ 设置",
      debug: "调试",
      debugHelp:
        "开发者与排错用：好感、记忆、策略重载等。Ctrl+Shift+D 可开关调试窗；顶栏「更多」展开时按 Esc 先收起本栏。",
      openDebugPanel: "打开调试面板",
      virtualTime: "虚拟时间",
      virtualTimeHint1: "故事内的时间，与真实时钟独立。点击时间可打开滚轮调整。",
      virtualTimeHint2: "可用快捷按钮推进时间；部分角色包会在跳转后触发场景或独白。",
      narrativeScene: "叙事场景",
      narrativeSceneHelp:
        "你当前叙事的场景；与角色包中的场景配置一致。切换后可能触发历史记录折叠分界。",
      characterAt: "角色在：{label}",
      pluginBtnV1: "插件与后端（V1）",
      pluginBtnV2: "插件管理（V2）",
      settingsTileHelpV2:
        "将快捷键说明、设置页、插件管理集中到同一处。快捷键：Ctrl+Shift+S 打开设置；" +
        "Ctrl+Shift+F 与下方按钮打开插件管理（V2 预览）；在设置中关闭「V2 预览」可恢复专业模式（V1）。 Ctrl+Shift+D 开关调试面板。",
      settingsTileHelpV1:
        "将快捷键说明、设置页、插件与后端管理集中到同一处。快捷键：Ctrl+Shift+S 打开设置；" +
        "Ctrl+Shift+F 打开专业模式（V1）插件与后端管理（含开发者调试）。 Ctrl+Shift+D 开关调试面板。",
    },
    toast: {
      remoteLifeOn: "异地心声已开启",
      remoteLifeOff: "异地心声已关闭",
      interactionImmersive: "互动模式已切换为沉浸",
      interactionPureChat: "互动模式已切换为纯聊",
      layoutResetOk: "已恢复为角色包推荐布局。",
      layoutResetFailPrefix: "恢复失败：",
      noRolesScanned:
        "未扫描到任何可用角色包（roles 目录为空或全部校验失败）。请检查宿主使用的 roles 路径：开发可设置环境变量 OCLIVE_ROLES_DIR 指向仓库的 roles 文件夹。",
      fallbackReply: "本次为备用回复（模型未返回正文时自动生成）",
      roleSwitched: "已切换角色: {id}",
      relationSetPerScene: "已设置当前场景身份：{name}",
      relationSetGlobal: "已设置身份：{name}",
      pluginInstalledFromWeb: "已通过网页链接安装插件：{id}",
      pluginFilesChanged: "检测到插件变更，已自动刷新",
    },
    sceneTransition: {
      going: "正在前往「{label}」…",
    },
    sidebar: {
    favorability: "好感度",
    lifeNow: "此刻：{label}",
    scheduleInference: "日程推断",
    },
    floatingSlot: "浮层插件区",
    scene: {
      selectDestinationFirst: "请先选择目的地",
      toastTogether: "已前往（同行）",
      toastNarrativeOnly: "已切换叙事场景（角色未移动）",
      systemLine:
        "叙事场景已切换为「{narrative}」；角色仍位于「{character}」。",
    },
  },
  pluginTerms: {
    module: {
      llm: "对话大脑（LLM）",
      emotion: "情绪引擎（Emotion）",
      complex_emotion: "复杂情感（Complex Emotion）",
    },
    category: {
      all: "全部功能",
      module: "按模块",
      type: "按实现方式",
      status: "按状态",
    },
    type: {
      builtin: "内置",
      remote: "远程",
      directory: "本地目录插件",
    },
    status: {
      enabled: "已启用",
      disabled: "已关闭",
      needs_config: "还需配置",
    },
    backend: {
      follow_default: "跟随角色包默认",
      ollama: "Ollama（本地模型）",
      remote: "远程服务",
      directory: "目录插件",
      builtin: "内置",
      builtin_v2: "内置 V2",
    },
    field: {
      backend: "运行方式",
      directory_plugin: "目录插件 ID",
      remote_life: "异地心声",
    },
    hint: {
      directory_id_empty: "留空会清空会话覆盖，回到角色包默认。",
      endpoint_env: "地址建议放在环境变量里，便于迁移与排错。",
    },
    action: {
      apply: "应用改动",
      open_v1: "打开专业模式（V1）",
      close: "关闭",
    },
    title: { v2: "插件与后端管理 V2（简易模式）" },
    subtitle: {
      v2:
        "面向日常使用：大白话说明、分类筛选、模板化配置。目录插件「开发者调试」请在专业模式（V1）的「界面插件」中使用。",
    },
  },
  pluginManager: {
    legend: {
      enabled: "已启用：当前配置可直接生效",
      pending: "还需配置：通常缺少目录插件 ID",
      disabled: "已关闭：当前链路未启用",
    },
    source: {
      session_override: "会话覆盖",
      env_override: "环境覆盖",
      pack_default: "角色包默认",
    },
    risk: {
      needsConfig: "缺配置",
      envFirst: "环境优先",
    },
    nav: {
      explorerAria: "筛选（工作区风格）",
      title: "资源管理器",
      subtitle: "筛选视图",
      rootTooltip: "仅用于 UI 层级展示，不代表磁盘路径",
      treeAria: "筛选树",
      byModule: "按模块",
      byBackend: "按实现",
      byStatus: "按状态",
    },
    search: {
      placeholder: "搜索：例如 远程、情绪、目录插件",
      empty: "没有匹配项，试试更短的关键词。",
    },
    detail: {
      readonlyNotice:
        "只读说明：此处不会写入任何配置；请在环境变量或角色包中修改后重载应用。",
      previewNotice:
        "变更预览：点击下方「应用改动」后写入当前会话（不修改角色包 settings.json；若与环境变量冲突，以环境解析为准）。",
      expand: "展开",
      collapse: "收起",
      placeholder: "先从中间列表选一个卡片。",
    },
    env: { label: "环境变量" },
    cards: {
      optionPackDefault: "跟随角色包默认（{backend}）",
      llmMain: {
        title: "对话回复引擎",
        description: "决定回复模型来源：本地模型、远程服务或目录插件。",
      },
      llmEndpoint: {
        title: "LLM 远程地址说明",
        description: "选择远程服务时，优先读取 LLM 专用地址。",
        summary: "建议在系统环境变量配置地址，便于迁移与排错。",
        fieldLlmUrl: "LLM 专用远程地址（优先）",
        fieldPluginUrl: "通用远程地址（兜底）",
      },
      emotionMain: {
        title: "情绪推理引擎",
        description: "控制情绪由内置逻辑、远程服务或目录插件处理。",
      },
      emotionEndpoint: {
        title: "Emotion 远程地址说明",
        description: "情绪 remote 默认读取通用远程地址。",
        summary: "建议在系统环境变量配置地址，避免写死到角色包。",
        fieldPluginUrl: "Emotion 常用远程入口",
      },
      complexSwitch: {
        title: "复杂情感开关",
        description: "开启后启用异地心声链路，复杂情感表现更明显。",
        sessionOn: "当前会话已开启",
        sessionOff: "当前会话已关闭",
        label: "启用复杂情感（异地心声）",
        hint: "开启后建议配置 URL 与 TOKEN 环境变量。",
      },
      complexEndpoint: {
        title: "复杂情感地址说明",
        description: "复杂情感服务通常独立部署，支持鉴权 token。",
        summary: "若服务要求鉴权，请同时配置 URL 和 TOKEN。",
        fieldUrl: "复杂情感服务地址",
        fieldToken: "复杂情感服务鉴权 Token",
      },
    },
    apply: {
      endpointNoSave: "地址说明项无需保存，请在环境变量中配置。",
      remoteLifeUpdated: "复杂情感开关已更新。",
      unsupported: "当前仅支持 LLM / Emotion 配置写入。",
      sessionSaved: "配置已写入当前会话。",
    },
    /** V1 已安装区 · 单插件工作区 */
    installed: {
      privateSettings: "插件私有设置",
      debugWorkbench: "调试台",
    },
  },
  settings: {
    ariaDialog: "设置",
    ariaNav: "设置分区",
    title: "设置",
    closeAria: "关闭",
    tabGeneral: "常规",
    tabPlugins: "插件扩展",
    generalLeadHtml:
      "顶栏<strong>「更多」</strong>集中设置入口；打开设置可用 <strong>Ctrl+Shift+S</strong>；" +
      "<strong>Ctrl+Shift+F</strong> 打开插件管理（未勾选下方「V2 预览」时为<strong>专业模式（V1）</strong>；" +
      "勾选后同一快捷键为<strong>V2 预览</strong>，V1 可从 V2 内入口打开）。",
    shortcutsLabel: "快捷",
    shortcutsHelp:
      "Ctrl+Shift+S 打开设置；Ctrl+Shift+F 打开插件管理（V1/V2 由下方实验性勾选决定）；Ctrl+Shift+D 开关调试面板。",
    immersiveOnlyNote: "虚拟时间、叙事场景等仅在沉浸模式下显示于「更多」。",
    experimentalLabel: "实验性功能",
    experimentalSectionHelp:
      "灰度入口：用于预览新版插件管理界面（V2）。若当前构建未集成 V2，会继续使用现有专业模式。",
    experimentalToggleTitle: "启用新版插件管理界面（V2 预览）",
    experimentalToggleHtml:
      "开启后，<strong>Ctrl+Shift+F</strong> 与顶栏「更多」里的插件管理入口将<strong>打开并切换 V2 预览</strong>（设置会记住此项）。" +
      "需要「开发者调试」等完整能力时，请在 V2 内进入<strong>专业模式（V1）</strong>；也可关闭本项恢复默认。",
    openV2Preview: "打开插件管理 V2 预览",
    advancedTitle: "扩展区（settings.advanced）",
    advancedDesc:
      "manifest 中声明 <code>settings.advanced</code> 的插件显示于此。",
    advancedSlotAria: "设置扩展区",
    securityLabel: "安全",
    forceIframeTitle: "强制 iframe 模式",
    forceIframeDesc:
      "开启后，所有插件界面将使用 iframe 加载，更安全但体验可能下降。保存后需重启应用以完全生效。",
    pluginsPanelTitle: "目录插件 · 设置页插槽",
    pluginsPanelHint1:
      "在插件 manifest 的 ui_slots 中声明 slot 为 settings.panel，即可在此嵌入配置页。",
    pluginsPanelHint2:
      "与 chat_toolbar 相同，使用 https://ocliveplugin.localhost/<id>/<entry> 加载；可在插件管理中调整顺序或隐藏。",
    iframeSavedInfo: "已保存。重启应用后强制 iframe 模式将完全生效。",
  },
  common: {
    cancel: "取消",
    close: "关闭",
    loading: "加载中…",
    preparing: "准备中…",
    importPackTitle: "导入角色包",
    chatInputLabel: "输入消息",
    chatPlaceholder: "对 {name} 说点什么...",
    send: "发送",
    sceneTravel: {
      togetherAria: "邀请同行并选择目的地",
      togetherLabel: "检测到邀请同行，请选择目的地",
      postAria: "选择要切换的场景",
      postLabel: "检测到出行或前往意图，请选择目的地",
      pickPlaceholder: "请选择目的地",
      solo: "仅我过去",
      together: "同行前往",
      dismiss: "稍后再说",
    },
    sceneMode: {
      title: "前往「{label}」",
      desc: "仅切换你的叙事视角，或让角色与你同往？",
      solo: "仅我过去（角色留守）",
      together: "同行前往",
    },
    autonomousNotice:
      "系统：虚拟时间变化后，角色场景已从「{from}」切换为「{to}」（叙事视角未自动改变）。",
    shortcutHelp: {
      aria: "快捷键",
      title: "快捷键",
      rowOpenSettings: "打开设置（扩展区、安全、快捷键与插件配置）",
      rowCtrlLong: "打开本快捷键说明",
      rowCtrlLongKeys: "Ctrl（长按约 1 秒）",
      foot: "更多快捷键将随功能迭代补充。",
      slotSectionAria: "启动器插槽",
      slotHeading: "插件槽（launcher.palette）",
      slotEmbedAria: "启动器插槽",
      ctrlShiftFV2:
        "打开插件管理（V2 预览）；关闭设置中的「V2 预览」后恢复为专业模式（V1）",
      ctrlShiftFV1: "打开专业模式（V1）插件与后端管理（含界面插件 · 开发者调试）",
    },
    rolePack: {
      exportFilterName: "OCPak 角色包",
      importFilterName: "OCPak / ZIP",
      exported: "角色包已导出",
      importedOverwrite: "已覆盖并导入角色: {id}",
      imported: "已导入角色: {name}",
      barTitle:
        "安装 .ocpak / .zip 压缩包，或已解压的目录（与 roles/{id}/ 一致）",
      export: "导出角色包",
      importArchive: "导入压缩包",
      importFolder: "从文件夹导入",
      conflictTitle: "角色已存在",
      conflictBody:
        "本地已有角色 ID「{id}」（{name} v{version}）。导入将覆盖该角色目录，是否继续？",
      overwrite: "覆盖导入",
    },
  },
  roleRuntime: {
    personalityProfile: "档案（可变正文由对话维护）",
    personalityVector: "七维向量",
    profileHint1:
      "人格来源为 profile：运行时以核心性格档案与数据库中的「可变性格档案」为准；界面七维多为从正文归纳的视图。",
    profileHint2:
      "与 vector 模式（七维直接参与事件演化）不同；设计说明见仓库 docs/personality-archive-notes.md。",
    vectorHint1:
      "人格来源为 vector：事件与情绪按七维精细化调整；与 settings 中 evolution.personality_source 一致。",
    versionAuthor: "版本 {version} · 作者 {author}",
    personalitySource: "人格来源：",
    backendHintBefore: "模块后端、异地心声、会话覆盖与调试快照已迁至",
    backendLink: "插件与后端管理 → 后端模块",
    backendHintAfter: "（Ctrl+Shift+F）",
    relation: "关系",
    eventImpact: "事件影响",
  },
  editor: {
    personalityTrait: {
      stubbornness: "倔强",
      clinginess: "黏人",
      sensitivity: "敏感",
      assertiveness: "强势",
      forgiveness: "宽容",
      talkativeness: "话多",
      warmth: "温暖",
    },
    chatExport: {
      allRoles: "导出全部角色",
      pluginDebug: "附带插件诊断（单角色）",
      exportJson: "导出 JSON",
      exportTxt: "导出 TXT",
      downloaded: "已下载 {name}",
      success: "导出成功",
      saveCancelled: "已取消保存",
    },
    debug: {
      monologueInserted: "已插入独白",
      monologuePrefix: "【独白】",
      title: "🎛️ 开发面板",
      hint1:
        "供开发与排错：查看好感度、性格维度、近期事件与记忆摘要；可重载策略、生成独白、导入或管理角色包等。",
      hint2:
        "快捷键 Ctrl+Shift+D（同时按住 Ctrl、Shift，再按字母 D）可随时打开或关闭本面板；按 Esc 也可关闭。顶栏「更多」里亦可点「打开调试面板」。",
      dockSlotAria: "调试面板扩展槽",
      insertMonoGenerating: "生成中…",
      insertMono: "插入独白",
      knowledgeTitle: "世界观知识",
      knowledgeIndexed: "包内索引：",
      knowledgeLoaded: "已加载",
      knowledgeNotLoaded: "未加载",
      knowledgeChunks: "· 共 {n} 块",
      knowledgeLastPrompt: "上一句注入 Prompt：",
      knowledgeChunksUnit: "块",
      knowledgeLastPromptLine: "上一句注入 Prompt：{n} 块",
      knowledgeHint:
        "发话后更新「上一句」；点「刷新调试数据」同步包内块数（改磁盘后请先 load_role）。",
      favorability: "好感度",
      personalityVector: "性格向量",
      personalityProfileHelp:
        "当前包为「档案」人格来源：此处七维多为运行时从核心与可变性格档案归纳的视图，便于理解，不是唯一数据源。",
      metaCounts: "事件数: {events} · 记忆数: {memories}",
      recentEvents: "最近事件",
      recentMemories: "最近记忆",
      refresh: "刷新调试数据",
      reloadPolicy: "重载策略",
      footer: "💡 Ctrl+Shift+D 开关面板 · 角色包与独白已收在此",
      fav80: "💖 超级亲密！",
      fav60: "💕 关系很好~",
      fav40: "👍 还不错",
      fav20: "🤝 慢慢熟悉中",
      fav0: "😶 还有点陌生",
      presenceCoPresent: "共景",
      presenceRemoteStub: "异地占位",
      presenceRemoteLife: "异地心声",
    },
  },
  hotkeys: {
    title: "全局快捷键",
    lead:
      "默认全部关闭。启用后由系统全局监听，可能与系统或其它应用冲突；保存失败时会提示原因。",
    fieldAccelerator: "快捷键",
    accelPlaceholder: "如 Ctrl+Shift+L",
    enabled: "启用",
    action: "动作",
    actionOpenLauncher: "打开插件目录列表",
    actionOpenSlot: "打开某插件插槽页",
    pluginId: "插件 id",
    slotName: "插槽名",
    appearanceOptional: "appearance（可选）",
    remove: "删除",
    addRow: "添加一条",
    save: "保存",
    savedToast: "已保存快捷键配置（仅启用的项会注册全局快捷键）。",
  },
};
