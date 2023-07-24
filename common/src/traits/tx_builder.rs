use anyhow::Result;
use async_trait::async_trait;
use ckb_types::core::TransactionView;
use ckb_types::H256;
use ethereum_types::H160;

use crate::traits::ckb_rpc_client::CkbRpc;
use crate::traits::smt::{
    DelegateSmtStorage, ProposalSmtStorage, RewardSmtStorage, StakeSmtStorage,
};
use crate::types::ckb_rpc_client::Cell;
use crate::types::tx_builder::*;

use std::collections::HashMap;

#[async_trait]
pub trait IInitTxBuilder<'a, C: CkbRpc> {
    fn new(
        ckb: &'a C,
        seeder_key: PrivateKey,
        max_supply: Amount,
        checkpoint: Checkpoint,
        epoch0_metadata: Metadata,
        epoch1_metadata: Metadata,
        stakers: Vec<StakerEthAddr>,
    ) -> Self;

    async fn build_tx(self) -> Result<(TransactionView, TypeIds)>;
}

#[async_trait]
pub trait IMintTxBuilder<'a, C: CkbRpc> {
    fn new(
        ckb: &'a C,
        seeder_key: PrivateKey,
        stakers: Vec<(StakerEthAddr, Amount)>,
        selection_type_id: H256,
        issue_type_id: H256,
    ) -> Self;

    async fn build_tx(self) -> Result<TransactionView>;
}

#[async_trait]
pub trait IStakeTxBuilder<'a, C: CkbRpc> {
    fn new(
        ckb: &'a C,
        type_ids: StakeTypeIds,
        staker: EthAddress,
        current_epoch: Epoch,
        stake: StakeItem,
        first_stake_info: Option<FirstStakeInfo>,
    ) -> Self;

    async fn build_tx(self) -> Result<TransactionView>;
}

#[async_trait]
pub trait IDelegateTxBuilder<'a, C: CkbRpc> {
    fn new(
        ckb: &'a C,
        type_ids: StakeTypeIds,
        delegator: EthAddress,
        current_epoch: Epoch,
        delegate_info: Vec<DelegateItem>,
    ) -> Self;

    async fn build_tx(self) -> Result<TransactionView>;
}

#[async_trait]
pub trait IWithdrawTxBuilder<'a, C: CkbRpc> {
    fn new(ckb: &'a C, type_ids: StakeTypeIds, user: EthAddress, current_epoch: Epoch) -> Self;

    async fn build_tx(self) -> Result<TransactionView>;
}

#[async_trait]
pub trait IRewardTxBuilder<'a, C, S>
where
    C: CkbRpc,
    S: RewardSmtStorage + StakeSmtStorage + DelegateSmtStorage + ProposalSmtStorage,
{
    fn new(
        ckb: &'a C,
        type_ids: RewardTypeIds,
        smt: S,
        info: RewardInfo,
        user: EthAddress,
        current_epoch: Epoch,
    ) -> Self;

    async fn build_tx(self) -> Result<TransactionView>;
}

#[async_trait]
pub trait ICheckpointTxBuilder<'a, C>
where
    C: CkbRpc,
{
    async fn new(
        ckb: &'a C,
        kicker_key: PrivateKey,
        type_ids: CheckpointTypeIds,
        epoch_len: u64,
        new_checkpoint: Checkpoint,
        proof: CheckpointProof,
    ) -> Self;

    async fn build_tx(self) -> Result<TransactionView>;
}

#[async_trait]
pub trait IMetadataTxBuilder<'a, C, PSmt> {
    fn new(
        ckb: &'a C,
        kicker: PrivateKey,
        type_ids: MetadataTypeIds,
        last_checkpoint: Cell,
        smt: PSmt,
    ) -> Self;

    async fn build_tx(
        self,
    ) -> Result<(
        TransactionView,
        Vec<(H160, u128)>,
        HashMap<H160, HashMap<H160, u128>>,
    )>;
}

#[async_trait]
pub trait IStakeSmtTxBuilder<'a, C: CkbRpc, S: StakeSmtStorage> {
    fn new(
        ckb: &'a C,
        kicker: PrivateKey,
        current_epoch: Epoch,
        type_ids: StakeSmtTypeIds,
        quorum: u16,
        stake_cells: Vec<Cell>,
        stake_smt_storage: S,
    ) -> Self;

    async fn build_tx(self) -> Result<(TransactionView, NonTopStakers)>;
}

#[async_trait]
pub trait IDelegateSmtTxBuilder<'a, C: CkbRpc, D: DelegateSmtStorage> {
    fn new(
        ckb: &'a C,
        kicker: PrivateKey,
        current_epoch: Epoch,
        type_ids: DelegateSmtTypeIds,
        delegate_at_cells: Vec<Cell>,
        delegate_smt_storage: D,
    ) -> Self;

    async fn build_tx(mut self) -> Result<(TransactionView, NonTopDelegators)>;
}
