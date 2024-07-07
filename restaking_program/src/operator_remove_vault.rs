use jito_restaking_core::{
    config::SanitizedConfig, operator::SanitizedOperator,
    operator_vault_list::SanitizedOperatorVaultList,
};
use jito_restaking_sanitization::signer::SanitizedSignerAccount;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

/// [`crate::RestakingInstruction::OperatorRemoveVault`]
pub fn process_operator_remove_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let SanitizedAccounts {
        operator,
        mut operator_vault_list,
        admin,
        vault,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    operator.operator().check_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;
    operator_vault_list
        .operator_vault_list_mut()
        .remove_vault(*vault.key, slot)?;

    operator_vault_list.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    // config: SanitizedConfig<'a, 'info>,
    operator: SanitizedOperator<'a, 'info>,
    operator_vault_list: SanitizedOperatorVaultList<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    vault: &'a AccountInfo<'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitizes the accounts for the instruction: [`crate::RestakingInstruction::OperatorRemoveVault`]
    pub fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let _config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let operator =
            SanitizedOperator::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let operator_vault_list = SanitizedOperatorVaultList::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            true,
            operator.account().key,
        )?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
        // TODO (LB): should run more verification on the vault here?
        //  program owner? deserialize it/check header?
        let vault = next_account_info(accounts_iter)?;

        Ok(SanitizedAccounts {
            // config,
            operator,
            operator_vault_list,
            admin,
            vault,
        })
    }
}
