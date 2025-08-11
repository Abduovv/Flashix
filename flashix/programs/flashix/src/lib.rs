#[allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

mod states;
mod instructions;
mod errors;
mod emits;


use instructions::*;
// Align program ID with Anchor.toml [programs.localnet]
declare_id!("FKwjo9xTr2CVns9xDfwSzBFGD3GP5y1JBVFMTv7KuiGn");
	     	
#[program]
pub mod blueshift_flash_loan {
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


