use crate::error::Result;
use oclive_kernel_runtime::infrastructure::role_market_index_sync::{
    role_market_index_cache_path, sync_role_market_index_from_url, DEFAULT_ROLES_INDEX_URL,
};
pub use oclive_kernel_runtime::infrastructure::role_pack_archive::install_role_pack_from_direct_url;
pub use oclive_kernel_runtime::models::role_market_index::{RoleIndexEntry, RoleIndexFile};
use std::path::Path;

pub async fn sync_role_index_online(
    app_data_dir: &Path,
    source_url: Option<&str>,
) -> Result<RoleIndexFile> {
    let url = source_url.unwrap_or(DEFAULT_ROLES_INDEX_URL);
    let cache = role_market_index_cache_path(app_data_dir, url);
    sync_role_market_index_from_url(url, &cache).await
}
