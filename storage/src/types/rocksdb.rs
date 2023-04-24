use std::path::PathBuf;

use serde::Deserialize;

pub const DEFAULT_CACHE_SIZE: usize = 100;

fn default_cache_size() -> usize {
    DEFAULT_CACHE_SIZE
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigRocksDB {
    pub max_open_files: i32,
    #[serde(default = "default_cache_size")]
    pub cache_size:     usize,
    pub options_file:   Option<PathBuf>,
}

impl Default for ConfigRocksDB {
    fn default() -> Self {
        Self {
            max_open_files: 64,
            cache_size:     default_cache_size(),
            options_file:   None,
        }
    }
}
