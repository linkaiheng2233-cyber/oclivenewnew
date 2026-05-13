import {
  TAURI_INVOKE_CAPABILITIES,
  type TauriInvokeCapabilityKey,
} from "../gen/tauri-invoke-capabilities";

export { TAURI_INVOKE_CAPABILITIES, type TauriInvokeCapabilityKey };

/** Maps Tauri command names (snake_case) to optional invoke surface keys. */
const COMMAND_CAPABILITY: Readonly<
  Partial<Record<string, TauriInvokeCapabilityKey>>
> = {
  list_mcp_servers: "agent",
  list_mcp_tools: "agent",
  call_mcp_tool: "agent",
  get_agent_debug_traces: "agent",
  clear_agent_debug_traces: "agent",
  preview_mcp_server_import: "agent",
  import_mcp_server_from_path: "agent",

  expert_models_get_effective: "expertModels",
  expert_models_set_session_override: "expertModels",
  expert_models_clear_session_override: "expertModels",
  expert_models_set_role_default: "expertModels",
  expert_models_clear_role_default: "expertModels",
  expert_models_apply_to_session: "expertModels",
  expert_models_validate_graph: "expertModels",
  expert_models_list_local_base_models: "expertModels",
  expert_models_list_local_loras: "expertModels",
  expert_models_import_base_gguf: "expertModels",
  expert_models_import_lora_gguf: "expertModels",
  expert_models_delete_local_base_model: "expertModels",
  expert_models_rename_local_base_model: "expertModels",
  expert_models_set_gguf_repo_meta: "expertModels",
  expert_models_rollback_last_run: "expertModels",
  expert_models_list_runs: "expertModels",
  expert_models_get_run_detail: "expertModels",
  expert_models_clear_runs: "expertModels",
  expert_models_set_run_pinned: "expertModels",
  expert_models_rollback_to_run: "expertModels",
  expert_workflows_list: "expertModels",
  expert_workflows_get: "expertModels",
  expert_workflows_save: "expertModels",
  expert_workflows_delete: "expertModels",
  github_publish_oclexpert_recipe: "expertModels",
  ollama_models_health: "expertModels",
  ollama_models_list_names: "expertModels",
  ollama_models_delete: "expertModels",
  probe_local_llm_runtime: "expertModels",

  sync_role_market_index: "roleMarket",
  install_role_pack_from_market: "roleMarket",

  check_plugin_updates: "pluginMarket",
  extract_plugin_zip: "pluginMarket",
  preview_plugin_zip_permissions: "pluginMarket",
  preview_plugin_dir_permissions: "pluginMarket",
  install_plugin_dir: "pluginMarket",
  sync_plugin_reviews_index: "pluginMarket",
  get_cached_plugin_reviews_index: "pluginMarket",
  sync_plugin_index_command: "pluginMarket",
  get_cached_plugin_index: "pluginMarket",
  install_plugin_from_market: "pluginMarket",
  install_plugin_version_from_market: "pluginMarket",
  install_plugin_from_git: "pluginMarket",
  update_plugin_from_market: "pluginMarket",
  uninstall_plugin_from_market: "pluginMarket",
  batch_update_plugins: "pluginMarket",
  batch_uninstall_plugins: "pluginMarket",
  consume_pending_protocol_installs: "pluginMarket",
  get_plugin_market_sources_config: "pluginMarket",
  set_plugin_market_developer_mode: "pluginMarket",
  set_plugin_index_sources: "pluginMarket",
  get_plugin_audit_logs: "pluginMarket",

  create_plugin_scaffold: "pluginCreator",
  pack_plugin: "pluginCreator",
  spawn_plugin_for_test: "pluginCreator",
  kill_plugin_process: "pluginCreator",
  list_plugin_processes: "pluginCreator",
  get_plugin_logs: "pluginCreator",
  clear_plugin_logs: "pluginCreator",
  test_plugin_method: "pluginCreator",
  discover_plugin_methods: "pluginCreator",
  preview_profile_from_path: "pluginCreator",
};

export function capabilityKeyForCommand(
  command: string,
): TauriInvokeCapabilityKey | undefined {
  return COMMAND_CAPABILITY[command];
}
