use anyhow::Result;
use ckb_jsonrpc_types::Uint32;
use ckb_types::packed::{Byte32, CellDep, OutPoint, Script};
use ckb_types::prelude::{Builder, Entity, Pack};

use common::traits::ckb_rpc_client::CkbRpc;
use common::types::ckb_rpc_client::{Cell, Order, ScriptType, SearchKey, SearchKeyFilter};
use common::types::tx_builder::{Amount, NetworkType};
use common::utils::convert::*;

use crate::ckb::define::constants::TOKEN_BYTES;
use crate::ckb::define::scripts::*;
use crate::ckb::NETWORK_TYPE;
use crate::{cell_dep, out_point, script};

pub struct Xudt;

impl Xudt {
    pub fn type_(owner_lock_hash: &Byte32) -> Script {
        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => script!(
                &XUDT_TYPE_MAINNET.code_hash,
                XUDT_TYPE_MAINNET.hash_type,
                owner_lock_hash.as_bytes()
            ),
            NetworkType::Testnet => script!(
                &XUDT_TYPE_TESTNET.code_hash,
                XUDT_TYPE_TESTNET.hash_type,
                owner_lock_hash.as_bytes()
            ),
            NetworkType::Devnet => script!(
                &XUDT_TYPE_DEVNET.code_hash,
                XUDT_TYPE_DEVNET.hash_type,
                owner_lock_hash.as_bytes()
            ),
        }
    }

    pub fn type_dep() -> CellDep {
        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => cell_dep!(
                &XUDT_TYPE_MAINNET.tx_hash,
                XUDT_TYPE_MAINNET.index,
                XUDT_TYPE_MAINNET.dep_type
            ),
            NetworkType::Testnet => cell_dep!(
                &XUDT_TYPE_TESTNET.tx_hash,
                XUDT_TYPE_TESTNET.index,
                XUDT_TYPE_TESTNET.dep_type
            ),
            NetworkType::Devnet => cell_dep!(
                &XUDT_TYPE_DEVNET.tx_hash,
                XUDT_TYPE_DEVNET.index,
                XUDT_TYPE_DEVNET.dep_type
            ),
        }
    }

    pub async fn collect(
        ckb_rpc: &impl CkbRpc,
        owner_lock: Script,
        xudt: Script,
        expected_amount: Amount,
    ) -> Result<(Vec<Cell>, Amount)> {
        let mut after = None;
        let limit = Uint32::from(20);
        let search_key = SearchKey {
            script:               owner_lock.into(),
            script_type:          ScriptType::Lock,
            filter:               Some(SearchKeyFilter {
                script: Some(xudt.into()),
                ..Default::default()
            }),
            script_search_mode:   None,
            with_data:            Some(true),
            group_by_transaction: None,
        };

        let mut cells = vec![];
        let mut total = 0;

        while total < expected_amount {
            let result = ckb_rpc
                .get_cells(search_key.clone(), Order::Asc, limit, after)
                .await?;
            result.objects.into_iter().for_each(|cell| {
                if total < expected_amount {
                    total +=
                        new_u128(&cell.output_data.as_ref().unwrap().as_bytes()[..TOKEN_BYTES]);
                    cells.push(cell);
                }
            });
            if result.last_cursor.is_empty() {
                break;
            }
            after = Some(result.last_cursor);
        }

        Ok((cells, total))
    }
}
