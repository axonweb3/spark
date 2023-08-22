use std::collections::HashSet;

use ckb_types::H256;
use common::types::tx_builder::{
    Checkpoint, Metadata, MetadataInfo, PrivateKey, ProposeCount, RewardMeta,
};
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use tx_builder::ckb::helper::{OmniEth, Tx};
use tx_builder::ckb::init::InitTxBuilder;

use crate::config::types::TypeIds as CTypeIds;
use crate::config::write_file;
use crate::mock::mock_axon_validators_v2;
use crate::{MAX_TRY, TYPE_IDS_PATH};

pub async fn run_init_tx(
    ckb: &CkbRpcClient,
    seeder_key: H256,
    stakers_key: Vec<H256>,
    quorum: u16,
) {
    let omni_eth = OmniEth::new(seeder_key.clone());
    println!("seeder ckb addres: {}\n", omni_eth.ckb_address().unwrap());

    let mut stakers = HashSet::new();
    let mut staker_privkeys = vec![];
    let mut propose_count = vec![];

    for (i, staker_privkey) in stakers_key.into_iter().enumerate() {
        staker_privkeys.push(staker_privkey.clone());

        let omni_eth = OmniEth::new(staker_privkey);
        println!(
            "staker{} ckb addres: {}",
            i,
            omni_eth.ckb_address().unwrap()
        );
        stakers.insert(omni_eth.address().unwrap());

        propose_count.push(ProposeCount {
            proposer: omni_eth.address().unwrap(),
            count:    100,
        });
    }

    init_tx(
        ckb,
        seeder_key,
        Checkpoint {
            epoch: 0,
            period: 0,
            latest_block_height: 10,
            timestamp: 11111,
            propose_count,
            ..Default::default()
        },
        MetadataInfo {
            reward_meta:     RewardMeta {
                base_reward:           10000,
                half_reward_cycle:     200,
                propose_minimum_rate:  95,
                propose_discount_rate: 95,
            },
            epoch0_metadata: Metadata {
                epoch_len: 1,
                period_len: 100,
                quorum: 10,
                validators: mock_axon_validators_v2(&staker_privkeys),
                ..Default::default()
            },
            epoch1_metadata: Metadata {
                epoch_len: 1,
                period_len: 100,
                quorum,
                validators: mock_axon_validators_v2(&staker_privkeys),
                ..Default::default()
            },
        },
        stakers,
    )
    .await;
}

pub async fn init_tx(
    ckb: &CkbRpcClient,
    seeder_key: PrivateKey,
    checkpoint: Checkpoint,
    metadata: MetadataInfo,
    stakers: HashSet<ckb_types::H160>,
) -> Tx<CkbRpcClient> {
    let (tx, type_id_args) =
        InitTxBuilder::new(ckb, seeder_key, 1000000, checkpoint, metadata, stakers)
            .build_tx()
            .await
            .unwrap();

    let mut tx = Tx::new(ckb, tx);

    match tx.send().await {
        Ok(tx_hash) => println!("init tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    println!("init tx ready");
    tx.wait_until_committed(1000, MAX_TRY).await.unwrap();
    println!("init tx committed");

    let type_ids: CTypeIds = type_id_args.into();
    write_file(TYPE_IDS_PATH, &type_ids);

    tx
}
