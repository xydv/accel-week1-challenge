use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::TransferChecked,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

use crate::state::Vault;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mint::decimals = 9,
        mint::authority = user,
        mint::token_program = user,
        extensions::transfer_hook::authority = user,
        extensions::transfer_hook::program_id = crate::ID,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

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
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64, bumps: &DepositBumps) -> Result<()> {
        let decimals = self.mint.decimals;

        let cpi_accounts = TransferChecked {
            mint: self.mint.to_account_info(),
            from: self.user_ata.to_account_info(),
            to: self.vault_ata.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let cpi_program = self.token_program.to_account_info();
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        token_interface::transfer_checked(cpi_context, amount, decimals)?;

        Ok(())
    }
}
