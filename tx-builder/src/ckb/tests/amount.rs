use common::types::tx_builder::DelegateItem;

use crate::ckb::define::error::CkbTxResult;
use crate::ckb::helper::amount_calculator::*;

#[cfg(test)]
mod add_success {
    use super::*;

    #[test]
    fn none_last_info() {
        let wallet_amount = 1;
        let total_amount = 0;
        let last_info = DelegateItem::default();
        let new_info = DelegateItem {
            is_increase: true,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info =
            calc_actual_info(wallet_amount, total_amount, last_info, new_info).unwrap();

        assert_eq!(actual_info.wallet_amount, 0);
        assert_eq!(actual_info.total_amount, 1);
        assert_eq!(actual_info.amount, 1);
        assert!(actual_info.is_increase);
    }

    #[test]
    fn last_add_info() {
        let wallet_amount = 1;
        let total_amount = 0;
        let last_info = DelegateItem {
            is_increase: true,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };
        let new_info = DelegateItem {
            is_increase: true,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info =
            calc_actual_info(wallet_amount, total_amount, last_info, new_info).unwrap();

        assert_eq!(actual_info.wallet_amount, 0);
        assert_eq!(actual_info.total_amount, 1);
        assert_eq!(actual_info.amount, 2);
        assert!(actual_info.is_increase);
    }

    #[test]
    fn last_redeem_amount_smaller() {
        let wallet_amount = 2;
        let total_amount = 0;
        let last_info = DelegateItem {
            is_increase: false,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };
        let new_info = DelegateItem {
            is_increase: true,
            amount: 3,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info =
            calc_actual_info(wallet_amount, total_amount, last_info, new_info).unwrap();

        assert_eq!(actual_info.wallet_amount, 0);
        assert_eq!(actual_info.total_amount, 2);
        assert_eq!(actual_info.amount, 2);
        assert!(actual_info.is_increase);
    }

    #[test]
    fn last_redeem_amount_bigger() {
        let wallet_amount = 1;
        let total_amount = 0;
        let last_info = DelegateItem {
            is_increase: false,
            amount: 3,
            inauguration_epoch: 3,
            ..Default::default()
        };
        let new_info = DelegateItem {
            is_increase: true,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info =
            calc_actual_info(wallet_amount, total_amount, last_info, new_info).unwrap();

        assert_eq!(actual_info.wallet_amount, 1);
        assert_eq!(actual_info.total_amount, 0);
        assert_eq!(actual_info.amount, 2);
        assert!(!actual_info.is_increase);
    }

    #[test]
    fn last_expired_add_amount_smaller() {
        let wallet_amount = 2;
        let total_amount = 0;
        let last_info = DelegateItem {
            is_increase: true,
            amount: 1,
            inauguration_epoch: 2,
            ..Default::default()
        };
        let new_info = DelegateItem {
            is_increase: true,
            amount: 3,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info =
            calc_actual_info(wallet_amount, total_amount, last_info, new_info).unwrap();

        assert_eq!(actual_info.wallet_amount, 0);
        assert_eq!(actual_info.total_amount, 2);
        assert_eq!(actual_info.amount, 2);
        assert!(actual_info.is_increase);
    }

    #[test]
    fn last_expired_add_amount_bigger() {
        let wallet_amount = 0;
        let total_amount = 2;
        let last_info = DelegateItem {
            is_increase: true,
            amount: 3,
            inauguration_epoch: 2,
            ..Default::default()
        };
        let new_info = DelegateItem {
            is_increase: true,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info =
            calc_actual_info(wallet_amount, total_amount, last_info, new_info).unwrap();

        assert_eq!(actual_info.wallet_amount, 2);
        assert_eq!(actual_info.total_amount, 0);
        assert_eq!(actual_info.amount, 0);
    }

    #[test]
    fn last_expired_redeem_info() {
        let wallet_amount = 1;
        let total_amount = 0;
        let last_info = DelegateItem {
            is_increase: false,
            amount: 1,
            inauguration_epoch: 2,
            ..Default::default()
        };
        let new_info = DelegateItem {
            is_increase: true,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info =
            calc_actual_info(wallet_amount, total_amount, last_info, new_info).unwrap();

        assert_eq!(actual_info.wallet_amount, 0);
        assert_eq!(actual_info.total_amount, 1);
        assert_eq!(actual_info.amount, 1);
        assert!(actual_info.is_increase);
    }
}

#[cfg(test)]
mod redeem_success {
    use super::*;

    #[test]
    fn none_last_info() {
        let wallet_amount = 0;
        let total_amount = 1;
        let last_info = DelegateItem::default();
        let new_info = DelegateItem {
            is_increase: false,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info =
            calc_actual_info(wallet_amount, total_amount, last_info, new_info).unwrap();

        assert_eq!(actual_info.wallet_amount, 0);
        assert_eq!(actual_info.total_amount, 1);
        assert_eq!(actual_info.amount, 1);
        assert!(!actual_info.is_increase);
    }

    #[test]
    fn last_redeem_info() {
        let wallet_amount = 0;
        let total_amount = 2;
        let last_info = DelegateItem {
            is_increase: false,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };
        let new_info = DelegateItem {
            is_increase: false,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info =
            calc_actual_info(wallet_amount, total_amount, last_info, new_info).unwrap();

        assert_eq!(actual_info.wallet_amount, 0);
        assert_eq!(actual_info.total_amount, 2);
        assert_eq!(actual_info.amount, 2);
        assert!(!actual_info.is_increase);
    }

    #[test]
    fn last_add_amount_smaller() {
        let wallet_amount = 0;
        let total_amount = 2;
        let last_info = DelegateItem {
            is_increase: true,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };
        let new_info = DelegateItem {
            is_increase: false,
            amount: 3,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info =
            calc_actual_info(wallet_amount, total_amount, last_info, new_info).unwrap();

        assert_eq!(actual_info.wallet_amount, 1);
        assert_eq!(actual_info.total_amount, 1);
        assert_eq!(actual_info.amount, 2);
        assert!(!actual_info.is_increase);
    }

    #[test]
    fn last_add_amount_bigger() {
        let wallet_amount = 0;
        let total_amount = 1;
        let last_info = DelegateItem {
            is_increase: true,
            amount: 3,
            inauguration_epoch: 3,
            ..Default::default()
        };
        let new_info = DelegateItem {
            is_increase: false,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info =
            calc_actual_info(wallet_amount, total_amount, last_info, new_info).unwrap();

        assert_eq!(actual_info.wallet_amount, 1);
        assert_eq!(actual_info.total_amount, 0);
        assert_eq!(actual_info.amount, 2);
        assert!(actual_info.is_increase);
    }

    #[test]
    fn last_expired_add_amount_smaller() {
        let wallet_amount = 0;
        let total_amount = 1;
        let last_info = DelegateItem {
            is_increase: true,
            amount: 1,
            inauguration_epoch: 2,
            ..Default::default()
        };
        let new_info = DelegateItem {
            is_increase: false,
            amount: 3,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info =
            calc_actual_info(wallet_amount, total_amount, last_info, new_info).unwrap();

        assert_eq!(actual_info.wallet_amount, 1);
        assert_eq!(actual_info.total_amount, 0);
        assert_eq!(actual_info.amount, 2);
        assert!(!actual_info.is_increase);
    }

    #[test]
    fn last_expired_add_amount_bigger() {
        let wallet_amount = 0;
        let total_amount = 3;
        let last_info = DelegateItem {
            is_increase: true,
            amount: 3,
            inauguration_epoch: 2,
            ..Default::default()
        };
        let new_info = DelegateItem {
            is_increase: false,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info =
            calc_actual_info(wallet_amount, total_amount, last_info, new_info).unwrap();

        assert_eq!(actual_info.wallet_amount, 3);
        assert_eq!(actual_info.total_amount, 0);
        assert_eq!(actual_info.amount, 0);
    }

    #[test]
    fn last_expired_redeem_info() {
        let wallet_amount = 0;
        let total_amount = 1;
        let last_info = DelegateItem {
            is_increase: false,
            amount: 1,
            inauguration_epoch: 2,
            ..Default::default()
        };
        let new_info = DelegateItem {
            is_increase: false,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info =
            calc_actual_info(wallet_amount, total_amount, last_info, new_info).unwrap();

        assert_eq!(actual_info.wallet_amount, 0);
        assert_eq!(actual_info.total_amount, 1);
        assert_eq!(actual_info.amount, 1);
        assert!(!actual_info.is_increase);
    }
}

#[cfg(test)]
mod add_failed {
    use super::*;

    #[test]
    fn none_last_info() {
        let wallet_amount = 0;
        let total_amount = 0;
        let last_info = DelegateItem::default();
        let new_info = DelegateItem {
            is_increase: true,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info = calc_actual_info(wallet_amount, total_amount, last_info, new_info);
        assert!(actual_info.is_err());
    }

    #[test]
    fn last_add_info() {
        let wallet_amount = 0;
        let total_amount = 0;
        let last_info = DelegateItem {
            is_increase: true,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };
        let new_info = DelegateItem {
            is_increase: true,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info = calc_actual_info(wallet_amount, total_amount, last_info, new_info);
        assert!(actual_info.is_err());
    }

    #[test]
    fn last_redeem_amount_smaller() {
        let wallet_amount = 1;
        let total_amount = 1;
        let last_info = DelegateItem {
            is_increase: false,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };
        let new_info = DelegateItem {
            is_increase: true,
            amount: 3,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info = calc_actual_info(wallet_amount, total_amount, last_info, new_info);
        assert!(actual_info.is_err());
    }

    #[test]
    fn last_expired_add_amount_smaller() {
        let wallet_amount = 1;
        let total_amount = 0;
        let last_info = DelegateItem {
            is_increase: true,
            amount: 1,
            inauguration_epoch: 2,
            ..Default::default()
        };
        let new_info = DelegateItem {
            is_increase: true,
            amount: 3,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info = calc_actual_info(wallet_amount, total_amount, last_info, new_info);
        assert!(actual_info.is_err());
    }

    #[test]
    fn last_expired_add_amount_bigger() {
        let wallet_amount = 0;
        let total_amount = 0;
        let last_info = DelegateItem {
            is_increase: true,
            amount: 3,
            inauguration_epoch: 2,
            ..Default::default()
        };
        let new_info = DelegateItem {
            is_increase: true,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info = calc_actual_info(wallet_amount, total_amount, last_info, new_info);
        assert!(actual_info.is_err());
    }

    #[test]
    fn last_expired_redeem_info() {
        let wallet_amount = 0;
        let total_amount = 0;
        let last_info = DelegateItem {
            is_increase: false,
            amount: 1,
            inauguration_epoch: 2,
            ..Default::default()
        };
        let new_info = DelegateItem {
            is_increase: true,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info = calc_actual_info(wallet_amount, total_amount, last_info, new_info);
        assert!(actual_info.is_err());
    }
}

#[cfg(test)]
mod redeem_failed {
    use super::*;

    #[test]
    fn last_add_amount_smaller() {
        let wallet_amount = 0;
        let total_amount = 0;
        let last_info = DelegateItem {
            is_increase: true,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };
        let new_info = DelegateItem {
            is_increase: false,
            amount: 3,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info = calc_actual_info(wallet_amount, total_amount, last_info, new_info);
        assert!(actual_info.is_err());
    }

    #[test]
    fn last_add_amount_bigger() {
        let wallet_amount = 0;
        let total_amount = 0;
        let last_info = DelegateItem {
            is_increase: true,
            amount: 3,
            inauguration_epoch: 3,
            ..Default::default()
        };
        let new_info = DelegateItem {
            is_increase: false,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info = calc_actual_info(wallet_amount, total_amount, last_info, new_info);
        assert!(actual_info.is_err());
    }

    #[test]
    fn last_expired_add_amount_smaller() {
        let wallet_amount = 0;
        let total_amount = 0;
        let last_info = DelegateItem {
            is_increase: true,
            amount: 1,
            inauguration_epoch: 2,
            ..Default::default()
        };
        let new_info = DelegateItem {
            is_increase: false,
            amount: 3,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info = calc_actual_info(wallet_amount, total_amount, last_info, new_info);
        assert!(actual_info.is_err());
    }

    #[test]
    fn last_expired_add_amount_bigger() {
        let wallet_amount = 0;
        let total_amount = 2;
        let last_info = DelegateItem {
            is_increase: true,
            amount: 3,
            inauguration_epoch: 2,
            ..Default::default()
        };
        let new_info = DelegateItem {
            is_increase: false,
            amount: 1,
            inauguration_epoch: 3,
            ..Default::default()
        };

        let actual_info = calc_actual_info(wallet_amount, total_amount, last_info, new_info);
        assert!(actual_info.is_err());
    }
}

fn calc_actual_info(
    wallet_amount: u128,
    total_amount: u128,
    last_info: DelegateItem,
    new_info: DelegateItem,
) -> CkbTxResult<ActualAmount> {
    ElectAmountCalculator::new(
        wallet_amount,
        total_amount,
        ElectAmountCalculator::last_delegate_info(&last_info, 1),
        ElectItem::Delegate(&new_info),
    )
    .calc_actual_amount()
}
