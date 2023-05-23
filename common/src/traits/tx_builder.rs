use anyhow::Result;
use async_trait::async_trait;
use ckb_sdk::rpc::ckb_indexer::Cell;
use ckb_types::core::TransactionView;
use ckb_types::H256;

use crate::traits::ckb_rpc_client::CkbRpc;
use crate::types::tx_builder::*;

// todo: the parameters of the new method have not stabilized yet

#[async_trait]
pub trait IInitTxBuilder<C: CkbRpc> {
    fn new(
        ckb: CkbNetwork<C>,
        seeder_key: PrivateKey,
        max_supply: Amount,
        checkpoint: Checkpoint,
        metadata: Metadata,
        type_ids: TypeIds,
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
        metadata_type_id: H256,
        xudt_args: H256,
        staker: EthAddress,
        current_epoch: Epoch,
        stake: StakeItem,
        delegate: Option<StakeDelegate>,
    ) -> Self;

    async fn build_tx(&self) -> Result<TransactionView>;
}

#[async_trait]
pub trait IDelegateTxBuilder {
    fn new(delegator: EthAddress, current_epoch: Epoch, delegate_info: Vec<DelegateItem>) -> Self;

    async fn build_tx(&self) -> Result<TransactionView>;
}

#[async_trait]
pub trait IWithdrawTxBuilder {
    fn new(user: EthAddress, current_epoch: Epoch) -> Self;

    async fn build_tx(&self) -> Result<TransactionView>;
}

#[async_trait]
pub trait IRewardTxBuilder {
    fn new(
        user: EthAddress,
        current_epoch: Epoch,
        base_reward: u128,
        half_reward_cycle: Epoch,
        theoretical_propose_count: u64,
    ) -> Self;

    async fn build_txs(&self) -> Result<Vec<TransactionView>>;
}

#[async_trait]
pub trait ICheckpointTxBuilder {
    fn new(kicker: PrivateKey, checkpoint: Checkpoint) -> Self;

    async fn build_tx(&self) -> Result<TransactionView>;
}

#[async_trait]
pub trait IMetadataTxBuilder<PSmt> {
    fn new(kicker: PrivateKey, quorum: u16, last_checkpoint: Checkpoint, smt: PSmt) -> Self;

    async fn build_tx(&self) -> Result<(TransactionView, NonTopStakers, NonTopDelegators)>;
}

#[async_trait]
pub trait IStakeSmtTxBuilder {
    fn new(kicker: PrivateKey, current_epoch: Epoch, quorum: u16, stake_cells: Vec<Cell>) -> Self;

    async fn build_tx(&self) -> Result<(TransactionView, NonTopStakers)>;
}

#[async_trait]
pub trait IDelegateSmtTxBuilder {
    fn new(
        kicker: PrivateKey,
        current_epoch: Epoch,
        quorum: u16,
        delegate_cells: Vec<Cell>,
    ) -> Self;

    async fn build_tx(&self) -> Result<(TransactionView, NonTopDelegators)>;
}
