#!/usr/bin/env bash
# SQLite 数据库滚动备份（oclive_kernel_server 的 OCLIVE_DB_PATH）。
#
# 环境变量：
#   OCLIVE_DB_PATH        必填 — 与运行中内核使用的路径一致。
#   OCLIVE_BACKUP_DIR     默认 /var/backups/oclive
#   OCLIVE_BACKUP_PREFIX  默认 oclive（生成文件名 oclive-YYYYmmdd-HHMMSS.db）
#   OCLIVE_BACKUP_KEEP_DAYS  默认 14 — 早于此天数的备份文件删除（按 mtime）。
#
# 备份策略：
#   • 若系统有 sqlite3，使用 `.backup` 热备（推荐，服务可不停止）。
#   • 否则退化为 cp（仍有读一致性风险；大写入时建议停服务或装 sqlite3）。
#
# cron 示例（每天 3:15，先装 sqlite3）：
#   15 3 * * * OCLIVE_DB_PATH=/opt/oclive/data/oclive.db OCLIVE_BACKUP_DIR=/var/backups/oclive /opt/oclive/bin/backup_kernel_db.sh

set -euo pipefail

db="${OCLIVE_DB_PATH:-}"
if [[ -z "${db}" ]]; then
  echo "OCLIVE_DB_PATH is required" >&2
  exit 1
fi
if [[ ! -f "${db}" ]]; then
  echo "database file not found: ${db}" >&2
  exit 1
fi

dest_dir="${OCLIVE_BACKUP_DIR:-/var/backups/oclive}"
prefix="${OCLIVE_BACKUP_PREFIX:-oclive}"
keep_days="${OCLIVE_BACKUP_KEEP_DAYS:-14}"

mkdir -p "${dest_dir}"
ts="$(date +%Y%m%d-%H%M%S)"
out="${dest_dir}/${prefix}-${ts}.db"

if command -v sqlite3 >/dev/null 2>&1; then
  # 路径勿含空格或单引号；若需复杂路径请改用符号链接到无空格路径。
  sqlite3 "${db}" ".backup ${out}"
else
  echo "warning: sqlite3 not found; using cp (less safe under write load)" >&2
  cp -p "${db}" "${out}"
fi

echo "backup written: ${out}"

# 清理旧备份
if [[ "${keep_days}" =~ ^[0-9]+$ ]] && [[ "${keep_days}" -gt 0 ]]; then
  find "${dest_dir}" -maxdepth 1 -type f -name "${prefix}-*.db" -mtime "+${keep_days}" -print -delete || true
fi
