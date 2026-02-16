use anchor_lang::{
    prelude::*,
    solana_program::sysvar::instructions::{
        load_current_index_checked, load_instruction_at_checked,
    },
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
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
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
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

    /// CHECK: instructions sysvar account
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self) -> Result<()> {
        let amount = self.check_transfer_instruction()?;

        match self.user_state.balance.checked_add(amount) {
            Some(x) => self.user_state.balance = x,
            None => {} // add error
        }

        Ok(())
    }

    pub fn check_transfer_instruction(&self) -> Result<u64> {
        // instruction introspection
        let current_index =
            load_current_index_checked(&self.instructions.to_account_info())? as usize;
        let ix =
            load_instruction_at_checked(current_index - 1, &self.instructions.to_account_info())?;

        // check above ix is from token22 program and transferchecked
        require_keys_eq!(ix.program_id, anchor_spl::token_2022::ID);
        require_eq!(ix.data.split_first().unwrap().0, &12);

        let authority = ix.accounts.get(4).unwrap();
        require_keys_eq!(authority.pubkey, self.user.key());

        let amount_bytes = &ix.data[1..9];
        let amount = u64::from_le_bytes(amount_bytes.try_into().unwrap());

        Ok(amount)
    }
}
