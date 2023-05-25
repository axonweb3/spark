use axon_types::delegate::DelegateInfoDelta;
use axon_types::stake::StakeInfoDelta;
use ckb_types::bytes::Bytes;

use common::types::tx_builder::{Amount, DelegateItem, Epoch, StakeItem};
use common::utils::convert::*;

use crate::ckb::define::config::{INAUGURATION, TOKEN_BYTES};
use crate::ckb::define::error::*;

pub struct ElectAmountCaculator<'a> {
    wallet_amount:      Amount,
    total_elect_amount: Amount,
    last_info:          LastElectItem,
    new_info:           ElectItem<'a>,
}

pub struct LastElectItem {
    pub amount:      Amount,
    pub is_increase: bool,
    pub has_expired: bool,
}

pub enum ElectItem<'a> {
    Stake(&'a StakeItem),
    Delegate(&'a DelegateItem),
}

pub struct ActualAmount {
    pub wallet_amount:      Amount,
    pub total_elect_amount: Amount,
    pub amount:             Amount,
    pub is_increase:        bool,
}

impl<'a> ElectAmountCaculator<'a> {
    pub fn new(
        wallet_amount: Amount,
        total_elect_amount: Amount,
        last_info: LastElectItem,
        new_elect_item: ElectItem<'a>,
    ) -> Self {
        Self {
            wallet_amount,
            total_elect_amount,
            last_info,
            new_info: new_elect_item,
        }
    }

    pub fn calc_actual_amount(&self) -> CkbTxResult<ActualAmount> {
        let (is_increase, amount) = match self.new_info {
            ElectItem::Stake(stake) => (stake.is_increase, stake.amount),
            ElectItem::Delegate(delegate) => (delegate.is_increase, delegate.amount),
        };

        if is_increase {
            self.increase_amount(amount)
        } else {
            self.redeem_amount(amount)
        }
    }

    pub fn calc_wallet_amount(wallet_data: &[Bytes]) -> u128 {
        let mut wallet_amount = 0;
        for data in wallet_data.iter() {
            wallet_amount += new_u128(&data[..TOKEN_BYTES]);
        }
        wallet_amount
    }

    pub fn last_stake_info(stake: &StakeInfoDelta, current_epoch: Epoch) -> LastElectItem {
        LastElectItem {
            amount:      to_u128(&stake.amount()),
            is_increase: to_bool(&stake.is_increase()),
            has_expired: to_u64(&stake.inauguration_epoch()) < current_epoch + INAUGURATION,
        }
    }

    pub fn last_delegate_info(delegate: &DelegateInfoDelta, current_epoch: Epoch) -> LastElectItem {
        LastElectItem {
            amount:      to_u128(&delegate.amount()),
            is_increase: to_bool(&delegate.is_increase()),
            has_expired: to_u64(&delegate.inauguration_epoch()) < current_epoch + INAUGURATION,
        }
    }

    fn increase_amount(&self, new_amount: Amount) -> CkbTxResult<ActualAmount> {
        let mut wallet_amount = self.wallet_amount;
        let mut total_elect_amount = self.total_elect_amount;

        let mut actual_amount = new_amount;
        let mut actual_is_increase = true;

        if self.last_info.amount == 0 {
            if wallet_amount < new_amount {
                return Err(CkbTxErr::ExceedWalletAmount {
                    wallet_amount,
                    amount: new_amount,
                });
            }
            wallet_amount -= new_amount;
            total_elect_amount += new_amount;
        } else if self.last_info.has_expired {
            if self.last_info.is_increase {
                let diff_amount = if new_amount >= self.last_info.amount {
                    actual_amount = new_amount - self.last_info.amount;
                    actual_amount
                } else {
                    actual_amount = 0;
                    self.last_info.amount - new_amount
                };
                if wallet_amount < diff_amount {
                    return Err(CkbTxErr::ExceedWalletAmount {
                        wallet_amount: self.wallet_amount,
                        amount:        diff_amount,
                    });
                }
                wallet_amount -= diff_amount;
                total_elect_amount += diff_amount;
            }
        } else if self.last_info.is_increase {
            actual_amount = new_amount + self.last_info.amount;
            if wallet_amount < new_amount {
                return Err(CkbTxErr::ExceedWalletAmount {
                    wallet_amount: self.wallet_amount,
                    amount:        new_amount,
                });
            }
            wallet_amount -= new_amount;
            total_elect_amount += new_amount;
        } else if new_amount >= self.last_info.amount {
            actual_amount = new_amount - self.last_info.amount;
            if wallet_amount < actual_amount {
                return Err(CkbTxErr::ExceedWalletAmount {
                    wallet_amount: self.wallet_amount,
                    amount:        actual_amount,
                });
            }
            wallet_amount -= actual_amount;
            total_elect_amount += actual_amount;
        } else {
            actual_amount = self.last_info.amount - new_amount;
            actual_is_increase = false;
        }
        Ok(ActualAmount {
            wallet_amount,
            total_elect_amount,
            amount: actual_amount,
            is_increase: actual_is_increase,
        })
    }

    fn redeem_amount(&self, new_amount: Amount) -> CkbTxResult<ActualAmount> {
        let mut wallet_amount = self.wallet_amount;
        let mut total_elect_amount = self.total_elect_amount;

        let mut actual_amount = new_amount;
        let mut actual_is_increase = false;

        if self.last_info.amount == 0 {
            wallet_amount += new_amount;
            if total_elect_amount < new_amount {
                return Err(CkbTxErr::ExceedTotalAmount {
                    total_amount: total_elect_amount,
                    new_amount,
                });
            }
            total_elect_amount -= new_amount;
        } else if self.last_info.has_expired {
            if self.last_info.is_increase {
                if new_amount >= self.last_info.amount {
                    actual_amount = new_amount - self.last_info.amount;
                } else {
                    actual_amount = 0;
                }
                wallet_amount += self.last_info.amount;
                if total_elect_amount < new_amount {
                    return Err(CkbTxErr::ExceedTotalAmount {
                        total_amount: total_elect_amount,
                        new_amount,
                    });
                }
                total_elect_amount -= self.last_info.amount;
            }
        } else if self.last_info.is_increase {
            if new_amount >= self.last_info.amount {
                actual_amount = new_amount - self.last_info.amount;
                wallet_amount += self.last_info.amount;
                if total_elect_amount < new_amount {
                    return Err(CkbTxErr::ExceedTotalAmount {
                        total_amount: total_elect_amount,
                        new_amount,
                    });
                }
                total_elect_amount -= self.last_info.amount;
            } else {
                actual_amount = self.last_info.amount - new_amount;
                wallet_amount += new_amount;
                if total_elect_amount < new_amount {
                    return Err(CkbTxErr::ExceedTotalAmount {
                        total_amount: total_elect_amount,
                        new_amount,
                    });
                }
                total_elect_amount -= new_amount;
                actual_is_increase = true;
            }
        } else {
            actual_amount = new_amount + self.last_info.amount;
        }
        Ok(ActualAmount {
            wallet_amount,
            total_elect_amount,
            amount: actual_amount,
            is_increase: actual_is_increase,
        })
    }
}
