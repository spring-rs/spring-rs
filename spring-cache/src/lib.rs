mod moka;
mod redis;

use dashmap::DashMap;
use spring::async_trait;

#[derive(Default)]
pub struct GenericCacheManager<C> {
    caches: DashMap<String, C>,
}

impl<C> GenericCacheManager<C> {
    #[inline]
    fn get_cache<S: Into<String>>(&self, cache_name: S, cache_supplier: impl FnOnce() -> C) -> &C {
        self.caches
            .entry(cache_name.into())
            .or_insert_with(cache_supplier)
            .value()
    }

    #[inline]
    fn cache_names(&self) -> Vec<String> {
        self.caches.iter().map(|e| e.key().clone()).collect()
    }
}

#[async_trait]
pub trait CacheTrait<K, V> {
    fn name(&self) -> String;

    async fn get(&self, key: &K) -> Option<V>;

    async fn put(&self, key: K, value: V);

    async fn evict(&self, key: &K) -> Option<V>;
}
