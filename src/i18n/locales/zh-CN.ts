export default {
  app: {
    locale: {
      label: "界面语言",
      system: "跟随系统",
      zhCN: "中文",
      enUS: "English",
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
  },
};
