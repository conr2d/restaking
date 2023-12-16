use borsh::{BorshDeserialize, BorshSerialize};
use jito_restaking_sanitization::assert_with_msg;
use solana_program::{
    account_info::AccountInfo, entrypoint_deprecated::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::AccountType;

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct Config {
    /// The account type
    account_type: AccountType,

    /// The configuration admin
    admin: Pubkey,

    /// The signer for restaking operations on a vault
    restaking_program_signer: Pubkey,

    /// The number of vaults managed by the program
    num_vaults: u64,

    /// Reserved space
    reserved: [u8; 1024],

    /// The bump seed for the PDA
    bump: u8,
}

impl Config {
    pub const fn new(admin: Pubkey, restaking_program_signer: Pubkey, bump: u8) -> Self {
        Self {
            account_type: AccountType::Config,
            admin,
            restaking_program_signer,
            num_vaults: 0,
            reserved: [0; 1024],
            bump,
        }
    }

    pub const fn admin(&self) -> Pubkey {
        self.admin
    }

    pub fn increment_vaults(&mut self) -> Option<u64> {
        self.num_vaults = self.num_vaults.checked_add(1)?;
        Some(self.num_vaults)
    }

    pub const fn vaults_count(&self) -> u64 {
        self.num_vaults
    }

    pub const fn bump(&self) -> u8 {
        self.bump
    }

    pub fn is_struct_valid(&self) -> bool {
        self.account_type == AccountType::Config
    }

    pub fn seeds() -> Vec<Vec<u8>> {
        vec![b"config".to_vec()]
    }

    pub const fn restaking_program_signer(&self) -> Pubkey {
        self.restaking_program_signer
    }

    pub fn find_program_address(program_id: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds();
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
    ) -> Result<Self, ProgramError> {
        assert_with_msg(
            !account.data_is_empty(),
            ProgramError::UninitializedAccount,
            "Config account is not initialized",
        )?;
        assert_with_msg(
            account.owner == program_id,
            ProgramError::InvalidAccountOwner,
            "Invalid Config account owner",
        )?;

        let config = Self::deserialize(&mut account.data.borrow_mut().as_ref())?;
        assert_with_msg(
            config.is_struct_valid(),
            ProgramError::InvalidAccountData,
            "Invalid Config account data",
        )?;

        // double check derivation address
        let mut seeds = Self::seeds();
        seeds.push(vec![config.bump()]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)?;

        assert_with_msg(
            expected_pubkey == *account.key,
            ProgramError::InvalidAccountData,
            "Invalid Config account address",
        )?;

        Ok(config)
    }
}

pub struct SanitizedConfig<'a, 'info> {
    account: &'a AccountInfo<'info>,
    config: Config,
}

impl<'a, 'info> SanitizedConfig<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
    ) -> Result<SanitizedConfig<'a, 'info>, ProgramError> {
        if expect_writable {
            assert_with_msg(
                account.is_writable,
                ProgramError::InvalidAccountData,
                "Invalid writable flag for Config",
            )?;
        }
        let config = Config::deserialize_checked(program_id, account)?;

        Ok(SanitizedConfig { account, config })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn config(&self) -> &Config {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(&mut self.account.data.borrow_mut()[..], &self.config)?;
        Ok(())
    }
}
