//! https://github.com/moka-rs/moka
//!

use crate::CacheTrait;
use moka::future::Cache;
use spring::async_trait;
use std::hash::Hash;

#[derive(Clone)]
pub struct MokaCache<K: Hash + Eq, V> {
    name: String,
    cache: Cache<K, V>,
}

#[async_trait]
impl<K, V> CacheTrait<K, V> for MokaCache<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    #[inline]
    fn name(&self) -> String {
        self.name.clone()
    }

    #[inline]
    async fn get(&self, key: &K) -> Option<V> {
        self.cache.get(key).await
    }

    #[inline]
    async fn put(&self, key: K, value: V) {
        self.cache.insert(key, value).await
    }

    #[inline]
    async fn evict(&self, key: &K) -> Option<V> {
        self.cache.remove(key).await
    }
}
