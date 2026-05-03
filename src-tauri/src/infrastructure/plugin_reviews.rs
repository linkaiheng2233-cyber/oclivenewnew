use crate::error::Result;
pub use oclive_kernel_runtime::infrastructure::plugin_reviews_index_sync::DEFAULT_PLUGIN_REVIEWS_INDEX_URL;
use oclive_kernel_runtime::infrastructure::plugin_reviews_index_sync::{
    load_plugin_reviews_index_cache, plugin_reviews_index_default_cache_path,
    resolve_plugin_reviews_index_url, sync_plugin_reviews_index_from_url,
};
pub use oclive_kernel_runtime::models::plugin_reviews_index::{
    PluginReviewEntry, PluginReviewsIndexFile,
};
use std::path::Path;

pub fn load_cached_plugin_reviews_index(app_data_dir: &Path) -> Result<PluginReviewsIndexFile> {
    let p = plugin_reviews_index_default_cache_path(app_data_dir);
    load_plugin_reviews_index_cache(&p)
}

pub fn sync_plugin_reviews_index_online(
    app_data_dir: &Path,
    url: Option<&str>,
) -> Result<PluginReviewsIndexFile> {
    let u = resolve_plugin_reviews_index_url(url);
    let p = plugin_reviews_index_default_cache_path(app_data_dir);
    sync_plugin_reviews_index_from_url(&u, &p)
}
