use anchor_lang::prelude::*;
// use anchor_lang::solana_program::hash::hash;
use anchor_spl::{
    associated_token::{
        spl_associated_token_account::solana_program::keccak::hash, AssociatedToken,
    },
    token_2022::TransferChecked,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

use crate::state::{User, Vault};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_state.bump,
    )]
    pub user_state: Account<'info, User>,

    #[account(
        init, // init_if_needed
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init, // init_if_needed
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program
    )]
    pub vault_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault"],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mint::token_program = token_program,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64, bumps: &DepositBumps) -> Result<()> {
        match self.user_state.balance.checked_add(amount) {
            Some(x) => self.user_state.balance = x,
            None => {} // add error
        }
        Ok(())
    }

    pub fn check_transfer_instruction(&self) -> Result<()> {
        // instruction introspection
        Ok(())
    }
}
