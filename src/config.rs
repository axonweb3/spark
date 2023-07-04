use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::{fs, io};

use ckb_types::H256;
use common::types::tx_builder::NetworkType;
use serde::{de, Deserialize};

#[derive(Clone, Debug, Deserialize)]
pub struct SparkConfig {
    pub private_key:        String,
    pub ckb_node_url:       String,
    pub rpc_listen_address: SocketAddr,
    pub rdb_url:            String,
    pub kvdb_path:          PathBuf,
    pub network_type:       NetworkType,
    pub metadata_type_id:   H256,
    pub checkpoint_type_id: H256,
}

/// Parse a config from reader.
pub fn parse_reader<R: io::Read, T: de::DeserializeOwned>(r: &mut R) -> Result<T, ParseError> {
    let mut buf = String::new();
    r.read_to_string(&mut buf)?;
    Ok(toml::from_str(&buf)?)
}

/// Parse a config from file.
///
/// Note: In most cases, function `parse` is better.
pub fn parse_file<T: de::DeserializeOwned>(name: impl AsRef<Path>) -> Result<T, ParseError> {
    let mut f = fs::File::open(name)?;
    parse_reader(&mut f)
}

#[derive(Debug)]
pub enum ParseError {
    IO(io::Error),
    Deserialize(toml::de::Error),
}

impl From<io::Error> for ParseError {
    fn from(error: io::Error) -> ParseError {
        ParseError::IO(error)
    }
}

impl From<toml::de::Error> for ParseError {
    fn from(error: toml::de::Error) -> ParseError {
        ParseError::Deserialize(error)
    }
}
