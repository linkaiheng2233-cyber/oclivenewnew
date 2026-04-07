use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// 缓存条目
#[derive(Clone, Debug)]
struct CacheEntry<T: Clone> {
    data: T,
    created_at: DateTime<Utc>,
    ttl: Option<Duration>,
}

impl<T: Clone> CacheEntry<T> {
    fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            Utc::now() - self.created_at > ttl
        } else {
            false
        }
    }
}

/// 通用缓存管理器
///
/// 提供线程安全的内存缓存，支持 TTL 过期机制
#[derive(Debug)]
pub struct Cache<T: Clone + Send + Sync> {
    data: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
}

impl<T: Clone + Send + Sync> Cache<T> {
    /// 创建新缓存
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 获取缓存值
    ///
    /// 如果缓存过期，会自动清理并返回 None
    pub fn get(&self, key: &str) -> Option<T> {
        let mut cache = self.data.write();

        if let Some(entry) = cache.get(key) {
            if entry.is_expired() {
                cache.remove(key);
                return None;
            }
            return Some(entry.data.clone());
        }

        None
    }

    /// 设置缓存值（无过期时间）
    pub fn set(&self, key: String, value: T) {
        self.set_with_ttl(key, value, None);
    }

    /// 设置缓存值（带过期时间）
    pub fn set_with_ttl(&self, key: String, value: T, ttl: Option<Duration>) {
        let entry = CacheEntry {
            data: value,
            created_at: Utc::now(),
            ttl,
        };
        self.data.write().insert(key, entry);
    }

    /// 删除缓存
    pub fn remove(&self, key: &str) {
        self.data.write().remove(key);
    }

    /// 清空所有缓存
    pub fn clear(&self) {
        self.data.write().clear();
    }

    /// 获取缓存大小
    pub fn len(&self) -> usize {
        self.data.read().len()
    }

    /// 检查缓存是否为空
    pub fn is_empty(&self) -> bool {
        self.data.read().is_empty()
    }

    /// 清理过期缓存
    pub fn cleanup_expired(&self) {
        let mut cache = self.data.write();
        cache.retain(|_, entry| !entry.is_expired());
    }
}

impl<T: Clone + Send + Sync> Default for Cache<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_set_and_get() {
        let cache: Cache<String> = Cache::new();
        cache.set("key1".to_string(), "value1".to_string());

        assert_eq!(cache.get("key1"), Some("value1".to_string()));
        assert_eq!(cache.get("key2"), None);
    }

    #[test]
    fn test_cache_remove() {
        let cache: Cache<String> = Cache::new();
        cache.set("key1".to_string(), "value1".to_string());
        cache.remove("key1");

        assert_eq!(cache.get("key1"), None);
    }

    #[test]
    fn test_cache_clear() {
        let cache: Cache<String> = Cache::new();
        cache.set("key1".to_string(), "value1".to_string());
        cache.set("key2".to_string(), "value2".to_string());
        cache.clear();

        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_len() {
        let cache: Cache<String> = Cache::new();
        cache.set("key1".to_string(), "value1".to_string());
        cache.set("key2".to_string(), "value2".to_string());

        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_cache_ttl_expiration() {
        let cache: Cache<String> = Cache::new();
        let ttl = Some(Duration::milliseconds(100));
        cache.set_with_ttl("key1".to_string(), "value1".to_string(), ttl);

        assert_eq!(cache.get("key1"), Some("value1".to_string()));

        std::thread::sleep(std::time::Duration::from_millis(150));
        assert_eq!(cache.get("key1"), None);
    }
}
