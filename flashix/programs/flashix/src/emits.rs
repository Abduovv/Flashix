use anchor_lang::prelude::*;

#[event]
pub struct DepositEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub shares_minted: u64,
}

#[event]
pub struct BorrowEvent {
    pub user: Pubkey,
    pub amount: u64,
}

#[event]
pub struct RepayEvent {
    pub user: Pubkey,
    pub amount: u64,
}

#[event]
pub struct WithdrawEvent {
    pub user: Pubkey,
    pub lp_amount: u64,
    pub usdt_amount: u64
}

