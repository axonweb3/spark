pub mod types;

use std::fs;
use std::io::Write;
use std::path::Path;

use serde::ser;

use common::config_parser::{parse_file, types::ConfigLogger};

use crate::config::types::{PrivKeys, TypeIds};

pub fn parse_priv_keys(path: impl AsRef<Path>) -> PrivKeys {
    let priv_keys: PrivKeys = parse_file(path, false).expect("priv keys");
    priv_keys
}

pub fn parse_type_ids(path: impl AsRef<Path>) -> TypeIds {
    let type_ids: TypeIds = parse_file(path, false).expect("type ids");
    type_ids
}

pub fn parse_log_config(path: impl AsRef<Path>) -> ConfigLogger {
    let config: ConfigLogger = parse_file(path, false).expect("log config");
    config
}

pub fn write_file<T: ser::Serialize>(path: impl AsRef<Path>, content: &T) {
    let toml = toml::to_string(content).unwrap();
    let toml = toml.as_bytes();
    let mut write_file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)
        .unwrap();
    write_file.write_all(toml).unwrap();
    write_file.flush().unwrap();
}

#[cfg(test)]
mod tests {
    use super::types::*;
    use super::{parse_priv_keys, parse_type_ids, write_file};

    fn _test_parse_config() {
        let file_path = "./src/config/priv_keys.toml";
        let priv_keys: PrivKeys = parse_priv_keys(file_path);
        println!("{:?}", priv_keys);
    }

    fn _test_write_config() {
        let file_path = "./src/config/type_ids.toml";

        let type_ids = TypeIds {
            selection_type_id:    TypeId::new(
                "0x9f606e3b29f2a89ee14883c8e172ac8fc9051eb23bae6a80cb82aa562c6e1099",
            ),
            issue_type_id:        TypeId::new(
                "0x9f606e3b29f2a89ee14883c8e172ac8fc9051eb23bae6a80cb82aa562c6e1099",
            ),
            checkpoint_type_id:   TypeId::new(
                "0x9f606e3b29f2a89ee14883c8e172ac8fc9051eb23bae6a80cb82aa562c6e1099",
            ),
            metadata_type_id:     TypeId::new(
                "0x9f606e3b29f2a89ee14883c8e172ac8fc9051eb23bae6a80cb82aa562c6e1099",
            ),
            stake_smt_type_id:    TypeId::new(
                "0x9f606e3b29f2a89ee14883c8e172ac8fc9051eb23bae6a80cb82aa562c6e1099",
            ),
            delegate_smt_type_id: TypeId::new(
                "0x9f606e3b29f2a89ee14883c8e172ac8fc9051eb23bae6a80cb82aa562c6e1099",
            ),
            reward_smt_type_id:   TypeId::new(
                "0x9f606e3b29f2a89ee14883c8e172ac8fc9051eb23bae6a80cb82aa562c6e1099",
            ),
            xudt_owner:           TypeId::new(
                "0x9f606e3b29f2a89ee14883c8e172ac8fc9051eb23bae6a80cb82aa562c6e1099",
            ),
        };
        write_file(file_path, &type_ids);

        let type_ids: TypeIds = parse_type_ids(file_path);
        println!("{:?}", type_ids);
    }
}
