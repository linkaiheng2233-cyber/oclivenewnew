/** 内核 `KernelErrorBody.code`（`SCREAMING_SNAKE_CASE`）→ 文案（见 `toFriendlyErrorMessage`）。 */
export default {
  EMPTY_MESSAGE: '消息不能为空或仅含空格/换行，请输入至少一个可见字符。',
  INVALID_ROLE_PATH: '角色路径不是有效目录。请传入包含 manifest.json 的角色目录绝对路径。',
  LOAD_ROLE_TASK_PANIC: '加载角色时后台任务异常，请查看日志并重试；若可稳定复现请提 issue。',
  TXN_BEGIN_FAILED: '事务启动失败，请稍后重试。',
  TXN_RUNTIME_ENSURE_FAILED: '角色运行时状态初始化失败。',
  TXN_PERSONALITY_INSERT_FAILED: '性格数据写入失败。',
  TXN_FAVORABILITY_UPDATE_FAILED: '好感度更新失败。',
  TXN_FAVORABILITY_HISTORY_INSERT_FAILED: '好感度历史记录失败。',
  TXN_MEMORY_INSERT_FAILED: '记忆数据保存失败。',
  TXN_SHORT_TERM_INSERT_FAILED: '对话记录写入失败。',
  TXN_SHORT_TERM_TRIM_FAILED: '对话记录整理失败。',
  TXN_EVENT_INSERT_FAILED: '事件写入失败。',
  TXN_FAVORABILITY_READ_FAILED: '好感度读取失败。',
  TXN_COMMIT_FAILED: '事务提交失败，请稍后再试。',
  TXN_ROLLBACK_FAILED: '事务回滚异常，请联系技术支持。',
  TXN_MEMORY_ID_FETCH_FAILED: '记忆写入后读取行号失败，请重试或查看数据库日志。',
  TXN_EVENT_ID_FETCH_FAILED: '事件写入后读取行号失败，请重试或查看数据库日志。',
  TXN_IDENTITY_ENSURE_FAILED: '身份/关系状态初始化失败，请重试。',
  TXN_IDENTITY_FAVOR_UPDATE_FAILED: '身份好感关联更新失败，请重试。',
  TXN_RUNTIME_MIRROR_FAILED: '运行时镜像同步失败，请重试。',
  TXN_MEMORY_FIFO_TRIM_FAILED: '短期记忆整理（FIFO）失败，请重试。',
  DB_ERROR: '数据库操作失败，请稍后重试。',
  IO_ERROR:
    '本地文件读写失败。请检查：① 应用数据目录是否可写（设置 → 常规 → 环境自检）；② 杀毒/权限是否拦截；③ 勿将数据目录放在只读介质。详见 CONFIGURATION_FILES.md。',
  IO_ERROR_HOST_JSON:
    '插件桥返回的数据无法序列化为 JSON，可能是宿主与插件接口不兼容，请查看控制台日志。',
  API_PLUGIN_NOT_FOUND: '未找到该目录插件或插件未扫描到，请检查插件 id 与安装路径。',
  API_PERMISSION_DENIED: '插件权限不足，请在 manifest 中声明所需权限。',
  API_INVALID_MANIFEST: '插件 manifest 无效，请检查 manifest.json。',
  LLM_ERROR:
    '模型调用失败。请确认：① 若 `OCLIVE_LLM_BACKEND=ollama`（默认）：本机已安装并启动 Ollama，`ollama list` / `ollama pull` 与 `OLLAMA_MODEL` 一致，`OLLAMA_BASE_URL` 端口正确；② 若为 **remote**：`OCLIVE_REMOTE_LLM_URL` 可达、超时 `OCLIVE_REMOTE_LLM_TIMEOUT_MS` 合理且上游可用。设置 → 常规 → 环境自检可探测本机 Ollama。',
  VOICE_RPC_TIMEOUT:
    '语音插件调用超时（CosyVoice 预热/合成较慢）。请先在 设置 → 语音交互 点「预热 TTS 侧车」并等待完成；首次合成可能需数分钟。也可提高环境变量 `OCLIVE_VOICE_RPC_TIMEOUT_MS`（默认 speak 600000 / warm 900000 ms）。',
  ROLE_NOT_FOUND: '角色不存在，请确认 role_id 与 `OCLIVE_ROLES_DIR` 下目录结构。',
  ROLE_NOT_FOUND_DETAIL: '角色不存在或找不到 manifest。{detail}',
  ROLE_RUNTIME_NOT_READY:
    '尚未建立本角色的运行时会话（未 load_role 或数据库无 role_runtime）。请先在本界面选择/重新加载该角色后再试。',
  STARTUP_HEALTH_FAILED:
    '启动健康检查未通过：{detail}。可检查角色目录是否含 manifest.json、插件槽配置、数据库是否可写；或暂时设置环境变量 `OCLIVE_SKIP_STARTUP_HEALTH=1` 以跳过（仅排障）。',
  PLUGIN_BACKENDS_DIRECTORY_SLOT:
    '后端槽位配置不完整：使用 directory 类后端时，必须在 `directory_plugins` 中为对应槽填写非空插件 id。请打开插件工作台 → 后端模块或编辑角色包 settings。',
  ROLE_PACK_EXISTS: '该角色 ID 已存在。若要替换本地版本，请选择覆盖。',
  INVALID_PARAMETER: '参数无效，请检查输入内容。',
  INVALID_PARAMETER_DETAIL: '参数无效：{detail}',
  HIGH_RISK_CAPABILITY_NOT_GRANTED:
    '尚未授权该高风险能力（MCP 传输或目录插件子进程）。请在「插件与后端 → Agent 调试」中授权对应项，或由发行版提供显式确认流程。',
  REMOTE_SERVICE_UNAVAILABLE:
    '远端 HTTP 插件或侧车不可用，且当前已关闭「远端失败时自动降级内置」。请检查 `OCLIVE_REMOTE_PLUGIN_URL` / `OCLIVE_REMOTE_LLM_URL` 等是否可达，或在设置 → 常规中重新开启降级；亦可用环境变量 `OCLIVE_REMOTE_FALLBACK_TO_BUILTIN=1`。',
  OLLAMA_TIMEOUT: '沐沐走神了，再问一次吧。',
  TXN_ROLLBACK: '操作失败，请稍后再试。',
  SERDE_ERROR: '数据解析失败，请稍后重试。',
  UNKNOWN_ERROR:
    '发生未知错误。请重试；若与网络或外部服务相关，检查代理/防火墙及环境变量（见 ERROR_CODES §1.6）；仍失败请导出 `oclive_chat` / `oclive_plugin` 日志片段。',
  UNKNOWN_WITH_CODE: '发生错误（{code}）。请重试或查看日志；若界面持续异常可尝试重启应用。',
} as Record<string, string>
