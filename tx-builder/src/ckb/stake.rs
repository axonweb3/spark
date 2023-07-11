use anyhow::Result;
use async_trait::async_trait;
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellInput, CellOutput, Script},
    prelude::{Entity, Pack},
    H256,
};
use molecule::prelude::Builder;

use common::traits::ckb_rpc_client::CkbRpc;
use common::traits::tx_builder::IStakeTxBuilder;
use common::types::axon_types::{
    delegate::DelegateCellData, stake::StakeAtCellData as AStakeAtCellData,
    withdraw::WithdrawAtCellData,
};
use common::types::ckb_rpc_client::Cell;
use common::types::tx_builder::*;
use common::utils::convert::*;

use crate::ckb::define::constants::*;
use crate::ckb::define::error::{CkbTxErr, CkbTxResult};
use crate::ckb::define::scripts::{
    DELEGATE_REQUIREMENT_TYPE_DEVNET, DELEGATE_REQUIREMENT_TYPE_MAINNET,
    DELEGATE_REQUIREMENT_TYPE_TESTNET,
};
use crate::ckb::define::types::{
    DelegateRequirementArgs, DelegateRequirementInfo, StakeAtCellData, StakeAtCellLockData,
};
use crate::ckb::helper::{
    amount_calculator::*, token_cell_data, Checkpoint, Delegate, Metadata, OmniEth, Secp256k1,
    Stake, Tx, TypeId, Withdraw, Xudt,
};
use crate::ckb::NETWORK_TYPE;

pub struct StakeTxBuilder<'a, C: CkbRpc> {
    ckb:              &'a C,
    type_ids:         StakeTypeIds,
    staker:           EthAddress,
    current_epoch:    Epoch,
    stake:            StakeItem,
    first_stake_info: Option<FirstStakeInfo>,
    stake_lock:       Script,
    token_lock:       Script,
    withdraw_lock:    Script,
    xudt:             Script,
}

#[async_trait]
impl<'a, C: CkbRpc> IStakeTxBuilder<'a, C> for StakeTxBuilder<'a, C> {
    fn new(
        ckb: &'a C,
        type_ids: StakeTypeIds,
        staker: EthAddress,
        current_epoch: Epoch,
        stake_item: StakeItem,
        first_stake_info: Option<FirstStakeInfo>,
    ) -> Self {
        let stake_lock = Stake::lock(&type_ids.metadata_type_id, &staker);
        let withdraw_lock = Withdraw::lock(&type_ids.metadata_type_id, &staker);
        let token_lock = OmniEth::lock(&staker);
        let xudt = Xudt::type_(&type_ids.xudt_owner.pack());

        Self {
            ckb,
            type_ids,
            staker,
            current_epoch,
            stake: stake_item,
            first_stake_info,
            stake_lock,
            token_lock,
            withdraw_lock,
            xudt,
        }
    }

    async fn build_tx(&self) -> Result<TransactionView> {
        if self.stake.inauguration_epoch > self.current_epoch + INAUGURATION {
            return Err(CkbTxErr::InaugurationEpoch {
                expected: self.current_epoch,
                found:    self.stake.inauguration_epoch,
            }
            .into());
        }

        let stake_cell =
            Stake::get_cell(self.ckb, self.stake_lock.clone(), self.xudt.clone()).await?;
        if stake_cell.is_none() {
            self.build_first_stake_tx().await
        } else {
            self.build_update_stake_tx(stake_cell.unwrap()).await
        }
    }
}

