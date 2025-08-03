use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Token, TokenAccount, Mint},
};

use crate::{states::config::Config,ID};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        mut,
        address = ID
    )]
    pub protocol: Signer<'info>,
    #[account(
        init,
        payer = protocol,
        space = 8 + 32 + 2 + 32,
        seeds = [b"config".as_ref()],
        bump,
    )]
    pub config: Account<'info, Config>,
    #[account(
        init,
        payer = protocol,
        mint::decimals = 6,
        mint::authority = config,
    )]
    pub lp_mint: Account<'info, Mint>,
    pub usdt_mint: Account<'info, Mint>,
    pub protocol_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl <'info> Initialize<'info> {
    pub fn initialize(&mut self, basis_points:  u16, bumps: InitializeBumps) -> Result<()> {
        self.config.net_deposits = 0;
        self.config.lp_mint = self.lp_mint.key();
        self.config.basis_points = basis_points;
        self.config.collected_fees = 0;
        self.config.usdt_mint = self.usdt_mint.key();
        self.config.bumps = bumps.config;
        Ok(())
    }
    
}