use lru::LruCache;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::Mutex;

use super::Chunker;
use crate::UPOS;

/// Wraps any chunker implementation to add an LRU Cache.
/// Useful for incremental lints.
pub struct CachedChunker<C: Chunker> {
    inner: C,
    cache: Mutex<LruCache<CacheKey, Vec<bool>>>,
}

impl<C: Chunker> CachedChunker<C> {
    pub fn new(inner: C, capacity: NonZeroUsize) -> Self {
        Self {
            inner,
            cache: Mutex::new(LruCache::new(capacity)),
        }
    }
}

impl<C: Chunker> Chunker for CachedChunker<C> {
    fn chunk_sentence(&self, sentence: &[String], tags: &[Option<UPOS>]) -> Vec<bool> {
        let key = CacheKey::new(sentence, tags);

        // Attempt a cache hit.
        // We put this in the block so `read` gets dropped as early as possible.
        if let Ok(mut read) = self.cache.try_lock() {
            if let Some(result) = read.get(&key) {
                return result.clone();
            }
        };

        // We don't want to hold the lock since it may take a while to run the chunker.
        let result = self.inner.chunk_sentence(sentence, tags);

        if let Ok(mut cache) = self.cache.try_lock() {
            cache.put(key, result.clone());
        }

        result
    }
}

#[derive(Hash, PartialEq, Eq)]
struct CacheKey {
    sentence: Vec<String>,
    tags: Vec<Option<UPOS>>,
}

impl CacheKey {
    fn new(sentence: &[String], tags: &[Option<UPOS>]) -> Self {
        Self {
            sentence: sentence.to_vec(),
            tags: tags.to_vec(),
        }
    }
}
