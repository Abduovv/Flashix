#[allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Token, TokenAccount, Mint, Transfer, transfer},
};
use anchor_lang::{
    solana_program::sysvar::instructions::{ID as INSTRUCTIONS_SYSVAR_ID, load_instruction_at_checked},
};

use crate::{
    errors::*,
    states::config::Config,
    emits::RepayEvent,
};

#[derive(Accounts)]
pub struct Repay<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,

    #[account(
        seeds = [b"config".as_ref()],
        bump = config.bumps,
        has_one = usdt_mint @ ProtocolError::InvalidMint
    )]
    pub config: Account<'info, Config>,

    pub usdt_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = borrower,
        associated_token::mint = usdt_mint,
        associated_token::authority = borrower
    )]
    pub borrower_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = usdt_mint,
        associated_token::authority = config
    )]
    pub protocol_ata: Account<'info, TokenAccount>,

    #[account(
        address = INSTRUCTIONS_SYSVAR_ID
    )]
    /// CHECK: Instructions sysvar account
    pub instructions: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Repay<'info> {
    pub fn repay_process(&mut self) -> Result<()> {
        let ixs = self.instructions.to_account_info();

        // 1. Parse `borrow` instruction at index 0 to get original borrowed amount
        let borrow_ix = load_instruction_at_checked(0, &ixs)
            .map_err(|_| ProtocolError::MissingBorrowIx)?;

        let mut amount_borrowed_data: [u8; 8] = [0u8; 8];
        amount_borrowed_data.copy_from_slice(&borrow_ix.data[8..16]);
        let amount_borrowed = u64::from_le_bytes(amount_borrowed_data);

        // 2. Calculate protocol fee (e.g., 500 basis points = 5%)
        let fee = (amount_borrowed as u128)
            .checked_mul(self.config.basis_points as u128)
            .unwrap()
            .checked_div(10_000)
            .ok_or(ProtocolError::Overflow)? as u64;

        let total_repayment = amount_borrowed
            .checked_add(fee)
            .ok_or(ProtocolError::Overflow)?;

        // 3. Transfer amount_borrowed to protocol vault
        transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.borrower_ata.to_account_info(),
                    to: self.protocol_ata.to_account_info(),
                    authority: self.borrower.to_account_info(),
                },
            ),
            total_repayment,
        )?;

        // 4. Update state tracking
        self.config.net_deposits = self
            .config
            .net_deposits
            .checked_add(amount_borrowed)
            .ok_or(ProtocolError::Overflow)?;

        self.config.collected_fees = self
            .config
            .collected_fees
            .checked_add(fee)
            .ok_or(ProtocolError::Overflow)?;

        // 5. Emit event
        emit!(RepayEvent {
            user: self.borrower.key(),
            amount: total_repayment,
        });

        Ok(())
    }
}