impl<'a, C: CkbRpc> StakeTxBuilder<'a, C> {
    async fn build_first_stake_tx(&self) -> Result<TransactionView> {
        let mut inputs = vec![];

        // AT cells
        let token_amount = self.add_token_to_inputs(&mut inputs).await?;

        let mut outputs_data = self.first_stake_data(token_amount)?;
        let mut stake_data = outputs_data[1].clone();

        let mut outputs = vec![
            // AT cell
            CellOutput::new_builder()
                .lock(self.token_lock.clone())
                .type_(Some(self.xudt.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // stake AT cell
            CellOutput::new_builder()
                .lock(self.stake_lock.clone())
                .type_(Some(self.xudt.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[1].len())?)?,
            // delegate requirement cell
            CellOutput::new_builder()
                .lock(self.token_lock.clone())
                .type_(
                    Some(Delegate::requirement_type(
                        &self.type_ids.metadata_type_id,
                        &H256::default(),
                    ))
                    .pack(),
                )
                .build_exact_capacity(Capacity::bytes(outputs_data[2].len())?)?,
        ];

        self.add_withdraw_to_outputs(&mut outputs, &mut outputs_data)
            .await?;

        let cell_deps = vec![
            OmniEth::lock_dep(),
            Secp256k1::lock_dep(),
            Xudt::type_dep(),
            Delegate::requriement_type_dep(),
        ];

        let witnesses = vec![
            OmniEth::witness_placeholder().as_bytes(), // AT cell lock
            OmniEth::witness_placeholder().as_bytes(), // capacity provider lock
        ];

        let tx = TransactionBuilder::default()
            .inputs(inputs)
            .outputs(outputs)
            .outputs_data(outputs_data.pack())
            .cell_deps(cell_deps)
            .witnesses(witnesses.pack())
            .build();

        let mut tx = Tx::new(self.ckb, tx);
        tx.balance(self.token_lock.clone()).await?;

        let tx = tx.inner();
        let mut outputs = tx.outputs().into_iter().collect::<Vec<_>>();
        let mut outputs_data = tx.outputs_data().into_iter().collect::<Vec<_>>();

        let first_input = tx.inputs().get(0).unwrap();
        let requirement_type_id = TypeId::calc(&first_input, 2);
        let network_type = NETWORK_TYPE.load();

        outputs_data[1] = {
            let total_stake_amount = new_u128(&stake_data[..TOKEN_BYTES]);
            let stake_data = AStakeAtCellData::new_unchecked(stake_data.split_off(TOKEN_BYTES));
            let inner_stake_data = stake_data.lock();
            token_cell_data(
                total_stake_amount,
                stake_data
                    .as_builder()
                    .lock(
                        inner_stake_data
                            .as_builder()
                            .requirement_info(
                                DelegateRequirementInfo {
                                    code_hash:   if **network_type == NetworkType::Mainnet {
                                        DELEGATE_REQUIREMENT_TYPE_MAINNET.code_hash.clone()
                                    } else if **network_type == NetworkType::Testnet {
                                        DELEGATE_REQUIREMENT_TYPE_TESTNET.code_hash.clone()
                                    } else {
                                        DELEGATE_REQUIREMENT_TYPE_DEVNET.code_hash.clone()
                                    },
                                    requirement: DelegateRequirementArgs {
                                        metadata_type_hash:  Metadata::type_(
                                            &self.type_ids.metadata_type_id,
                                        )
                                        .calc_script_hash(),
                                        requirement_type_id: requirement_type_id.clone(),
                                    },
                                }
                                .into(),
                            )
                            .build(),
                    )
                    .build()
                    .as_bytes(),
            )
            .pack()
        };

        outputs[2] = tx
            .output(2)
            .unwrap()
            .as_builder()
            .type_(
                Some(Delegate::requirement_type(
                    &self.type_ids.metadata_type_id,
                    &requirement_type_id,
                ))
                .pack(),
            )
            .build();

        let tx = tx
            .as_advanced_builder()
            .set_outputs(outputs)
            .set_outputs_data(outputs_data)
            .build();

        Ok(tx)
    }

    async fn build_update_stake_tx(&self, stake_cell: Cell) -> Result<TransactionView> {
        // stake AT cell
        let mut inputs = vec![CellInput::new_builder()
            .previous_output(stake_cell.out_point.into())
            .build()];

        // AT cells
        let token_amount = self.add_token_to_inputs(&mut inputs).await?;

        let stake_data = stake_cell.output_data.unwrap().into_bytes();
        let outputs_data = self.update_stake_data(token_amount, stake_data)?;

        let outputs = vec![
            // stake AT cell
            CellOutput::new_builder()
                .lock(self.stake_lock.clone())
                .type_(Some(self.xudt.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // AT cell
            CellOutput::new_builder()
                .lock(self.token_lock.clone())
                .type_(Some(self.xudt.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[1].len())?)?,
        ];

        let cell_deps = vec![
            OmniEth::lock_dep(),
            Secp256k1::lock_dep(),
            Xudt::type_dep(),
            Stake::lock_dep(),
            Checkpoint::cell_dep(self.ckb, &self.type_ids.checkpoint_type_id).await?,
            Metadata::cell_dep(self.ckb, &self.type_ids.metadata_type_id).await?,
        ];

        let witnesses = vec![
            Stake::witness(0u8).as_bytes(),            // stake AT cell lock, todo
            OmniEth::witness_placeholder().as_bytes(), // AT cell lock
            OmniEth::witness_placeholder().as_bytes(), // capacity provider lock
        ];

        let tx = TransactionBuilder::default()
            .inputs(inputs)
            .outputs(outputs)
            .outputs_data(outputs_data.pack())
            .cell_deps(cell_deps)
            .witnesses(witnesses.pack())
            .build();

        let mut tx = Tx::new(self.ckb, tx);
        tx.balance(self.token_lock.clone()).await?;

        Ok(tx.inner())
    }

    async fn add_token_to_inputs(&self, inputs: &mut Vec<CellInput>) -> Result<Amount> {
        let (token_cells, amount) = Xudt::collect(
            self.ckb,
            self.token_lock.clone(),
            self.xudt.clone(),
            self.stake.amount,
        )
        .await?;

        if token_cells.is_empty() {
            return Err(CkbTxErr::CellNotFound("AT".to_owned()).into());
        }

        // AT cells
        for token_cell in token_cells.into_iter() {
            inputs.push(
                CellInput::new_builder()
                    .previous_output(token_cell.out_point.into())
                    .build(),
            );
        }

        Ok(amount)
    }

    async fn add_withdraw_to_outputs(
        &self,
        outputs: &mut Vec<CellOutput>,
        outputs_data: &mut Vec<Bytes>,
    ) -> Result<()> {
        let withdraw_cell =
            Withdraw::get_cell(self.ckb, self.withdraw_lock.clone(), self.xudt.clone()).await?;

        if withdraw_cell.is_none() {
            outputs_data.push(token_cell_data(0, WithdrawAtCellData::default().as_bytes()));
            outputs.push(
                CellOutput::new_builder()
                    .lock(self.withdraw_lock.clone())
                    .type_(Some(self.xudt.clone()).pack())
                    .build_exact_capacity(Capacity::bytes(outputs_data.last().unwrap().len())?)?,
            );
        }

        Ok(())
    }

    fn first_stake_data(&self, mut wallet_amount: Amount) -> CkbTxResult<Vec<Bytes>> {
        if !self.stake.is_increase {
            return Err(CkbTxErr::Increase(self.stake.is_increase));
        }

        if wallet_amount < self.stake.amount {
            return Err(CkbTxErr::ExceedWalletAmount {
                wallet_amount,
                amount: self.stake.amount,
            });
        }
        wallet_amount -= self.stake.amount;

        let first_stake = self.first_stake_info.as_ref().ok_or(CkbTxErr::FirstStake)?;

        Ok(vec![
            // AT cell data
            wallet_amount.pack().as_bytes(),
            // stake AT cell data
            token_cell_data(
                self.stake.amount,
                AStakeAtCellData::from(StakeAtCellData {
                    lock: StakeAtCellLockData {
                        l1_pub_key: first_stake.l1_pub_key.clone(),
                        l2_address: self.staker.clone(),
                        bls_pub_key: first_stake.bls_pub_key.clone(),
                        stake_info: self.stake.clone(),
                        ..Default::default()
                    },
                })
                .as_bytes(),
            ),
            // delegate requirement cell data
            DelegateCellData::new_builder()
                .delegate_requirement(first_stake.delegate.clone().into())
                .build()
                .as_bytes(),
        ])
    }

    fn update_stake_data(
        &self,
        wallet_amount: Amount,
        stake_data: Bytes,
    ) -> CkbTxResult<Vec<Bytes>> {
        let total_stake_amount = new_u128(&stake_data[..TOKEN_BYTES]);

        let mut stake_data = stake_data;
        let stake_data = AStakeAtCellData::new_unchecked(stake_data.split_off(TOKEN_BYTES));
        let last_info =
            ElectAmountCalculator::last_stake_info(&stake_data.lock().delta(), self.current_epoch);

        let actual_info = ElectAmountCalculator::new(
            wallet_amount,
            total_stake_amount,
            last_info,
            ElectItem::Stake(&self.stake),
        )
        .calc_actual_amount()?;

        let inner_stake_data = stake_data.lock();

        Ok(vec![
            // stake AT cell data
            token_cell_data(
                actual_info.total_elect_amount,
                stake_data
                    .as_builder()
                    .lock(
                        inner_stake_data
                            .as_builder()
                            .delta(
                                StakeItem {
                                    is_increase:        actual_info.is_increase,
                                    amount:             actual_info.amount,
                                    inauguration_epoch: self.stake.inauguration_epoch,
                                }
                                .into(),
                            )
                            .build(),
                    )
                    .build()
                    .as_bytes(),
            ),
            // AT cell data
            actual_info.wallet_amount.pack().as_bytes(),
        ])
    }
}
