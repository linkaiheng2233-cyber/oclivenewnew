/** settings — zh. */
export default {
  settings: {
    ariaDialog: '设置',
    ariaNav: '设置分区',
    title: '设置',
    closeAria: '关闭',
    tabGeneral: '常规',
    tabPlugins: '插件扩展',
    tabStorage: '存储管理',
    generalLeadHtml:
      '顶栏<strong>「更多」</strong>集中设置入口；<strong>Ctrl+Shift+S</strong> 打开设置；'
      + '<strong>Ctrl+Shift+F</strong> 已安装插件；<strong>Ctrl+Shift+M</strong> 模型管理。',
    shortcutsLabel: '快捷',
    shortcutsHelp:
      'Ctrl+Shift+S 设置；Ctrl+Shift+F 已安装插件；Ctrl+Shift+M 模型管理；Ctrl+Shift+D 调试面板。',
    immersiveOnlyNote: '虚拟时间、叙事场景等仅在沉浸模式下显示于「更多」。',
    envCheckTitle: '环境自检',
    envCheckHelp:
      '快速探测本机 Ollama 是否可达、角色根目录是否可读、应用数据目录是否可写；不替代完整启动健康检查。',
    envCheckLead:
      '若对话报错或模型无响应，可先运行此项；详细错误码见文档 ERROR_CODES.md §1.5。',
    envCheckRun: '运行检测',
    envCheckRunning: '检测中…',
    envCheckDoneToast: '环境自检已完成。',
    envCheckOllama: 'Ollama（{url}）',
    envCheckOllamaOk: '可达',
    envCheckOllamaFail: '不可达或异常',
    envCheckRoles: '角色根目录',
    envCheckRolesMissing: '不存在',
    envCheckRolesUnreadable: '存在但不可读',
    envCheckRolesOk: '可读',
    envCheckRolesHint:
      '路径来自 OCLIVE_ROLES_DIR 或默认；需为各角色子文件夹的父目录，且子目录含 manifest.json。',
    envCheckAppData: '应用数据目录',
    envCheckAppDataOk: '可写',
    envCheckAppDataFail: '不可写',
    envCheckDetail: '详情：',
    envCheckOllamaPullNote:
      '模型下载与拉取进度请在终端通过 ollama pull 查看；此处仅检测服务是否可达，不显示拉取百分比。',
    sentrySectionTitle: '崩溃诊断（Sentry）',
    sentrySectionLead:
      '仅当本发行构建已注入 DSN 时显示本区。会上报 Vue 侧未捕获异常（不含聊天正文）；Rust 侧仍主要依赖本机日志。',
    sentryOptOutLabel: '禁用崩溃上报',
    sentryOptOutHelp:
      '勾选后立即尝试关闭 Sentry 客户端；偏好保存在本机 localStorage（键 oclive.telemetry.sentryOptOut）。取消勾选后需重启应用才会重新初始化上报。',
    sentryDisabledToast: '已禁用崩溃上报。',
    sentryReenableRestartToast: '已取消禁用；请重启应用后才会恢复上报。',
    remoteFallbackSectionTitle: '远端插件失败策略',
    remoteFallbackLabel: '远端 HTTP 失败时自动降级内置',
    remoteFallbackHelp:
      '关闭后，若角色包将记忆/情绪/事件/Prompt/LLM 等槽设为 remote 且侧车不可达，将返回错误码 REMOTE_SERVICE_UNAVAILABLE，而不再静默用内置实现。与「高风险网络授权」互补：授权决定能否发起出站请求，本项决定在失败出口是否允许降级。可用环境变量 <code>OCLIVE_REMOTE_FALLBACK_TO_BUILTIN</code> 覆盖（设置后本开关对进程内有效值锁定）。',
    remoteFallbackEnvLocked: '已设置环境变量，进程内以此为准；数据库值仍可保存供未设置环境时生效。',
    remoteFallbackSavedToast: '已保存。',
    advancedTitle: '扩展区（settings.advanced）',
    advancedDesc:
      'manifest 中声明 <code>settings.advanced</code> 的插件显示于此。',
    advancedSlotAria: '设置扩展区',
    securityLabel: '安全',
    forceIframeTitle: '强制 iframe 模式',
    forceIframeDesc:
      '开启后，所有插件界面将使用 iframe 加载，更安全但体验可能下降。保存后需重启应用以完全生效。',
    pluginsPanelTitle: '目录插件 · 设置页插槽',
    pluginsPanelHint1:
      '在插件 manifest 的 ui_slots 中声明 slot 为 settings.panel，即可在此嵌入配置页。',
    pluginsPanelHint2:
      '与 chat_toolbar 相同，使用 https://ocliveplugin.localhost/<id>/<entry> 加载；可在插件管理中调整顺序或隐藏。',
    iframeSavedInfo: '已保存。重启应用后强制 iframe 模式将完全生效。',
  },
  hotkeys: {
    title: '全局快捷键',
    lead:
      '默认全部关闭。启用后由系统全局监听，可能与系统或其它应用冲突；保存失败时会提示原因。',
    fieldAccelerator: '快捷键',
    accelPlaceholder: '如 Ctrl+Shift+L',
    enabled: '启用',
    action: '动作',
    actionOpenLauncher: '打开插件目录列表',
    actionOpenSlot: '打开某插件插槽页',
    pluginId: '插件 id',
    slotName: '插槽名',
    appearanceOptional: 'appearance（可选）',
    remove: '删除',
    addRow: '添加一条',
    save: '保存',
    savedToast: '已保存快捷键配置（仅启用的项会注册全局快捷键）。',
  }
}
