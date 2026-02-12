use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
};

use crate::ID;

#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        init,
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump,
        space = ExtraAccountMetaList::size_of(
            InitializeExtraAccountMetaList::extra_account_metas()?.len()
        ).unwrap(),
        payer = payer
    )]
    pub extra_account_meta_list: AccountInfo<'info>,
    pub mint: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeExtraAccountMetaList<'info> {
    pub fn extra_account_metas() -> Result<Vec<ExtraAccountMeta>> {
        let (vault_pda, _bump) = Pubkey::find_program_address(&[b"vault"], &ID);
        let vault_meta =
            ExtraAccountMeta::new_with_pubkey(&vault_pda.to_bytes().into(), false, false).unwrap();

        let user_meta = ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: b"user".to_vec(),
                },
                Seed::AccountKey { index: 3 },
            ],
            false,
            false,
        )
        .unwrap();

        Ok(vec![vault_meta, user_meta])
    }
}
