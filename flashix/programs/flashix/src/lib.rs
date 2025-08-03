#[allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

mod states;
mod instructions;
mod errors;
mod emits;

use instructions::*;
declare_id!("9pJ3k2bUo5tQQXhsyEM9uxJ3PobZ2DNWzgxGjPxRHXhm");
	     	
#[program]
pub mod flash_loan {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, basis_points: u16) -> Result<()> {
        ctx.accounts.initialize(basis_points, ctx.bumps)?;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)?;
        Ok(())
    }
    pub fn borrow(ctx: Context<Borrow>, borrow_amount: u64) -> Result<()> {
        ctx.accounts.borrow_process(borrow_amount, ctx.bumps)?;
        Ok(())
    }

    pub fn repay(ctx: Context<Repay>) -> Result<()> {
        ctx.accounts.repay_process()?;
        Ok(())
    }
}


