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

impl<K: Hash + Eq, V> CacheManagerTrait<K, V, MokaCache<K, V>> for MokaCacheManager<K, V> {
    fn get_cache<S: Into<String>>(cache_name: S) -> MokaCache<K, V> {
        todo!()
    }

    fn cache_names() -> Vec<String> {
        todo!()
    }
}

pub struct MokaCache<K: Hash + Eq, V> {
    name: String,
    cache: Cache<K, V>,
}

#[async_trait]
impl<K, V> CacheTrait<K, V> for MokaCache<K, V>
where
    K: Hash + Eq,
{
    fn name(&self) -> String {
        self.name.clone()
    }

    async fn get(&self, key: &K) -> V {
        self.cache.get(key).await
    }

    async fn put(&self, key: K, value: V) {
        self.cache.insert(key, value).await
    }

    async fn evict(&self, key: K) {
        self.cache.evict(key).await
    }
}
