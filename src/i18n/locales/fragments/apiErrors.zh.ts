/** Tauri `[CODE]` → zh copy (see `toFriendlyErrorMessage` in `utils/tauri-api.ts`). */
export default {
  TXN_BEGIN_FAILED: "事务启动失败，请稍后重试。",
  TXN_RUNTIME_ENSURE_FAILED: "角色运行时状态初始化失败。",
  TXN_PERSONALITY_INSERT_FAILED: "性格数据写入失败。",
  TXN_FAVORABILITY_UPDATE_FAILED: "好感度更新失败。",
  TXN_FAVORABILITY_HISTORY_INSERT_FAILED: "好感度历史记录失败。",
  TXN_MEMORY_INSERT_FAILED: "记忆数据保存失败。",
  TXN_SHORT_TERM_INSERT_FAILED: "对话记录写入失败。",
  TXN_SHORT_TERM_TRIM_FAILED: "对话记录整理失败。",
  TXN_EVENT_INSERT_FAILED: "事件写入失败。",
  TXN_FAVORABILITY_READ_FAILED: "好感度读取失败。",
  TXN_COMMIT_FAILED: "事务提交失败，请稍后再试。",
  TXN_ROLLBACK_FAILED: "事务回滚异常，请联系技术支持。",
  DB_ERROR: "数据库操作失败，请稍后重试。",
  IO_ERROR:
    "本地文件读写失败。请检查：① 应用数据目录是否可写（设置 → 常规 → 环境自检）；② 杀毒/权限是否拦截；③ 勿将数据目录放在只读介质。详见 CONFIGURATION_FILES.md。",
  IO_ERROR_HOST_JSON:
    "插件桥返回的数据无法序列化为 JSON，可能是宿主与插件接口不兼容，请查看控制台日志。",
  API_PLUGIN_NOT_FOUND: "未找到该目录插件或插件未扫描到，请检查插件 id 与安装路径。",
  API_PERMISSION_DENIED: "插件权限不足，请在 manifest 中声明所需权限。",
  API_INVALID_MANIFEST: "插件 manifest 无效，请检查 manifest.json。",
  LLM_ERROR:
    "模型调用失败。请确认：① 本机已安装并启动 Ollama；② 终端执行 `ollama list` 且已 `ollama pull` 所需模型；③ 环境变量 `OLLAMA_MODEL` / 角色包内模型名与列表一致；④ `OLLAMA_BASE_URL` 指向正确端口（默认 http://localhost:11434）。设置 → 常规 → 环境自检可快速探测。",
  ROLE_NOT_FOUND: "角色不存在，请确认 role_id 与 `OCLIVE_ROLES_DIR` 下目录结构。",
  ROLE_NOT_FOUND_DETAIL: "角色不存在或找不到 manifest。{detail}",
  ROLE_PACK_EXISTS: "该角色 ID 已存在。若要替换本地版本，请选择覆盖。",
  INVALID_PARAMETER: "参数无效，请检查输入内容。",
  INVALID_PARAMETER_DETAIL: "参数无效：{detail}",
  OLLAMA_TIMEOUT: "沐沐走神了，再问一次吧。",
  TXN_ROLLBACK: "操作失败，请稍后再试。",
  SERDE_ERROR: "数据解析失败，请稍后重试。",
  UNKNOWN_ERROR: "发生未知错误，请稍后重试。",
} as Record<string, string>;
