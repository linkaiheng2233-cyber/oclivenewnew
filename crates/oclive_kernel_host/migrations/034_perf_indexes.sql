-- K-PERF-23: hot-path lookups on long_term_memory content search and operation_logs by role.
CREATE INDEX IF NOT EXISTS idx_ltm_role_content ON long_term_memory(role_id, content);
CREATE INDEX IF NOT EXISTS idx_operation_logs_role ON operation_logs(role_id);
