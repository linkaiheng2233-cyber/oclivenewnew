/** pluginManager — zh. */
export default {
  pluginManager: {
    legend: {
      enabled: '已启用：当前配置可直接生效',
      pending: '还需配置：通常缺少目录插件 ID',
      disabled: '已关闭：当前链路未启用',
    },
    source: {
      session_override: '会话覆盖',
      env_override: '环境覆盖',
      pack_default: '角色包默认',
    },
    risk: {
      needsConfig: '缺配置',
      envFirst: '环境优先',
    },
    nav: {
      explorerAria: '筛选（工作区风格）',
      title: '资源管理器',
      subtitle: '筛选视图',
      rootTooltip: '仅用于 UI 层级展示，不代表磁盘路径',
      treeAria: '筛选树',
      byModule: '按模块',
      byBackend: '按实现',
      byStatus: '按状态',
    },
    search: {
      placeholder: '搜索：例如 远程、情绪、目录插件',
      empty: '没有匹配项，试试更短的关键词。',
    },
    detail: {
      readonlyNotice:
        '只读说明：此处不会写入任何配置；请在环境变量或角色包中修改后重载应用。',
      previewNotice:
        '变更预览：点击下方「应用改动」后写入当前会话（不修改角色包 settings.json；若与环境变量冲突，以环境解析为准）。',
      expand: '展开',
      collapse: '收起',
      placeholder: '先从中间列表选一个卡片。',
    },
    env: { label: '环境变量' },
    cards: {
      optionPackDefault: '跟随角色包默认（{backend}）',
      llmMain: {
        title: '对话回复引擎',
        description: '决定回复模型来源：本地模型、远程服务或目录插件。',
      },
      llmEndpoint: {
        title: 'LLM 远程地址说明',
        description: '选择远程服务时，优先读取 LLM 专用地址。',
        summary: '建议在系统环境变量配置地址，便于迁移与排错。',
        fieldLlmUrl: 'LLM 专用远程地址（优先）',
        fieldPluginUrl: '通用远程地址（兜底）',
      },
      emotionMain: {
        title: '情绪推理引擎',
        description: '控制情绪由内置逻辑、远程服务或目录插件处理。',
      },
      emotionEndpoint: {
        title: 'Emotion 远程地址说明',
        description: '情绪 remote 默认读取通用远程地址。',
        summary: '建议在系统环境变量配置地址，避免写死到角色包。',
        fieldPluginUrl: 'Emotion 常用远程入口',
      },
      complexSwitch: {
        title: '复杂情感开关',
        description: '开启后启用异地心声链路，复杂情感表现更明显。',
        sessionOn: '当前会话已开启',
        sessionOff: '当前会话已关闭',
        label: '启用复杂情感（异地心声）',
        hint: '开启后建议配置 URL 与 TOKEN 环境变量。',
      },
      complexEndpoint: {
        title: '复杂情感地址说明',
        description: '复杂情感服务通常独立部署，支持鉴权 token。',
        summary: '若服务要求鉴权，请同时配置 URL 和 TOKEN。',
        fieldUrl: '复杂情感服务地址',
        fieldToken: '复杂情感服务鉴权 Token',
      },
    },
    apply: {
      endpointNoSave: '地址说明项无需保存，请在环境变量中配置。',
      remoteLifeUpdated: '复杂情感开关已更新。',
      unsupported: '当前仅支持 LLM / Emotion 配置写入。',
      sessionSaved: '配置已写入当前会话。',
    },
    /** V1 已安装区 · 单插件工作区 */
    installed: {
      privateSettings: '插件私有设置',
      advanced: '高级',
      debugWorkbench: '调试台',
    },
    legendAria: '状态说明',
    v1ListItem: {
      aria: '插件 {id}',
      kindShell: '整壳',
      kindSlot: '插槽',
      uiSlots: 'UI 插槽: {list}',
      depsUnmet: '依赖未满足（{status}）：{issues}',
      disablePlugin: '停用插件',
      hideToolbarEmbed: '隐藏工具栏嵌入',
      hideSettingsEmbed: '隐藏设置页嵌入',
      hideRoleDetailEmbed: '隐藏角色详情嵌入',
      hideSidebarEmbed: '隐藏侧边栏嵌入',
      hideChatHeaderEmbed: '隐藏聊天头部嵌入',
    },
    v1Backend: {
      leadBefore: '以下为 ',
      leadAfter: ' 的包默认与会话级覆盖；不写入磁盘角色包。',
      leadPath: 'settings.json → plugin_backends',
      moduleLine:
        '模块后端：mem {mem} · emotion {emotion} · event {event} · prompt {prompt} · llm {llm} · agent {agent}',
      sessionEffectiveLine:
        '会话生效：mem {mem} · emotion {emotion} · event {event} · prompt {prompt} · llm {llm} · agent {agent}',
      sessionOverrideHint: '当前会话已启用模块覆盖（仅本会话生效，不写入角色包）。',
      sourcesLine:
        '来源：mem {mem} · emotion {emotion} · event {event} · prompt {prompt} · llm {llm} · agent {agent}',
      titleModule: 'settings.json → plugin_backends',
      titleSession: '会话生效',
      titleSources: '来源',
      remoteLife: '异地心声',
      packDefaultSuggestOn: '包默认建议开',
      followPackDefault: '跟随包默认（{value}）',
      localMemPlaceholder: 'provider_id，空串清除本会话覆盖',
      applySession: '应用到本会话',
      debugSnapshot: '调试快照',
      refresh: '刷新',
      copy: '复制',
      packPlugin: '打包插件',
      oneClickPack: '一键打包（agent/llm）',
      copyOk: '已复制',
      copyFail: '复制失败',
      packNeedTarget: '请先在目录插件槽位中配置目标插件（agent 或 llm）。',
      packDone: '打包完成：{path}（sha256={sha}…）',
      directoryPluginsPack: '包 · directory_plugins：{summary}',
      directoryPluginsEffective: '生效 · directory_plugins：{summary}',
    },
    slotsAria: {
      settingsPanelTablist: '插件设置页',
      settingsPanelEmpty: '暂无声明 {slot} 插槽的插件。',
      sidebarList: '侧边栏插件插槽',
      roleDetailList: '角色详情插件插槽',
      chatHeaderStrip: '聊天页顶部插件插槽',
      chatToolbar: '插件工具栏',
    },
    v2: {
      slotKey: '实例键',
      packBackend: '包默认后端',
    },
    pmSlot: {
      appearance: '外观',
      defaultVariant: '默认',
      hideSlot: '隐藏本槽',
    },
    template: {
      directoryIdPh: '例如 my-plugin-id',
      directoryManualPick: '手动输入或选择候选',
    },
    slotEmbed: {
      defaultAria: '插件嵌入区',
    },
    errorPlaceholder: {
      retry: '加载失败，点击重试',
      fallback: '使用 HTML 版本',
      viewDetails: '查看详情',
    },
    privateSettings: {
      loading: '加载设置…',
      noFields: '该插件未声明 uiSchema.fields。',
      templatePrefix: '模板：',
      saving: '保存中…',
      save: '保存私有配置',
      toastSaved: '已保存插件私有配置。',
    },
  },
  pluginTerms: {
    module: {
      llm: '对话大脑（LLM）',
      emotion: '情绪引擎（Emotion）',
      complex_emotion: '复杂情感（Complex Emotion）',
    },
    category: {
      all: '全部功能',
      module: '按模块',
      type: '按实现方式',
      status: '按状态',
    },
    type: {
      builtin: '内置',
      remote: '远程',
      directory: '本地目录插件',
    },
    status: {
      enabled: '已启用',
      disabled: '已关闭',
      needs_config: '还需配置',
    },
    backend: {
      follow_default: '跟随角色包默认',
      ollama: 'Ollama（本地模型）',
      remote: '远程服务',
      directory: '目录插件',
      builtin: '内置',
    },
    field: {
      backend: '运行方式',
      directory_plugin: '目录插件 ID',
      remote_life: '异地心声',
    },
    hint: {
      directory_id_empty: '留空会清空会话覆盖，回到角色包默认。',
      endpoint_env: '地址建议放在环境变量里，便于迁移与排错。',
    },
    action: {
      apply: '应用改动',
      open_v1: '打开专业模式（V1）',
      close: '关闭',
    },
    title: { v2: '插件与后端管理 V2（简易模式）' },
    subtitle: {
      v2:
        '轻量概览：按 slot_registry 实例展示后端与「本次覆盖」；完整增删槽位、分组与写盘请在插件管理专业模式（Ctrl+Shift+F）。专家路由 UI 暂未挂载。',
    },
  }
}
