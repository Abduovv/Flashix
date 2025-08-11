use anchor_lang::prelude::*;
use anchor_lang::{
    solana_program::sysvar::instructions::{
        load_instruction_at_checked, ID as INSTRUCTIONS_SYSVAR_ID,
    },
    Discriminator,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

use crate::{emits::BorrowEvent, errors::*, instruction, states::config::Config, ID};
#[derive(Accounts)]
pub struct Borrow<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,

    #[account(
        seeds = [b"config".as_ref()],
        bump
    )]
    pub config: Account<'info, Config>,

    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = borrower,
        associated_token::mint = mint,
        associated_token::authority = borrower
    )]
    pub borrower_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = config
    )]
    pub protocol_ata: Account<'info, TokenAccount>,
    #[account(
        address = INSTRUCTIONS_SYSVAR_ID
    )]
    /// CHECK: InstructionsSysvar account
    pub instructions: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Borrow<'info> {
    pub fn borrow_process(&mut self, borrow_amount: u64, bumps: BorrowBumps) -> Result<()> {
        require!(borrow_amount > 0, ProtocolError::InvalidAmount);
        require!(
            self.config.net_deposits >= borrow_amount,
            ProtocolError::NotEnoughFunds
        );

        // Derive the Signer Seeds for the Protocol Account
        let seeds = &[b"config".as_ref(), &[bumps.config]];
        let signer_seeds = &[&seeds[..]];

        // Transfer the funds from the protocol to the borrower
        transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.protocol_ata.to_account_info(),
                    to: self.borrower_ata.to_account_info(),
                    authority: self.config.to_account_info(),
                },
                signer_seeds,
            ),
            borrow_amount,
        )?;

        let ixs = self.instructions.to_account_info();

        // Repay Instruction Check
        // Make sure that the last instruction of this transaction is a repay instruction
        let instruction_sysvar = ixs.try_borrow_data()?;
        let len = u16::from_le_bytes(instruction_sysvar[0..2].try_into().unwrap());

        if let Ok(repay_ix) = load_instruction_at_checked(len as usize - 1, &ixs) {
            // Instruction checks
            require_keys_eq!(repay_ix.program_id, ID, ProtocolError::InvalidProgram);
            require!(
                repay_ix.data[0..8].eq(instruction::Repay::DISCRIMINATOR),
                ProtocolError::InvalidIx
            );

            // We could check the Wallet and Mint separately but by checking the ATA we do this automatically
            require_keys_eq!(
                repay_ix
                    .accounts
                    .get(3)
                    .ok_or(ProtocolError::InvalidBorrowerAta)?
                    .pubkey,
                self.borrower_ata.key(),
                ProtocolError::InvalidBorrowerAta
            );
            require_keys_eq!(
                repay_ix
                    .accounts
                    .get(4)
                    .ok_or(ProtocolError::InvalidProtocolAta)?
                    .pubkey,
                self.protocol_ata.key(),
                ProtocolError::InvalidProtocolAta
            );
            self.config.net_deposits = self
                .config
                .net_deposits
                .checked_sub(borrow_amount)
                .ok_or(ProtocolError::Underflow)?;

            emit!(BorrowEvent {
                amount: borrow_amount,
                user: self.borrower.key(),
            })
        } else {
            return Err(ProtocolError::MissingRepayIx.into());
        }
        Ok(())
    }
}
