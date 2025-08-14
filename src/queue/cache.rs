use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Instant;

/// Generic cache with TTL support
pub struct TtlCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    cache: Arc<Mutex<HashMap<K, (V, Instant)>>>,
}

impl<K, V> TtlCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get(&self, key: &K, ttl_secs: u64) -> Option<V> {
        let mut cache = self.cache.lock().await;
        if let Some((value, timestamp)) = cache.get(key) {
            if timestamp.elapsed().as_secs() < ttl_secs {
                return Some(value.clone());
            } else {
                // Remove expired entry
                cache.remove(key);
            }
        }
        None
    }

    pub async fn insert(&self, key: K, value: V) {
        let mut cache = self.cache.lock().await;
        cache.insert(key, (value, Instant::now()));
    }

    pub async fn cleanup_expired(&self, ttl_secs: u64) {
        let mut cache = self.cache.lock().await;
        let now = Instant::now();
        cache.retain(|_, (_, timestamp)| now.duration_since(*timestamp).as_secs() < ttl_secs);
    }

    pub async fn size(&self) -> usize {
        self.cache.lock().await.len()
    }

    pub async fn clear(&self) {
        self.cache.lock().await.clear();
    }
}

impl<K, V> Default for TtlCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}
