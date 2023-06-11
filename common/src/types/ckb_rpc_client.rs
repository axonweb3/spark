use std::future::Future;

use anyhow::Result;
use ckb_jsonrpc_types::{
    BlockNumber, Capacity, CellOutput, JsonBytes, OutPoint, Script, Uint32, Uint64,
};
use ckb_sdk::rpc::ckb_indexer::Cell as CkbCell;
use ckb_types::H256;
use serde::{Deserialize, Serialize};

pub type RPC<T> = dyn Future<Output = Result<T>>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IndexerTip {
    pub block_hash:   H256,
    pub block_number: BlockNumber,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    Desc,
    Asc,
}

#[derive(Serialize, Deserialize)]
pub struct Pagination<T> {
    pub objects:     Vec<T>,
    pub last_cursor: JsonBytes,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CellType {
    Input,
    Output,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TxWithCell {
    pub tx_hash:      H256,
    pub block_number: BlockNumber,
    pub tx_index:     Uint32,
    pub io_index:     Uint32,
    pub io_type:      CellType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TxWithCells {
    pub tx_hash:      H256,
    pub block_number: BlockNumber,
    pub tx_index:     Uint32,
    pub cells:        Vec<(CellType, Uint32)>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Tx {
    Ungrouped(TxWithCell),
    Grouped(TxWithCells),
}

impl Tx {
    pub fn tx_hash(&self) -> H256 {
        match self {
            Tx::Ungrouped(tx) => tx.tx_hash.clone(),
            Tx::Grouped(tx) => tx.tx_hash.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum IndexerScriptSearchMode {
    /// Mode `prefix` search script with prefix
    Prefix,
    /// Mode `exact` search script with exact match
    Exact,
}

impl Default for IndexerScriptSearchMode {
    fn default() -> Self {
        Self::Prefix
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchKey {
    pub script:               Script,
    pub script_type:          ScriptType,
    pub script_search_mode:   Option<IndexerScriptSearchMode>,
    pub filter:               Option<SearchKeyFilter>,
    pub with_data:            Option<bool>,
    pub group_by_transaction: Option<bool>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct SearchKeyFilter {
    pub script:                Option<Script>,
    pub script_len_range:      Option<[Uint64; 2]>,
    pub output_data_len_range: Option<[Uint64; 2]>,
    pub output_capacity_range: Option<[Uint64; 2]>,
    pub block_range:           Option<[BlockNumber; 2]>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScriptType {
    Lock,
    Type,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CellsCapacity {
    pub capacity:     Capacity,
    pub block_hash:   H256,
    pub block_number: BlockNumber,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Cell {
    pub output:       CellOutput,
    pub output_data:  Option<JsonBytes>,
    pub out_point:    OutPoint,
    pub block_number: BlockNumber,
    pub tx_index:     Uint32,
}

impl From<Cell> for CkbCell {
    fn from(cell: Cell) -> Self {
        Self {
            output:       cell.output,
            output_data:  cell.output_data,
            out_point:    cell.out_point,
            block_number: cell.block_number,
            tx_index:     cell.tx_index,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct RpcSearchKey {
    pub script:             Script,
    pub script_type:        ScriptType,
    pub script_search_mode: Option<IndexerScriptSearchMode>,
    pub filter:             Option<RpcSearchKeyFilter>,
}

impl RpcSearchKey {
    pub fn into_key(self, block_range: Option<[Uint64; 2]>) -> SearchKey {
        SearchKey {
            script:               self.script,
            script_type:          self.script_type,
            filter:               if self.filter.is_some() {
                self.filter.map(|f| f.into_filter(block_range))
            } else {
                Some(RpcSearchKeyFilter::default().into_filter(block_range))
            },
            script_search_mode:   self.script_search_mode,
            with_data:            None,
            group_by_transaction: Some(true),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Hash, PartialEq, Eq)]
pub struct RpcSearchKeyFilter {
    pub script:                Option<Script>,
    pub script_len_range:      Option<[Uint64; 2]>,
    pub output_data_len_range: Option<[Uint64; 2]>,
    pub output_capacity_range: Option<[Uint64; 2]>,
}

impl RpcSearchKeyFilter {
    fn into_filter(self, block_range: Option<[Uint64; 2]>) -> SearchKeyFilter {
        SearchKeyFilter {
            script: self.script,
            script_len_range: self.script_len_range,
            output_data_len_range: self.output_data_len_range,
            output_capacity_range: self.output_capacity_range,
            block_range,
        }
    }
}

pub trait TipState {
    fn load(&self) -> &BlockNumber;
    fn update(&mut self, current: BlockNumber);
}
