pub mod types;

use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

use serde::{de, ser};

pub fn parse_reader<R: io::Read, T: de::DeserializeOwned>(r: &mut R) -> T {
    let mut buf = String::new();
    r.read_to_string(&mut buf).unwrap();
    toml::from_str(&buf).unwrap()
}

pub fn parse_file<T: de::DeserializeOwned>(path: impl AsRef<Path>) -> T {
    let mut f = fs::File::open(path).unwrap();
    parse_reader(&mut f)
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
    use super::{parse_file, write_file};

    fn _test_parse_config() {
        let file_path = "./src/config/priv_keys.toml";
        let priv_keys: PrivKeys = parse_file(file_path);
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

        let type_ids: TypeIds = parse_file(file_path);
        println!("{:?}", type_ids);
    }
}
