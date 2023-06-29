use anyhow::Result;
use async_trait::async_trait;
use ckb_types::core::TransactionView;
use ckb_types::H256;

use crate::traits::ckb_rpc_client::CkbRpc;
use crate::traits::smt::{
    DelegateSmtStorage, ProposalSmtStorage, RewardSmtStorage, StakeSmtStorage,
};
use crate::types::ckb_rpc_client::Cell;
use crate::types::tx_builder::*;

#[async_trait]
pub trait IInitTxBuilder<C: CkbRpc> {
    fn new(
        ckb: CkbNetwork<C>,
        seeder_key: PrivateKey,
        max_supply: Amount,
        checkpoint: Checkpoint,
        metadata: Metadata,
    ) -> Self;

    async fn build_tx(&self) -> Result<(TransactionView, TypeIds)>;
}

#[async_trait]
pub trait IMintTxBuilder<C: CkbRpc> {
    fn new(
        ckb: CkbNetwork<C>,
        seeder_key: PrivateKey,
        stakers: Vec<(StakerEthAddr, Amount)>,
        selection_type_id: H256,
        issue_type_id: H256,
    ) -> Self;

    async fn build_tx(&self) -> Result<TransactionView>;
}

#[async_trait]
pub trait IStakeTxBuilder<C: CkbRpc> {
    fn new(
        ckb: CkbNetwork<C>,
        type_ids: StakeTypeIds,
        staker: EthAddress,
        current_epoch: Epoch,
        stake: StakeItem,
        first_stake_info: Option<FirstStakeInfo>,
    ) -> Self;

    async fn build_tx(&self) -> Result<TransactionView>;
}

#[async_trait]
pub trait IDelegateTxBuilder<C: CkbRpc> {
    fn new(
        ckb: CkbNetwork<C>,
        type_ids: StakeTypeIds,
        delegator: EthAddress,
        current_epoch: Epoch,
        delegate_info: Vec<DelegateItem>,
    ) -> Self;

    async fn build_tx(&self) -> Result<TransactionView>;
}

#[async_trait]
pub trait IWithdrawTxBuilder<C: CkbRpc> {
    fn new(
        ckb: CkbNetwork<C>,
        type_ids: StakeTypeIds,
        user: EthAddress,
        current_epoch: Epoch,
    ) -> Self;

    async fn build_tx(&self) -> Result<TransactionView>;
}

#[async_trait]
pub trait IRewardTxBuilder<C, S>
where
    C: CkbRpc,
    S: RewardSmtStorage + StakeSmtStorage + DelegateSmtStorage + ProposalSmtStorage,
{
    fn new(
        ckb: CkbNetwork<C>,
        type_ids: RewardTypeIds,
        smt: S,
        info: RewardInfo,
        user: EthAddress,
        current_epoch: Epoch,
    ) -> Self;

    async fn build_tx(&self) -> Result<TransactionView>;
}

#[async_trait]
pub trait ICheckpointTxBuilder<C>
where
    C: CkbRpc,
{
    async fn new(
        kicker_key: PrivateKey,
        ckb: CkbNetwork<C>,
        type_ids: CheckpointTypeIds,
        epoch_len: u64,
        new_checkpoint: Checkpoint,
        proof: CheckpointProof,
    ) -> Self;

    async fn build_tx(&self) -> Result<TransactionView>;
}

#[async_trait]
pub trait IMetadataTxBuilder<PSmt> {
    fn new(
        kicker: PrivateKey,
        quorum: u16,
        last_metadata: Metadata,
        last_checkpoint: Checkpoint,
        smt: PSmt,
    ) -> Self;

    async fn build_tx(&self) -> Result<(TransactionView, NonTopStakers, NonTopDelegators)>;
}

#[async_trait]
pub trait IStakeSmtTxBuilder<C: CkbRpc, S: StakeSmtStorage> {
    fn new(
        ckb: CkbNetwork<C>,
        kicker: PrivateKey,
        current_epoch: Epoch,
        type_ids: StakeSmtTypeIds,
        quorum: u16,
        stake_cells: Vec<Cell>,
        stake_smt_storage: S,
    ) -> Self;

    async fn build_tx(&self) -> Result<(TransactionView, NonTopStakers)>;
}

#[async_trait]
pub trait IDelegateSmtTxBuilder<C: CkbRpc, D: DelegateSmtStorage> {
    fn new(
        ckb: CkbNetwork<C>,
        kicker: PrivateKey,
        current_epoch: Epoch,
        type_ids: DelegateSmtTypeIds,
        delegate_at_cells: Vec<Cell>,
        delegate_smt_storage: D,
    ) -> Self;

    async fn build_tx(&mut self) -> Result<(TransactionView, NonTopDelegators)>;
}
