use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellOutput, Script},
    prelude::{Entity, Pack},
};

use common::traits::{
    smt::{DelegateSmtStorage, ProposalSmtStorage, StakeSmtStorage},
    tx_builder::IMetadataTxBuilder,
};
use common::types::tx_builder::*;
use ethereum_types::H160;

pub struct MetadataSmtTxBuilder<PSmt> {
    _kicker:         PrivateKey,
    _quorum:         u16,
    last_checkpoint: Checkpoint,
    smt:             PSmt,
}

#[async_trait]
impl<PSmt> IMetadataTxBuilder<PSmt> for MetadataSmtTxBuilder<PSmt>
where
    PSmt: ProposalSmtStorage + StakeSmtStorage + DelegateSmtStorage + Send + 'static + Sync,
{
    fn new(_kicker: PrivateKey, _quorum: u16, last_checkpoint: Checkpoint, smt: PSmt) -> Self {
        Self {
            _kicker,
            _quorum,
            last_checkpoint,
            smt,
        }
    }

    async fn build_tx(&self) -> Result<(TransactionView, NonTopStakers, NonTopDelegators)> {
        // todo: get metadata cell, stake smt cell, delegate smt cell
        let inputs = vec![];

        ProposalSmtStorage::insert(
            &self.smt,
            self.last_checkpoint.epoch,
            self.last_checkpoint
                .propose_count
                .iter()
                .map(|v| (v.proposer.0.into(), v.count as u64))
                .collect(),
        )
        .await?;

        struct StakeInfo {
            staker:     common::types::H160,
            amount:     u128,
            delegaters: HashMap<common::types::H160, u128>,
        }

        impl StakeInfo {
            fn total_stake(&self) -> u128 {
                self.amount + self.delegaters.values().sum::<u128>()
            }
        }

        impl PartialOrd for StakeInfo {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.total_stake().cmp(&other.total_stake()))
            }
        }

        impl Ord for StakeInfo {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.total_stake().cmp(&other.total_stake())
            }
        }

        impl PartialEq for StakeInfo {
            fn eq(&self, other: &Self) -> bool {
                self.staker == other.staker
                    && self.amount == other.amount
                    && self.delegaters == other.delegaters
            }
        }

        impl Eq for StakeInfo {}
        let (validators, no_top_stakers, no_top_delegators) = {
            let stakers =
                StakeSmtStorage::get_sub_leaves(&self.smt, self.last_checkpoint.epoch).await?;

            let mut mid = std::collections::BinaryHeap::new();

            for (staker, amount) in stakers {
                let delegaters = DelegateSmtStorage::get_sub_leaves(
                    &self.smt,
                    self.last_checkpoint.epoch,
                    staker.clone(),
                )
                .await?;
                mid.push(StakeInfo {
                    staker,
                    amount,
                    delegaters,
                })
            }

            let stake_infos = {
                let mut res = Vec::with_capacity(self._quorum as usize);
                for _ in 0..self._quorum {
                    res.push(mid.pop().unwrap())
                }
                res
            };

            if mid.is_empty() {
                (stake_infos, Vec::default(), HashMap::default())
            } else {
                let mut no_top_stakers = Vec::with_capacity(mid.len());
                let mut no_top_delegaters: HashMap<H160, HashMap<H160, u128>> = HashMap::default();

                for i in mid {
                    no_top_stakers.push((i.staker, i.amount));
                    for (d, u) in i.delegaters {
                        no_top_delegaters
                            .entry(d.0.into())
                            .or_default()
                            .insert(i.staker.0.into(), u);
                    }
                }

                (stake_infos, no_top_stakers, no_top_delegaters)
            }
        };

        StakeSmtStorage::remove(
            &self.smt,
            self.last_checkpoint.epoch,
            no_top_stakers.iter().map(|(a, _)| a).cloned().collect(),
        )
        .await
        .unwrap();

        let delegatets_remove_keys = no_top_delegators
            .iter()
            .map(|(d, i)| i.keys().cloned().zip(std::iter::repeat(d.clone())))
            .flatten()
            .collect();

        DelegateSmtStorage::remove(
            &self.smt,
            self.last_checkpoint.epoch,
            delegatets_remove_keys,
        )
        .await
        .unwrap();

        // todo: fetch validators' blspubkey pubkey to build metadata

        // todo
        let outputs_data = vec![Bytes::default()];

        // todo: fill lock, type
        let fake_lock = Script::default();
        let fake_type = Script::default();
        let outputs = vec![
            // metadata cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // stake smt cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // delegate smt cell
            CellOutput::new_builder()
                .lock(fake_lock)
                .type_(Some(fake_type).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
        ];

        // todo: add removed stakers' stake AT cells to inputs and outputs and
        //       add withdraw AT cells to outputs

        // todo: add removed delegators' delegate AT cells to inputs and outputs and
        //       add withdraw AT cells to outputs

        // todo
        let cell_deps = vec![];

        // todo: balance tx, fill placeholder witnesses,
        let tx = TransactionBuilder::default()
            .inputs(inputs)
            .outputs(outputs)
            .outputs_data(outputs_data.pack())
            .cell_deps(cell_deps)
            .build();

        // todo: sign tx

        Ok((tx, HashMap::default(), HashMap::default()))
    }
}
