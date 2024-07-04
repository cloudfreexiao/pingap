// Copyright 2024 Tree xie.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::http_cache::{CacheObject, HttpCacheStorage};
use super::Result;
use async_trait::async_trait;
use tinyufo::TinyUfo;

pub struct TinyUfoCache {
    cache: TinyUfo<String, CacheObject>,
}

impl TinyUfoCache {
    fn new(total_weight_limit: usize, estimated_size: usize) -> Self {
        Self {
            cache: TinyUfo::new(total_weight_limit, estimated_size),
        }
    }
}

pub fn new_tiny_ufo_cache(
    total_weight_limit: usize,
    estimated_size: usize,
) -> TinyUfoCache {
    TinyUfoCache::new(total_weight_limit, estimated_size)
}

#[async_trait]
impl HttpCacheStorage for TinyUfoCache {
    async fn get(&self, key: &str) -> Option<CacheObject> {
        self.cache.get(&key.to_string())
    }
    async fn put(
        &self,
        key: String,
        data: CacheObject,
        weight: u16,
    ) -> Result<()> {
        self.cache.put(key, data, weight);
        Ok(())
    }
}