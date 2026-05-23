use lru::LruCache;
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

const DEFAULT_MAX_CAPACITY: usize = 1000;

/// 缓存条目
#[derive(Clone, Debug)]
struct CacheEntry<T: Clone> {
    data: T,
    created_at: Instant,
    ttl: Option<Duration>,
}

impl<T: Clone> CacheEntry<T> {
    fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.created_at.elapsed() > ttl
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
    data: Arc<Mutex<LruCache<String, CacheEntry<T>>>>,
}

impl<T: Clone + Send + Sync> Cache<T> {
    /// 创建新缓存（默认最多 1000 条）
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_MAX_CAPACITY)
    }

    /// 创建带容量上限的缓存
    #[must_use]
    pub fn with_capacity(max_capacity: usize) -> Self {
        let cap = NonZeroUsize::new(max_capacity.max(1)).unwrap_or(NonZeroUsize::MIN);
        Self {
            data: Arc::new(Mutex::new(LruCache::new(cap))),
        }
    }

    /// 获取缓存值
    ///
    /// 如果缓存过期，会自动清理并返回 None
    #[must_use]
    pub fn get(&self, key: &str) -> Option<T> {
        let mut cache = self.data.lock();
        match cache.get(key) {
            Some(entry) if !entry.is_expired() => Some(entry.data.clone()),
            Some(_) => {
                cache.pop(key);
                None
            }
            None => None,
        }
    }

    /// 设置缓存值（无过期时间）
    pub fn set(&self, key: String, value: T) {
        self.set_with_ttl(key, value, None);
    }

    /// 设置缓存值（带过期时间）
    pub fn set_with_ttl(&self, key: String, value: T, ttl: Option<Duration>) {
        let entry = CacheEntry {
            data: value,
            created_at: Instant::now(),
            ttl,
        };
        self.data.lock().put(key, entry);
    }

    /// 删除缓存
    pub fn remove(&self, key: &str) {
        self.data.lock().pop(key);
    }

    /// 清空所有缓存
    pub fn clear(&self) {
        self.data.lock().clear();
    }

    /// 获取缓存大小
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.lock().len()
    }

    /// 检查缓存是否为空
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.lock().is_empty()
    }

    /// 清理过期缓存
    pub fn cleanup_expired(&self) {
        let mut cache = self.data.lock();
        let expired: Vec<String> = cache
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(k, _)| k.clone())
            .collect();
        for key in expired {
            cache.pop(&key);
        }
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
        let ttl = Some(Duration::from_millis(100));
        cache.set_with_ttl("key1".to_string(), "value1".to_string(), ttl);

        assert_eq!(cache.get("key1"), Some("value1".to_string()));

        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(cache.get("key1"), None);
    }

    #[test]
    fn test_cache_respects_max_capacity() {
        let cache: Cache<u32> = Cache::with_capacity(2);
        cache.set("a".to_string(), 1);
        cache.set("b".to_string(), 2);
        assert_eq!(cache.len(), 2);
        cache.set("c".to_string(), 3);
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get("c"), Some(3));
    }

    #[test]
    fn test_cache_lru_evicts_least_recently_used() {
        let cache: Cache<u32> = Cache::with_capacity(2);
        cache.set("a".to_string(), 1);
        cache.set("b".to_string(), 2);
        assert_eq!(cache.get("a"), Some(1));
        cache.set("c".to_string(), 3);
        assert_eq!(cache.get("a"), Some(1));
        assert_eq!(cache.get("b"), None);
        assert_eq!(cache.get("c"), Some(3));
    }
}
