use anchor_lang::prelude::*;

#[account]
pub struct Config {
    pub net_deposits: u64,
    pub basis_points: u16,
    pub collected_fees: u64,
    pub lp_mint: Pubkey,
    pub usdt_mint: Pubkey,
    pub bumps: u8
}