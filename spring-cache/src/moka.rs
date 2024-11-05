//! https://github.com/moka-rs/moka
//!

use crate::{CacheManagerTrait, CacheTrait};
use dashmap::DashMap;
use moka::future::Cache;
use spring::async_trait;
use std::hash::Hash;

#[derive(Default)]
pub struct MokaCacheManager<K: Hash + Eq, V> {
    caches: DashMap<String, MokaCache<K, V>>,
}

impl<K, V> CacheManagerTrait<K, V, MokaCache<K, V>> for MokaCacheManager<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Sync + Send + 'static,
{
    #[inline]
    fn get_cache<S: Into<String>>(&self, cache_name: S) -> MokaCache<K, V> {
        // self.caches.entry(cache_name).or_insert_with(||)
        todo!()
    }

    #[inline]
    fn cache_names(&self) -> Vec<String> {
        self.caches.iter().map(|e| e.key().clone()).collect()
    }
}

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
