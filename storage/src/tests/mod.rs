use std::{fs, path::PathBuf, vec};

use ckb_types::h160;

use common::{
    traits::smt::{DelegateSmtStorage, ProposalSmtStorage, RewardSmtStorage, StakeSmtStorage},
    types::smt::{Staker, UserAmount},
};

use super::smt::SmtManager;

static ROCKSDB_PATH: &str = "./free-space/smt";

#[tokio::test]
async fn test_stake_functions() {
    let mut path = PathBuf::from(ROCKSDB_PATH);
    path.push("stake");
    if path.exists() {
        fs::remove_dir_all(path.clone()).unwrap();
    }

    let smt_manager = SmtManager::new(path);
    let staker = [5u8; 20].into();
    let epoch = 1;
    let amount = 100u128;

    let amounts = vec![UserAmount {
        user: staker,
        amount,
        is_increase: true,
    }];

    // insert & get
    StakeSmtStorage::insert(&smt_manager, epoch, amounts.clone())
        .await
        .unwrap();
    let result = StakeSmtStorage::get_amount(&smt_manager, epoch, staker)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(result, amount);

    // update
    StakeSmtStorage::insert(&smt_manager, epoch, amounts)
        .await
        .unwrap();
    let result = StakeSmtStorage::get_amount(&smt_manager, epoch, staker)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result, amount);

    // new epoch
    StakeSmtStorage::new_epoch(&smt_manager, epoch + 1)
        .await
        .unwrap();
    let result = StakeSmtStorage::get_amount(&smt_manager, epoch + 1, staker)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result, amount);

    // remove
    StakeSmtStorage::remove(&smt_manager, epoch, vec![staker])
        .await
        .unwrap();

    let result = StakeSmtStorage::get_amount(&smt_manager, epoch, staker)
        .await
        .unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_delegate_smt() {
    let mut path = PathBuf::from(ROCKSDB_PATH);
    path.push("delegate1");
    if path.exists() {
        fs::remove_dir_all(path.clone()).unwrap();
    }

    let smt_manager = SmtManager::new(path);
    let stakers: Vec<Staker> = vec![
        h160!("0x11999a8db6049687978cf396ee4599876a12d960").0.into(),
        h160!("0xca00b6a1b34b7fcf087e9f81dfd63724c337b6fe").0.into(),
        h160!("0x5d164cf753be3272a22cbaf60c94f3e8e3b20878").0.into(),
    ];
    let delegator = h160!("0x25cdf0e188c0ed538709ad1b696b52595202b02b").0.into();
    let epoch = 2;
    let amount = 10u128;

    let delegators = vec![UserAmount {
        user: delegator,
        amount,
        is_increase: true,
    }];

    for staker in stakers {
        DelegateSmtStorage::insert(&smt_manager, epoch, staker, delegators.clone())
            .await
            .unwrap();
        let result = DelegateSmtStorage::get_amount(&smt_manager, epoch, staker, delegator)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result, amount);

        let smt = DelegateSmtStorage::get_sub_leaves(&smt_manager, epoch, staker)
            .await
            .unwrap();
        for (user, amount) in smt.into_iter() {
            assert_eq!(amount, 10);
            assert_eq!(user, delegator);
        }
    }
}

#[tokio::test]
async fn test_delegate_smt_no_extra_data() {
    let mut path = PathBuf::from(ROCKSDB_PATH);
    path.push("delegate2");
    if path.exists() {
        fs::remove_dir_all(path.clone()).unwrap();
    }

    let smt_manager = SmtManager::new(path);
    let stakers: Vec<Staker> = vec![
        h160!("0x11999a8db6049687978cf396ee4599876a12d960").0.into(),
        h160!("0xca00b6a1b34b7fcf087e9f81dfd63724c337b6fe").0.into(),
        h160!("0x5d164cf753be3272a22cbaf60c94f3e8e3b20878").0.into(),
    ];
    let delegator = h160!("0x25cdf0e188c0ed538709ad1b696b52595202b02b").0.into();
    let epoch = 2;
    let amount = 10u128;

    let delegators = vec![UserAmount {
        user: delegator,
        amount,
        is_increase: true,
    }];

    for staker in stakers.clone() {
        let smt = DelegateSmtStorage::get_sub_leaves(&smt_manager, epoch, staker)
            .await
            .unwrap();
        assert_eq!(0, smt.len());

        DelegateSmtStorage::insert(&smt_manager, epoch, staker, delegators.clone())
            .await
            .unwrap();
        let result = DelegateSmtStorage::get_amount(&smt_manager, epoch, staker, delegator)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result, amount);

        let smt = DelegateSmtStorage::get_sub_leaves(&smt_manager, epoch, staker)
            .await
            .unwrap();
        for (user, amount) in smt.into_iter() {
            assert_eq!(amount, 10);
            assert_eq!(user, delegator);
        }
    }
}

#[tokio::test]
async fn test_delegate_functions() {
    let mut path = PathBuf::from(ROCKSDB_PATH);
    path.push("delegate");
    if path.exists() {
        fs::remove_dir_all(path.clone()).unwrap();
    }

    let smt_manager = SmtManager::new(path);
    let staker = [5u8; 20].into();
    let delegator = [6u8; 20].into();
    let epoch = 1;
    let amount = 100u128;

    let delegators = vec![UserAmount {
        user: delegator,
        amount,
        is_increase: true,
    }];

    // insert & get
    DelegateSmtStorage::insert(&smt_manager, epoch, staker, delegators.clone())
        .await
        .unwrap();
    let result = DelegateSmtStorage::get_amount(&smt_manager, epoch, staker, delegator)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result, amount);

    // update
    DelegateSmtStorage::insert(&smt_manager, epoch, staker, delegators)
        .await
        .unwrap();
    let result = DelegateSmtStorage::get_amount(&smt_manager, epoch, staker, delegator)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result, amount);

    // new epoch
    DelegateSmtStorage::new_epoch(&smt_manager, epoch + 1)
        .await
        .unwrap();
    let result = DelegateSmtStorage::get_amount(&smt_manager, epoch, staker, delegator)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result, amount);

    // remove
    let delegators = vec![(staker, delegator)];
    DelegateSmtStorage::remove(&smt_manager, epoch, delegators)
        .await
        .unwrap();

    let result = DelegateSmtStorage::get_amount(&smt_manager, epoch, staker, delegator)
        .await
        .unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_reward_functions() {
    let mut path = PathBuf::from(ROCKSDB_PATH);
    path.push("reward");
    if path.exists() {
        fs::remove_dir_all(path.clone()).unwrap();
    }

    let smt_manager = SmtManager::new(path);
    let address = [5u8; 20].into();
    let epoch = 1;

    // insert
    RewardSmtStorage::insert(&smt_manager, epoch, address)
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
    if path.exists() {
        fs::remove_dir_all(path.clone()).unwrap();
    }

    let smt_manager = SmtManager::new(path);
    let validator = [5u8; 20].into();
    let epoch = 1;
    let proposal_count = 10;

    let proposals = vec![(validator, proposal_count)];

    // insert & get
    ProposalSmtStorage::insert(&smt_manager, epoch, proposals)
        .await
        .unwrap();
    let result = ProposalSmtStorage::get_count(&smt_manager, epoch, validator)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result, proposal_count);
}
