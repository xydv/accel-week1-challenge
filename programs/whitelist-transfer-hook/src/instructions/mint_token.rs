use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::MintTo,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

#[derive(Accounts)]
pub struct TokenFactory<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub user: SystemAccount<'info>,

    #[account(
        init_if_needed,
        payer = admin,
        mint::decimals = 9,
        mint::authority = admin,
        extensions::transfer_hook::authority = admin,
        extensions::transfer_hook::program_id = crate::ID,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> TokenFactory<'info> {
    pub fn init_mint(&mut self, amount: u64) -> Result<()> {
        let cpi = CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.mint.to_account_info(),
                to: self.user_ata.to_account_info(),
                authority: self.admin.to_account_info(),
            },
        );

        token_interface::mint_to(cpi, amount)?;

        Ok(())
    }
}
