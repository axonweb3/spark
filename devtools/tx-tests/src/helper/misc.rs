use std::fs;
use std::path::Path;

use crate::ROCKSDB_PATH;

pub fn remove_smt() {
    if Path::new(ROCKSDB_PATH).is_dir() {
        fs::remove_dir_all(ROCKSDB_PATH).unwrap();
    }
}
