use std::{path::PathBuf, vec};

use common::{
    traits::smt::{DelegateSmtStorage, ProposalSmtStorage, RewardSmtStorage, StakeSmtStorage},
    types::smt::UserAmount,
};

use super::smt::SmtManager;

static ROCKSDB_PATH: &str = "./free-space/smt";

#[tokio::test]
async fn test_stake_functions() {
    let mut path = PathBuf::from(ROCKSDB_PATH);
    path.push("stake");
    let smt_manager = SmtManager::new(path);
    let staker = [5u8; 20].into();
    let epoch = 1;
    let amount = 100u128;

    let amounts = vec![UserAmount {
        user: staker,
        amount,
        is_increase: true,
    }];
    // insert
    StakeSmtStorage::insert(&smt_manager, epoch, amounts)
        .await
        .unwrap();
    let result = StakeSmtStorage::get_amount(&smt_manager, epoch, staker)
        .await
        .unwrap()
        .unwrap();

    // remove
    StakeSmtStorage::remove(&smt_manager, epoch, staker)
        .await
        .unwrap();
    assert_eq!(result, amount);

    // get_sub_leaves

    // get sub roots

    // get sub root

    // get top root
}

#[tokio::test]
async fn test_delegate_functions() {
    let mut path = PathBuf::from(ROCKSDB_PATH);
    path.push("delegate");

    let smt_manager = SmtManager::new(path);
    let staker = [5u8; 20].into();
    let delegator = [6u8; 20].into();
    let epoch = 1;
    let amount = 100u128;

    let delegators = vec![(staker, UserAmount {
        user: delegator,
        amount,
        is_increase: true,
    })];

    // insert
    DelegateSmtStorage::insert(&smt_manager, epoch, delegators)
        .await
        .unwrap();
    let result = DelegateSmtStorage::get_amount(&smt_manager, delegator, staker, epoch)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result, amount);

    // remove
    let delegators = vec![(staker, delegator)];
    DelegateSmtStorage::remove(&smt_manager, epoch, delegators)
        .await
        .unwrap();

    let result = DelegateSmtStorage::get_amount(&smt_manager, delegator, staker, epoch)
        .await
        .unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_reward_functions() {
    let mut path = PathBuf::from(ROCKSDB_PATH);
    path.push("reward");
    let smt_manager = SmtManager::new(path);
    let address = [5u8; 20].into();
    let epoch = 1;

    // insert
    RewardSmtStorage::insert(&smt_manager, address, epoch)
        .await
        .unwrap();
    let result = RewardSmtStorage::get_epoch(&smt_manager, address)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result, epoch);
}

#[tokio::test]
async fn test_proposal_functions() {
    let mut path = PathBuf::from(ROCKSDB_PATH);
    path.push("proposal");
    let smt_manager = SmtManager::new(path);
    let validator = [5u8; 20].into();
    let epoch = 1;
    let proposal_count = 10;

    let proposals = vec![(validator, proposal_count)];

    // insert
    ProposalSmtStorage::insert(&smt_manager, epoch, proposals)
        .await
        .unwrap();
    let result = ProposalSmtStorage::get_count(&smt_manager, epoch, validator)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result, proposal_count);
}
