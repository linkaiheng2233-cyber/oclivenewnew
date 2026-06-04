use indexmap::IndexMap;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};

const DEFAULT_MAX_CAPACITY: usize = 1000;

/// Cache entry.
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

/// Bounded insertion-order map (read path does not refresh order; evicts oldest insertions).
#[derive(Debug)]
struct BoundedInsertMap<T: Clone> {
    map: IndexMap<String, CacheEntry<T>>,
    max_capacity: usize,
}

impl<T: Clone> BoundedInsertMap<T> {
    fn with_capacity(max_capacity: usize) -> Self {
        Self {
            map: IndexMap::new(),
            max_capacity: max_capacity.max(1),
        }
    }

    fn len(&self) -> usize {
        self.map.len()
    }

    fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    fn clear(&mut self) {
        self.map.clear();
    }

    fn peek(&self, key: &str) -> Option<&CacheEntry<T>> {
        self.map.get(key)
    }

    fn put(&mut self, key: String, entry: CacheEntry<T>) {
        if self.map.contains_key(&key) {
            self.map.swap_remove(&key);
        }
        self.map.insert(key, entry);
        while self.map.len() > self.max_capacity {
            self.map.shift_remove_index(0);
        }
    }

    fn pop(&mut self, key: &str) -> Option<CacheEntry<T>> {
        self.map.swap_remove(key)
    }

    fn iter(&self) -> impl Iterator<Item = (&String, &CacheEntry<T>)> {
        self.map.iter()
    }
}

/// General-purpose cache manager.
///
/// Thread-safe in-memory cache with TTL expiration.
#[derive(Debug)]
pub struct Cache<T: Clone + Send + Sync> {
    data: Arc<RwLock<BoundedInsertMap<T>>>,
}

impl<T: Clone + Send + Sync> Cache<T> {
    /// Creates a new cache (default max 1000 entries).
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_MAX_CAPACITY)
    }

    /// Creates a cache with a capacity limit.
    #[must_use]
    pub fn with_capacity(max_capacity: usize) -> Self {
        Self {
            data: Arc::new(RwLock::new(BoundedInsertMap::with_capacity(max_capacity))),
        }
    }

    /// Gets a cached value; expired entries are removed under a write lock.
    ///
    /// **LRU semantics (intentionally degraded)**: the read-hit path holds only a read lock and uses `peek`
    /// without refreshing access order. Eviction follows **insertion order**, capacity limit, and TTL—not strict LRU.
    /// Suitable for read-heavy, write-light cases such as `personality_snapshots` where LRU precision is not critical.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<T> {
        {
            let cache = self.data.read();
            match cache.peek(key) {
                Some(entry) if !entry.is_expired() => return Some(entry.data.clone()),
                Some(_) => {}
                None => return None,
            }
        }
        let mut cache = self.data.write();
        match cache.peek(key) {
            Some(entry) if !entry.is_expired() => Some(entry.data.clone()),
            Some(_) => {
                cache.pop(key);
                None
            }
            None => None,
        }
    }

    /// Sets a cache value (no expiration).
    pub fn set(&self, key: String, value: T) {
        self.set_with_ttl(key, value, None);
    }

    /// Sets a cache value with an expiration time.
    pub fn set_with_ttl(&self, key: String, value: T, ttl: Option<Duration>) {
        let entry = CacheEntry {
            data: value,
            created_at: Instant::now(),
            ttl,
        };
        self.data.write().put(key, entry);
    }

    /// Removes a cache entry.
    pub fn remove(&self, key: &str) {
        self.data.write().pop(key);
    }

    /// Retains only entries where `keep(key) == true`.
    pub fn retain(&self, keep: impl Fn(&str) -> bool) {
        let mut cache = self.data.write();
        let remove: Vec<String> = cache
            .iter()
            .filter_map(|(k, _)| if keep(k) { None } else { Some(k.clone()) })
            .collect();
        for k in remove {
            cache.pop(&k);
        }
    }

    /// Clears all cache entries.
    pub fn clear(&self) {
        self.data.write().clear();
    }

    /// Returns the number of cached entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.read().len()
    }

    /// Returns whether the cache is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.read().is_empty()
    }

    /// Removes expired cache entries.
    pub fn cleanup_expired(&self) {
        let mut cache = self.data.write();
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
    fn test_cache_evicts_by_insertion_order_without_read_lru_bump() {
        let cache: Cache<u32> = Cache::with_capacity(2);
        cache.set("a".to_string(), 1);
        cache.set("b".to_string(), 2);
        assert_eq!(cache.get("a"), Some(1));
        cache.set("c".to_string(), 3);
        assert_eq!(cache.get("a"), None);
        assert_eq!(cache.get("b"), Some(2));
        assert_eq!(cache.get("c"), Some(3));
    }
}
