mod moka;
mod redis;

use spring::async_trait;

pub trait CacheManagerTrait<K, V, C: CacheTrait<K, V>> {
    fn get_cache<S: Into<String>>(cache_name: S) -> C;

    fn cache_names() -> Vec<String>;
}

#[async_trait]
pub trait CacheTrait<K, V> {
    fn name(&self) -> String;

    async fn get(&self, key: &K) -> V;

    async fn put(&self, key: K, value: V);

    async fn evict(&self, key: K);
}
