use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};

use crate::{states::config::Config, errors::*, emits::WithdrawEvent};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub lp: Signer<'info>,

    #[account(mut,)]
    pub lp_mint: Account<'info, Mint>,
    pub usdt_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"config".as_ref()],
        bump = config.bumps,
        has_one = lp_mint @ ProtocolError::InvalidMint,
        has_one = usdt_mint @ ProtocolError::InvalidMint
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = lp,
    )]
    pub lp_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = usdt_mint,
        associated_token::authority = lp,
    )]
    pub lp_usdt_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = usdt_mint,
        associated_token::authority = config,
    )]
    pub usdt_protocol_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, lp_amount: u64) -> Result<()> {
        require!(lp_amount > 0, ProtocolError::InvalidAmount);
        require!(self.lp_ata.amount >= lp_amount, ProtocolError::NotEnoughFunds);

        let total_supply = self.lp_mint.supply;
        require!(total_supply > 0, ProtocolError::InvalidState);

        let withdraw_amount = (lp_amount as u128)
            .checked_mul(self.config.net_deposits as u128)
            .ok_or(ProtocolError::Overflow)?
            / total_supply as u128;

        let withdraw_amount = withdraw_amount as u64;

        anchor_spl::token::burn(
            CpiContext::new(
                self.token_program.to_account_info(),
                anchor_spl::token::Burn {
                    mint: self.lp_mint.to_account_info(),
                    from: self.lp_ata.to_account_info(),
                    authority: self.lp.to_account_info(),
                },
            ),
            lp_amount,
        )?;

        let seeds = &[b"protocol".as_ref(), &[self.config.bumps]];
        let signer_seeds = &[&seeds[..]];
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: self.usdt_protocol_ata.to_account_info(),
                    to: self.lp_usdt_ata.to_account_info(),
                    authority: self.config.to_account_info(),
                },
                signer_seeds,
            ),
            withdraw_amount,
        )?;

        self.config.net_deposits = self
            .config
            .net_deposits
            .checked_sub(withdraw_amount)
            .ok_or(ProtocolError::Overflow)?;

        emit!(WithdrawEvent {
            lp_amount,
            usdt_amount:withdraw_amount, 
            user: self.lp.key(), 
        });
        Ok(())
    }
}
