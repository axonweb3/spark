use std::sync::Arc;

use common::types::{relation_db::transaction::Model, smt::Address};
use common::utils::convert::{to_ckb_h160, to_u128, to_u32, to_u8};
use common::Result;
use common::{
    traits::{
        api::APIAdapter,
        async_trait,
        ckb_rpc_client::CkbRpc,
        query::TransactionStorage,
        smt::{DelegateSmtStorage, RewardSmtStorage, StakeSmtStorage},
    },
    types::{axon_types, tx_builder::DelegateRequirement},
};

use molecule::prelude::Entity;
use tx_builder::ckb::helper::Delegate;
use tx_builder::ckb::METADATA_TYPE_ID;

#[derive(Clone)]
pub struct DefaultAPIAdapter<C, T, S> {
    ckb_rpc_client:   Arc<C>,
    relation_storage: Arc<T>,
    _smt_storage:     Arc<S>,
}

impl<C, T, S> DefaultAPIAdapter<C, T, S>
where
    C: CkbRpc + 'static,
    T: TransactionStorage + 'static,
    S: StakeSmtStorage + DelegateSmtStorage + RewardSmtStorage + 'static,
{
    pub fn new(ckb_rpc_client: Arc<C>, relation_storage: Arc<T>, smt_storage: Arc<S>) -> Self {
        Self {
            ckb_rpc_client,
            relation_storage,
            _smt_storage: smt_storage,
        }
    }
}

#[async_trait]
impl<C, T, S> APIAdapter for DefaultAPIAdapter<C, T, S>
where
    C: CkbRpc + 'static,
    T: TransactionStorage + Sync + Send + 'static,
    S: StakeSmtStorage + DelegateSmtStorage + RewardSmtStorage + Sync + Send + 'static,
{
    async fn get_records_by_address(
        &self,
        addr: Address,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Model>> {
        self.relation_storage
            .get_records_by_address(addr, offset, limit)
            .await
    }

    async fn get_operation_history(
        &self,
        addr: Address,
        operation: u32,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Model>> {
        self.relation_storage
            .get_operation_history(addr, operation, offset, limit)
            .await
    }

    async fn get_stake_amount_by_epoch(
        &self,
        operation: u32,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Model>> {
        self.relation_storage
            .get_stake_amount_by_epoch(operation, offset, limit)
            .await
    }

    async fn get_top_stake_address(&self, operation: u32) -> Result<Vec<Model>> {
        self.relation_storage.get_top_stake_address(operation).await
    }

    async fn get_address_state(&self, addr: Address) -> Result<Vec<Model>> {
        self.relation_storage.get_address_state(addr).await
    }

    async fn get_latest_stake_transactions(&self, offset: u64, limit: u64) -> Result<Vec<Model>> {
        self.relation_storage
            .get_latest_stake_transactions(offset, limit)
            .await
    }

    async fn get_stake_requirement_info(&self, addr: Address) -> Result<DelegateRequirement> {
        let req_type_script =
            Delegate::requirement_type((*METADATA_TYPE_ID).load().as_ref(), &to_ckb_h160(&addr));
        let cell =
            Delegate::get_requirement_cell(self.ckb_rpc_client.as_ref(), req_type_script).await?;
        let cell_data = cell.output_data.unwrap().as_bytes().to_vec();
        let req = axon_types::delegate::DelegateRequirement::new_unchecked(cell_data.into());

        Ok(DelegateRequirement {
            commission_rate:    to_u8(&req.commission_rate()),
            maximum_delegators: to_u32(&req.max_delegator_size()),
            threshold:          to_u128(&req.threshold()),
        })
    }
}
