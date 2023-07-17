use common::traits::tx_builder::IInitTxBuilder;
use common::types::tx_builder::{Checkpoint, Metadata, PrivateKey};
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use tx_builder::ckb::helper::{OmniEth, Tx};
use tx_builder::ckb::init::InitTxBuilder;

use crate::config::types::{PrivKeys, TypeIds as CTypeIds};
use crate::config::{parse_file, write_file};
use crate::mock::mock_axon_validators;

use crate::{PRIV_KEYS_PATH, TYPE_IDS_PATH};

pub async fn init_tx(ckb: &CkbRpcClient) {
    let priv_keys: PrivKeys = parse_file(PRIV_KEYS_PATH);
    let test_seeder_key = priv_keys.seeder_privkey.into_h256().unwrap();
    let omni_eth = OmniEth::new(test_seeder_key.clone());
    println!("seeder ckb addres: {}\n", omni_eth.ckb_address().unwrap());

    let mut stakers = vec![];
    for (i, staker_privkey) in priv_keys.staker_privkeys.into_iter().enumerate() {
        let privkey = staker_privkey.clone().into_h256().unwrap();
        let omni_eth = OmniEth::new(privkey);
        println!(
            "staker{} ckb addres: {}",
            i,
            omni_eth.ckb_address().unwrap()
        );
        stakers.push(omni_eth.address().unwrap());
    }

    _init_tx(
        ckb,
        test_seeder_key,
        Checkpoint {
            epoch: 0,
            period: 0,
            latest_block_height: 10,
            timestamp: 11111,
            ..Default::default()
        },
        Metadata {
            epoch_len: 100,
            period_len: 100,
            quorum: 10,
            validators: mock_axon_validators(),
            ..Default::default()
        },
        stakers,
    )
    .await;
}

pub async fn _init_tx(
    ckb: &CkbRpcClient,
    seeder_key: PrivateKey,
    checkpoint: Checkpoint,
    metadata: Metadata,
    stakers: Vec<ckb_types::H160>,
) -> Tx<CkbRpcClient> {
    let (tx, type_id_args) =
        InitTxBuilder::new(ckb, seeder_key, 10000, checkpoint, metadata, stakers)
            .build_tx()
            .await
            .unwrap();

    let mut tx = Tx::new(ckb, tx);

    match tx.send().await {
        Ok(tx_hash) => println!("tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    let type_ids: CTypeIds = type_id_args.into();
    write_file(TYPE_IDS_PATH, &type_ids);

    tx
}
