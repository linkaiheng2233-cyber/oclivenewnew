/** app — zh. */
export default {
  app: {
    locale: {
      label: '界面语言',
      system: '跟随系统',
      zhCN: '中文',
      enUS: 'English',
    },
    connectivity: {
      pluginIndexOffline: 'GitHub 插件索引（plugins.json）当前无法联网更新，已使用本机缓存。',
      dismiss: '关闭提示',
    },
    theme: {
      system: '跟随系统',
      light: '浅色',
      dark: '深色',
    },
    defaultRoleName: '沐沐',
    /** 浏览器页签标题（与 index.html 内联引导脚本保持一致） */
    documentTitle: '沐沐 - 桌面AI伴侣',
    more: {
      collapse: '收起',
      more: '更多',
      ariaMoreFeatures: '更多功能',
      interactionMode: '互动模式',
      interactionImmersiveHint:
        '剧情模式：时间流动、场景切换，久不来关系也会慢慢变化。',
      interactionPureChatHint: '日常聊：专注对话与性格，界面更简洁，适合第一印象。',
      interactionImmersive: '剧情模式',
      interactionPureChat: '日常聊',
      identity: '身份',
      identityHelp:
        '与角色相处时的关系身份（如朋友、恋人等），影响对话与关系数值；与包内「核心性格档案」不同，后者写在 core_personality.txt。',
      ui: '界面',
      uiHint1: '字号 A− / A+ 与编写器、启动器使用同一套档位，会保存在本机。',
      uiHint2: '主题为浅色 / 深色 / 跟随系统，亦会记住。',
      appearanceToolbar: '外观与字号',
      scaleGroup: '界面大小',
      shrinkTitle: '缩小',
      shrinkAria: '缩小界面',
      scaleRelativeTitle: '相对默认字号：{label}',
      enlargeTitle: '放大',
      enlargeAria: '放大界面',
      themeTitle: '主题：{label}（点击切换）',
      settingsEntry: '设置入口',
      shortcutHelp: '快捷键说明',
      openSettings: '⚙ 设置',
      debug: '调试',
      debugHelp:
        '开发者与排错用：好感、记忆、策略重载等。Ctrl+Shift+D 可开关调试窗；顶栏「更多」展开时按 Esc 先收起本栏。',
      openDebugPanel: '打开调试面板',
      virtualTime: '虚拟时间',
      virtualTimeHint1: '故事内的时间，与真实时钟独立。点击时间可打开滚轮调整。',
      virtualTimeHint2: '可用快捷按钮推进时间；部分角色包会在跳转后触发场景或独白。',
      narrativeScene: '叙事场景',
      narrativeSceneHelp:
        '你当前叙事的场景；与角色包中的场景配置一致。切换后可能触发历史记录折叠分界。',
      characterAt: '角色在：{label}',
      pluginBtnSimple: '插件管理',
      pluginMarket: '插件市场',
      modelManager: '模型管理',
      settingsTileHelp:
        '将快捷键说明、设置、插件与模型管理集中到同一处。Ctrl+Shift+S 设置；Ctrl+Shift+F 已安装插件；'
        + 'Ctrl+Shift+M 模型管理；Ctrl+Shift+D 调试面板。',
    },
    toast: {
      remoteLifeOn: '异地心声已开启',
      remoteLifeOff: '异地心声已关闭',
      interactionImmersive: '已切换为剧情模式',
      interactionPureChat: '已切换为日常聊',
      pluginsStoryModeOnly: '插件功能仅在剧情模式下可用',
      layoutResetOk: '已恢复为角色包推荐布局。',
      layoutResetFailPrefix: '恢复失败：',
      noRolesScanned:
        '未扫描到任何可用角色包（roles 目录为空或全部校验失败）。请检查宿主使用的 roles 路径：开发可设置环境变量 OCLIVE_ROLES_DIR 指向仓库的 roles 文件夹。',
      fallbackReply:
        '本次为备用回复（云端/本地模型未连通）。请在「模型管理」确认 API Key、模型名，并在插件相关设置中授予出站网络权限后重试。',
      chatPersistFailed: '回复已发送，但聊天记录保存失败，刷新后可能看不到本条消息。',
      roleSwitched: '已切换角色: {id}',
      relationSetPerScene: '已设置当前场景身份：{name}',
      relationSetGlobal: '已设置身份：{name}',
      pluginInstalledFromWeb: '已通过网页链接安装插件：{id}',
      pluginFilesChanged: '检测到插件变更，已自动刷新',
    },
    sceneTransition: {
      going: '正在前往「{label}」…',
    },
    sidebar: {
      favorability: '好感度',
      lifeNow: '此刻：{label}',
      scheduleInference: '日程推断',
    },
    floatingSlot: '浮层插件区',
    narrativeAside: {
      aria: '叙事与内心',
      title: '叙事与内心',
    },
    scene: {
      selectDestinationFirst: '请先选择目的地',
      toastTogether: '已前往（同行）',
      toastNarrativeOnly: '已切换叙事场景（角色未移动）',
      systemLine:
        '叙事场景已切换为「{narrative}」；角色仍位于「{character}」。',
    },
    hotkeyHost: {
      pluginDialogAria: '插件快捷窗口',
      notFoundDialogAria: '插件未找到',
      cannotOpenTitle: '无法打开插件页',
      notFoundBody:
        '当前角色 bootstrap 中未找到 {plugin} 在槽 {slot} 的界面；请确认插件已启用、未隐藏该槽贡献，并已保存插件配置。',
      launcherDialogAria: '可启动插件',
      launcherTitle: '插件目录',
      noPlugins: '未扫描到插件。',
    },
    helpHintAria: '查看说明',
    roleSelector: {
      role: '🎭 角色',
      identity: '👤 身份',
    },
  }
}
