use anchor_lang::{prelude::*};
use anchor_spl::{associated_token::AssociatedToken, token::{transfer, Mint, Token, TokenAccount, Transfer, mint_to, MintTo}};

use crate::{emits::DepositEvent, errors::*, states::config::{Config}};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub liquidator: Signer<'info>,
    #[account(
        mut,
        constraint = lp_mint.key() == config.lp_mint
    )]
    pub lp_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = usdt_mint.key() == config.usdt_mint
    )]
    pub usdt_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [b"config".as_ref()],
        bump = config.bumps,
        has_one = lp_mint @ ProtocolError::InvalidMint,
    )]
    pub config: Account<'info, Config>,
    #[account(
        init_if_needed,
        payer = liquidator,
        associated_token::mint = lp_mint,
        associated_token::authority = liquidator,
    )]
    pub lp_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = usdt_mint,
        associated_token::authority = liquidator,
    )]
    pub liquidator_usdt_ata: Account<'info, TokenAccount>,
     #[account(
        mut,
        associated_token::mint = usdt_mint,
        associated_token::authority = config,
    )]
    pub usdt_protocol_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        require!(amount > 0, ProtocolError::InvalidAmount);
        require!(self.liquidator_usdt_ata.amount >= amount, ProtocolError::NotEnoughFunds);

        let total_supply = self.lp_mint.supply;
        let revenue = self.config.net_deposits;
        let shares_to_mint = if total_supply == 0 {
            amount
        } else {
            (amount as u128)
                .checked_mul(total_supply as u128)
                .unwrap()
                .checked_div(revenue as u128)
                .unwrap().try_into().unwrap()
        };

        transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.liquidator_usdt_ata.to_account_info(),
                    to: self.usdt_protocol_ata.to_account_info(),
                    authority: self.liquidator.to_account_info(),
                },
            ),
            amount,
        )?;
        let seeds = &[b"protocol".as_ref(), &[self.config.bumps]];
        let signer_seeds = &[&seeds[..]];
        mint_to(CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.lp_mint.to_account_info(),
                to: self.lp_ata.to_account_info(),
                authority: self.config.to_account_info(),
            },
            signer_seeds,
        ), shares_to_mint)?;
        self.config.net_deposits = self.config.net_deposits.checked_add(amount).ok_or(ProtocolError::Overflow)?;

        emit!(DepositEvent { 
            user: self.liquidator.key(), 
            amount, 
            shares_minted: shares_to_mint, }); 
        Ok(())
    }
}

