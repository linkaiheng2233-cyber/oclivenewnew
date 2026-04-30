export const zhCN = {
  common: {
    save: "保存",
    close: "关闭",
    cancel: "取消",
    confirm: "确认",
    continue: "继续",
    refresh: "刷新",
    security: "安全",
    advanced: "高级",
  },
  app: {
    startup: {
      loadingRoleAndPlugins: "正在加载角色与插件…",
      scanningRolePacks: "正在扫描角色包…",
      loadingRoleData: "正在加载角色数据…",
      initializingPlugins: "正在初始化插件…",
      failed: "启动失败，请检查角色与插件配置。",
      noRolesFound:
        "未扫描到任何可用角色包（roles 目录为空或全部校验失败）。请检查宿主使用的 roles 路径：开发可设置环境变量 OCLIVE_ROLES_DIR 指向仓库的 roles 文件夹。",
    },
    topBar: {
      more: {
        open: "更多",
        collapse: "收起",
        regionLabel: "更多功能",
      },
      tiles: {
        interactionMode: {
          title: "互动模式",
          hint: [
            "沉浸：启用虚拟时间、叙事场景、日程推断与位移相关能力。",
            "纯聊：只保留对话，隐藏场景与时间条，适合日常闲聊。",
          ],
          immersive: "沉浸",
          pureChat: "纯聊",
        },
        identity: {
          title: "身份",
          hint:
            "与角色相处时的关系身份（如朋友、恋人等），影响对话与关系数值；与包内「核心性格档案」不同，后者写在 core_personality.txt。",
        },
        appearance: {
          title: "界面",
          hint: [
            "字号 A− / A+ 与编写器、启动器使用同一套档位，会保存在本机。",
            "主题为浅色 / 深色 / 跟随系统，亦会记住。",
          ],
          toolbarLabel: "外观与字号",
          scaleLabel: "界面大小",
          shrink: "缩小",
          shrinkAria: "缩小界面",
          enlarge: "放大",
          enlargeAria: "放大界面",
          relativeScaleTitle: "相对默认字号：{label}",
          themeTitle: "主题：{label}（点击切换）",
          themeSystem: "跟随系统",
          themeDark: "深色",
          themeLight: "浅色",
        },
        settingsEntry: {
          title: "设置入口",
          groupLabel: "设置入口集合",
          shortcutHelp: "快捷键说明",
          settings: "⚙ 设置",
          pluginMarket: "插件市场（Ctrl+Shift+A）",
        },
        rolePackShare: {
          title: "角色包（朋友分享）",
          hint: [
            "从朋友处收到 .ocpak/.zip 后，点「导入压缩包」即可直接使用。",
            "roles.json 索引属于可选能力：不依赖官方，也可使用自建/社区源。",
          ],
        },
        debug: {
          title: "调试",
          hint:
            "开发者与排错用：好感、记忆、策略重载等。Ctrl+Shift+D 可开关调试窗；顶栏「更多」展开时按 Esc 先收起本栏。",
          openPanel: "打开调试面板",
        },
        virtualTime: {
          title: "虚拟时间",
          hint: [
            "故事内的时间，与真实时钟独立。点击时间可打开滚轮调整。",
            "可用快捷按钮推进时间；部分角色包会在跳转后触发场景或独白。",
          ],
        },
      },
    },
  },
  settings: {
    title: "设置",
    sectionsNavLabel: "设置分区",
    tabs: {
      general: "常规",
      plugins: "插件扩展",
    },
    language: {
      label: "语言",
      options: {
        system: "跟随系统",
        zhCN: "中文",
        enUS: "English",
      },
      hint: "立即生效；“跟随系统”会使用操作系统语言。",
    },
    shortcuts: {
      label: "快捷",
      immersiveHint: "虚拟时间、叙事场景等仅在沉浸模式下显示于「更多」。",
    },
    experimental: {
      label: "实验性功能",
    },
  },
  pluginManager: {
    entry: {
      settingsGeneralLeadHtml:
        "插件管理正在迭代中。为减少干扰，部分入口默认收起或隐藏。",
      settingsShortcutsHelpHint:
        "可通过顶栏「更多」或 Ctrl+Shift+F 快速打开插件管理。",
      settingsExperimentalSectionHelpHint:
        "实验性功能可能频繁变更，并可能造成兼容性问题。",
      settingsExperimentalToggleTitle: "启用新版插件管理界面（V2 预览）",
      settingsExperimentalToggleDescriptionHtml:
        "这是预览版本，部分功能可能不完整；欢迎反馈。",
      settingsOpenV2PreviewButtonLabel: "打开 V2 预览窗口",
    },
    moreMenu: {
      pluginButtonLabel: {
        v1: "Oclive Manager（V1）",
        v2: "Oclive Manager（V2）",
      },
      tileHelpText: {
        v1:
          "将快捷键说明、设置页与 Oclive Manager（插件与后端管理）集中到同一处。快捷键：Ctrl+Shift+S 打开设置；Ctrl+Shift+F 打开专业模式（V1）Oclive Manager（含开发者调试）；Ctrl+Shift+A 打开插件市场。 Ctrl+Shift+D 开关调试面板。",
        v2:
          "将快捷键说明、设置页与 Oclive Manager 集中到同一处。快捷键：Ctrl+Shift+S 打开设置；Ctrl+Shift+F 与下方按钮打开 Oclive Manager（V2 预览）；Ctrl+Shift+A 打开插件市场；在设置中关闭「V2 预览」可恢复专业模式（V1）。 Ctrl+Shift+D 开关调试面板。",
      },
    },
    shortcuts: {
      ctrlShiftFDescription: {
        v1: "打开专业模式（V1）Oclive Manager（含界面插件 · 开发者调试）",
        v2: "打开 Oclive Manager（V2 预览）；关闭设置中的「V2 预览」后恢复为专业模式（V1）",
      },
      ctrlShiftADescription:
        "打开插件市场（V1/V2 一致）：在线索引、安装、模块/Profile、本地投放导入等",
    },
  },
  pluginManagerV2: {
    slots: {
      settingsPanel: "设置页（插件设置）",
      settingsPlugins: "插件管理页内嵌",
      settingsAdvanced: "设置页（高级扩展区）",
      sidebar: "左侧边栏",
      roleDetail: "角色详情",
      chatHeader: "聊天顶部",
      chatToolbar: "聊天工具栏",
      overlayFloating: "悬浮层",
      launcherPalette: "启动器（快捷入口）",
      debugDock: "调试面板",
    },
    permissions: {
      risk: {
        high: "高风险",
        medium: "中风险",
        low: "低风险",
        unknown: "未知",
      },
      toastUpdated: "权限已更新。",
      toastNoDeclared: "该插件未声明任何权限。",
      confirmGrantAll:
        "将授予该插件全部声明权限（共 {n} 条）。\n\n提示：只给你信任的插件授权。\n\n继续吗？",
      toastGrantedAll: "已授予全部声明权限。",
      toastNoMissing: "该插件没有缺失权限。",
      confirmGrantMissing:
        "将补齐该插件缺失的声明权限（共 {n} 条）：\n\n{list}\n\n继续吗？",
      toastGrantedMissing: "已补齐缺失权限。",
    },
    slotDashboard: {
      toastSaved: "已保存：插槽位置与启用状态已写入配置。",
    },
    gitInstall: {
      confirm:
        "将从 Git 仓库安装插件：\n{url}\n\n提示：请仅安装你信任的来源；安装后如遇“权限不足”报错，请在插件管理里授权。继续吗？",
      toastInstalled: "已安装：{id}",
    },
  },
  pluginMarketV2: {
    lead: "与专业模式（V1）相同的在线索引与安装流程。本地文件夹投放与「扫描投放目录」仍在 V1 的社区索引区块。",
    preflight: {
      dialogLabel: "应用前确认",
      hint: "确认后将开始同步索引并进入逐插件的权限确认流程。",
      confirmAndContinue: "确认并继续",
    },
    permConsent: {
      dialogLabel: "插件安装权限确认",
      hint: "请选择你愿意授予的权限（安装后仍可在「专业模式 → 已安装插件 → 权限」中随时调整）。",
      loadingTokenInfo: "正在加载权限说明…",
      selectAll: "全选",
      selectNone: "全不选",
      continueInstall: "继续安装",
    },
    communityIndex: {
      title: "社区索引（插件市场）",
      entryTypeLabel: "市场条目类型",
    },
    tabs: {
      plugin: "插件",
      module: "模块",
      profile: "Profile",
    },
    sources: {
      official: "官方默认索引",
      thirdParty: "第三方源 · {s}",
    },
    sync: {
      syncing: "同步中…",
      syncOnlineIndex: "同步在线索引",
    },
    offlineMode: "当前为离线模式（使用本地缓存索引）。",
    thirdPartyWarning: "当前为第三方索引源。请仅安装你信任的来源，并谨慎授予权限（开发者模式功能）。",
    emptyHint: "尚无索引数据，请点击「同步在线索引」。",
    pager: {
      toolbarLabel: "市场分页",
      summary: "共 {total} 条 · 第 {page} / {pages} 页",
      pageSize: "每页",
      pageSizeAria: "每页条数",
      prev: "上一页",
      next: "下一页",
    },
  },
  shortcutHelp: {
    dialogLabel: "快捷键",
    title: "快捷键",
    rows: {
      ctrlShiftS: "打开设置（扩展区、安全、快捷键与插件配置）",
      ctrlHoldKey: "Ctrl（长按约 1 秒）",
      ctrlHoldDesc: "打开本快捷键说明",
    },
    footer: "更多快捷键将随功能迭代补充。",
    launcherSlot: {
      aria: "启动器插槽",
      title: "插件槽（launcher.palette）",
      embedAria: "启动器插槽",
    },
  },
  debugPanel: {
    title: "🎛️ 开发面板",
    hint: [
      "供开发与排错：查看好感度、性格维度、近期事件与记忆摘要；可重载策略、生成独白、导入或管理角色包等。",
      "快捷键 Ctrl+Shift+D（同时按住 Ctrl、Shift，再按字母 D）可随时打开或关闭本面板；按 Esc 也可关闭。顶栏「更多」里亦可点「打开调试面板」。",
    ],
    debugDockSlotAria: "调试面板扩展槽",
    monologue: {
      prefix: "【独白】",
      inserted: "已插入独白",
      generating: "生成中…",
      insert: "插入独白",
    },
    knowledge: {
      title: "世界观知识",
      packIndex: "包内索引：",
      loaded: "已加载",
      notLoaded: "未加载",
      totalChunks: "共 {n} 块",
      lastInjected: "上一句注入 Prompt：",
      hint: "发话后更新「上一句」；点「刷新调试数据」同步包内块数（改磁盘后请先 load_role）。",
      presence: {
        coPresent: "共景",
        remoteStub: "异地占位",
        remoteLife: "异地心声",
      },
    },
    favorability: {
      title: "好感度",
      status: {
        superClose: "💖 超级亲密！",
        veryGood: "💕 关系很好~",
        ok: "👍 还不错",
        gettingToKnow: "🤝 慢慢熟悉中",
        strangers: "😶 还有点陌生",
      },
    },
    personalityVector: {
      title: "性格向量",
      profileHint:
        "当前包为「档案」人格来源：此处七维多为运行时从核心与可变性格档案归纳的视图，便于理解，不是唯一数据源。",
    },
  },
  pluginMarketV1: {
    localKinds: {
      rolePack: "角色包",
      pluginArchive: "插件包",
      pluginDir: "插件目录",
      moduleJson: "模块条目",
      profileJson: "Profile",
    },
    localJson: {
      toastCopied: "已复制 JSON 内容到剪贴板。",
      errors: {
        mustBeObject: "JSON 须为对象。",
        typeMustBeModuleOrProfile: '本地条目 type 必须为 "module" 或 "profile"。',
        missingRequiredFields: "本地条目必须包含 id/name/version。",
        onlyModuleProfile: "仅支持 module/profile 本地条目。",
      },
    },
    rolePack: {
      confirmOverwriteImport:
        "覆盖导入角色包：{name}（id={id} v{version}）\n\n将替换本机已存在的同 id 角色包内容。确定继续吗？",
      confirmImport:
        "导入角色包：{name}（id={id} v{version}）\n\n确定导入到本机 roles/ 吗？（默认不覆盖同 id）",
      toastImported: "导入成功：{id}",
      toastImportedOverwrite: "覆盖导入成功：{id}",
    },
    perms: {
      confirmGrantAll:
        "{title}\n\n该插件声明权限：\n{list}\n\n继续则默认授予全部权限（安装后仍可在专业模式里调整）。",
    },
    install: {
      offlineBundleTitle: "安装插件（离线包）：{id}",
      zipTitle: "安装插件（ZIP）：{id}",
      dirTitle: "安装插件（目录）：{id}",
      confirmOverwritePlugin:
        "是否允许覆盖已存在的同 id 插件？\n\n插件：{id}\n\n“确定”=覆盖安装；“取消”=若已存在则报错。",
      toastInstalled: "已安装：{id}",
    },
  },
  pluginManagerV1: {
    llama: {
      toastNotFound: "未扫描到目录插件：{id}",
      permConsentTitle: "启用本地 Llama（目录插件）需要授权哪些能力？",
      permConsentTrustSummary:
        "来源：本地目录插件（随发行版附带或由你放入 plugins/）\n说明：启用 LLM 后端至少需要 process:spawn 才能启动本地 sidecar/llama-server。\n如果你要在插件里用 URL 下载模型文件，则还需要 network:*；否则可不勾选，改为手动把 .gguf 放到指定目录。",
      plan: {
        writeSessionOverride: "将写入会话级后端覆盖（仅当前会话）",
        writePermGrants: "将写入权限授权（可随时撤销）",
      },
      preflightTitle: "一键启用本地 Llama（目录插件）",
      toastEnabled: "已启用 Directory LLM：{id}",
    },
    sessionOverride: {
      confirmRollback:
        "回滚会话级后端覆盖（仅当前会话）\n\n来源：{source}\n条目：{label}\n保存时间：{savedAt}\n\n确定回滚吗？",
      toastRolledBack: "已回滚会话级后端覆盖。",
    },
    permissions: {
      risk: {
        high: "高风险",
        medium: "中风险",
        low: "低风险",
        unknown: "未知",
      },
      confirmHighRisk: "你选择了高风险权限：\n\n{list}\n\n确定继续吗？",
    },
    reviews: {
      none: "暂无评价",
      summary: "{avg} 分（{count}）",
      toastCopiedTemplate: "已复制评价模板（JSON）。",
    },
    profile: {
      toastLoaded: "已读取 Profile：{name}",
      toastNoPlugins: "该 Profile 未声明 plugins，已跳过插件安装。",
      toastMarketMissingPlugin: "索引未找到插件：{id}（source={source}）",
      toastApplied: "Profile 已应用：插件安装/权限确认已执行，后端覆盖已写入当前会话。",
    },
    ui: {
      dialogLabel: "插件工作台（专业模式）",
      title: "插件与功能设置",
      proModeBadge: "专业模式",
      proModeBadgeTitle: "面向创作者与排错：目录插件、后端与会话覆盖",
      subtitle:
        "Ctrl+Shift+F 开关本窗口 · Ctrl+Shift+A 打开插件市场 · 保存后插槽/启用状态建议重启应用生效",
      loading: "加载中…",
      tabsAria: "插件与功能分区",
      tabs: {
        plugins: "插件总览",
        backends: "对话后端",
        slots: "界面位置",
      },
      preflight: {
        dialogLabel: "应用前确认",
        hint: "确认后将开始同步索引并进入逐插件的权限确认流程。",
        confirmAndContinue: "确认并继续",
      },
      permConsent: {
        dialogLabel: "插件安装权限确认",
        trustSummaryTitle: "信任摘要",
        hint: "请选择你愿意授予的权限（安装后仍可在“已安装插件 → 权限”中随时调整）。",
        loadingTokenInfo: "正在加载权限说明…",
        selectAll: "全选",
        selectNone: "全不选",
        continueInstall: "继续安装",
      },
      market: {
        title: "插件市场",
        openMarket: "打开插件市场（Ctrl+Shift+A）",
        hint: "市场（社区索引 / 模块 / Profile / 本地投放）已拆分为独立弹窗，避免和管理功能混在一起。",
      },
      persistScope: {
        title: "这些改动保存到哪里？",
        hint: "选「当前角色」只影响现在这个角色；选「全局默认」会变成所有角色的默认值（会和每个角色自己的设置合并）。",
        aria: "插件配置保存范围",
        roleOnly: "仅当前角色",
        globalDefault: "全局默认",
      },
    },
  },
  expertModels: {
    title: "Expert Models（Module 9）",
    subtitle:
      "选择 Base GGUF + LoRA 强度，并可选覆盖 PromptStyle。会话覆盖优先于角色默认；不设置时不改变现有行为。",
    actions: {
      refresh: "刷新",
      backfillFromEffective: "从有效配置回填编辑器",
    },
    source: {
      sessionOverride: "会话覆盖",
      roleDefault: "角色默认",
      rolePackDefault: "角色包默认",
    },
    relative: {
      justNow: "刚刚",
      secondsAgo: "{n}s 前",
      minutesAgo: "{n}m 前",
      hoursAgo: "{n}h 前",
      daysAgo: "{n}d 前",
    },
  },
} as const;

