use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    msg,
};
use solana_program::program_pack::{IsInitialized, Pack, Sealed};
use arrayref::{array_ref, array_refs, array_mut_ref, mut_array_refs};
use std::convert::TryInto;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub enum DHelixError {
    InvalidDestinationAccount,
    InsufficientFunds,
    OverflowError,
    UnderflowError,
    Unauthorized,
    InvalidMultisigAccount,
    AccountLocked,
}

impl From<DHelixError> for ProgramError {
    fn from(e: DHelixError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

pub struct DHelixToken;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TokenAccount {
    pub is_initialized: bool,
    pub owner: Pubkey,
    pub amount: u64,
}

impl Sealed for TokenAccount {}

impl IsInitialized for TokenAccount {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for TokenAccount {
    const LEN: usize = 41;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if src.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let src = array_ref![src, 0, TokenAccount::LEN];
        let (is_initialized, owner, amount) = array_refs![src, 1, 32, 8];
        let is_initialized = is_initialized[0] != 0;
        let owner = Pubkey::new_from_array(*owner);
        let amount = u64::from_le_bytes(*amount);
        Ok(TokenAccount {
            is_initialized,
            owner,
            amount,
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        if dst.len() != Self::LEN {
            return;
        }
        let dst = array_mut_ref![dst, 0, TokenAccount::LEN];
        let (is_initialized_dst, owner_dst, amount_dst) = mut_array_refs![dst, 1, 32, 8];
        is_initialized_dst[0] = self.is_initialized as u8;
        owner_dst.copy_from_slice(self.owner.as_ref());
        *amount_dst = self.amount.to_le_bytes();
    }
}

impl DHelixToken {
    /// Mints a specified amount of tokens to the destination account.
    /// Checks for overflow and ensures the mint account is a signer.
    pub fn mint(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        if accounts.len() < 2 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        
        let account_info_iter = &mut accounts.iter();
        let mint_account = next_account_info(account_info_iter)?;
        let destination_account = next_account_info(account_info_iter)?;

        if !mint_account.is_signer {
            msg!("Error: Mint account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Ensure the mint account is authorized
        let mint_authority_pubkey = Pubkey::from_str("GSqP2u5zXbESXXxmLzJAs9cXpkbCSejyy5RSJsWVEADZ").unwrap();
        if mint_account.key != &mint_authority_pubkey {
            msg!("Error: Mint account is not authorized");
            return Err(DHelixError::Unauthorized.into());
        }

        if !destination_account.is_writable {
            msg!("Error: Destination account is not writable");
            return Err(DHelixError::InvalidDestinationAccount.into());
        }

        let mut destination_token_account = TokenAccount::unpack_unchecked(&destination_account.data.borrow())?;
        destination_token_account.amount = destination_token_account.amount.checked_add(amount).ok_or::<ProgramError>(DHelixError::OverflowError.into())?;

        TokenAccount::pack(destination_token_account, &mut destination_account.data.borrow_mut())?;
        msg!("Minted {} tokens to {}", amount, destination_account.key);

        Ok(())
    }

    /// Transfers a specified amount of tokens from the source account to the destination account.
    /// Checks for underflow and overflow and ensures the source account is a signer.
    pub fn transfer(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        if accounts.len() < 2 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let account_info_iter = &mut accounts.iter();
        let source_account = next_account_info(account_info_iter)?;
        let destination_account = next_account_info(account_info_iter)?;

        if !source_account.is_signer {
            msg!("Error: Source account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !source_account.is_writable || !destination_account.is_writable {
            msg!("Error: Source or destination account is not writable");
            return Err(DHelixError::InvalidDestinationAccount.into());
        }

        let mut source_token_account = TokenAccount::unpack_unchecked(&source_account.data.borrow())?;
        if source_token_account.amount < amount {
            msg!("Error: Insufficient funds in source account");
            return Err(DHelixError::InsufficientFunds.into());
        }

        let mut destination_token_account = TokenAccount::unpack_unchecked(&destination_account.data.borrow())?;
        source_token_account.amount = source_token_account.amount.checked_sub(amount).ok_or_else(|| {
            msg!("Error: Underflow occurred during balance subtraction");
            DHelixError::UnderflowError
        })?;
        destination_token_account.amount = destination_token_account.amount.checked_add(amount).ok_or_else(|| {
            msg!("Error: Overflow occurred during balance addition");
            DHelixError::OverflowError
        })?;

        TokenAccount::pack(source_token_account, &mut source_account.data.borrow_mut())?;
        TokenAccount::pack(destination_token_account, &mut destination_account.data.borrow_mut())?;
        
        msg!("Transferring {} tokens from {} to {}", amount, source_account.key, destination_account.key);

        // Log event
        msg!("Event: Transfer {{ amount: {}, source: {}, destination: {} }}", amount, source_account.key, destination_account.key);

        Ok(())
    }

    /// Burns a specified amount of tokens from the burn account.
    /// Checks for underflow and ensures the burn account is a signer.
    pub fn burn(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        if accounts.len() < 1 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let account_info_iter = &mut accounts.iter();
        let burn_account = next_account_info(account_info_iter)?;

        if !burn_account.is_signer {
            msg!("Error: Burn account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Ensure the burn account is authorized
        let burn_authority_pubkey = Pubkey::from_str("AxGavuYn6HHY95AjPyTaZHEpeKAgRJq4gAPJriC3iYP5").unwrap();
        if burn_account.key != &burn_authority_pubkey {
            msg!("Error: Burn account is not authorized");
            return Err(DHelixError::Unauthorized.into());
        }

        if !burn_account.is_writable {
            msg!("Error: Burn account is not writable");
            return Err(DHelixError::InvalidDestinationAccount.into());
        }

        let mut burn_token_account = TokenAccount::unpack_unchecked(&burn_account.data.borrow())?;
        if burn_token_account.amount < amount {
            msg!("Error: Insufficient funds in burn account");
            return Err(DHelixError::InsufficientFunds.into());
        }

        burn_token_account.amount = burn_token_account.amount.checked_sub(amount).ok_or_else(|| {
            msg!("Error: Underflow occurred during balance subtraction");
            DHelixError::UnderflowError
        })?;

        TokenAccount::pack(burn_token_account, &mut burn_account.data.borrow_mut())?;
        
        msg!("Burning {} tokens from {}", amount, burn_account.key);

        // Log event
        msg!("Event: Burn {{ amount: {}, burner: {} }}", amount, burn_account.key);

        Ok(())
    }

    /// Multisig functionality to ensure the correct number of signers and signatures.
    pub fn multisig(accounts: &[AccountInfo], required_signatures: u8) -> ProgramResult {
        if accounts.len() < 1 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let mut account_info_iter = accounts.iter();
        let multisig_account = next_account_info(&mut account_info_iter)?;

        if !multisig_account.is_signer {
            msg!("Error: Multisig account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !multisig_account.is_writable {
            msg!("Error: Multisig account is not writable");
            return Err(DHelixError::InvalidDestinationAccount.into());
        }

        let mut signature_count = 0;
        for account in account_info_iter {
            if account.is_signer {
                signature_count += 1;
            }
        }

        if signature_count < required_signatures {
            msg!("Error: Not enough signers");
            return Err(DHelixError::Unauthorized.into());
        }

        msg!("Multi-signature operation with {} signers", signature_count);

        // Log event
        msg!("Event: Multisig {{ required_signatures: {}, signers: {} }}", required_signatures, signature_count);

        Ok(())
    }

    /// Time-lock functionality to lock the account until a specific time.
    pub fn time_lock(accounts: &[AccountInfo], unlock_time: u64) -> ProgramResult {
        if accounts.len() < 1 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let account_info_iter = &mut accounts.iter();
        let time_lock_account = next_account_info(account_info_iter)?;

        if !time_lock_account.is_signer {
            msg!("Error: Time-lock account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let current_time = get_current_time()?;
        if current_time < unlock_time {
            return Err(DHelixError::AccountLocked.into());
        }

        msg!("Time-lock operation until {}", unlock_time);
        // Log event
        msg!("Event: TimeLock {{ account: {}, unlock_time: {} }}", time_lock_account.key, unlock_time);

        Ok(())
    }

    /// Emergency stop functionality to halt critical operations.
    pub fn emergency_stop(accounts: &[AccountInfo]) -> ProgramResult {
        if accounts.len() < 1 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let account_info_iter = &mut accounts.iter();
        let emergency_stop_account = next_account_info(account_info_iter)?;

        if !emergency_stop_account.is_signer {
            msg!("Error: Emergency stop account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        halt_operations();

        msg!("Emergency stop operation");
        // Log event
        msg!("Event: EmergencyStop {{ account: {} }}", emergency_stop_account.key);

        Ok(())
    }
}

pub struct DHelixDAO;

impl DHelixDAO {
    /// Submits a proposal with a specified ID and data.
    /// Ensures the proposer account is a signer.
    pub fn submit_proposal(accounts: &[AccountInfo], proposal_id: u64, proposal_data: &[u8]) -> ProgramResult {
        if accounts.len() < 1 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let account_info_iter = &mut accounts.iter();
        let proposer_account = next_account_info(account_info_iter)?;

        if !proposer_account.is_signer {
            msg!("Error: Proposer account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        msg!("Submitting proposal ID: {} by {}", proposal_id, proposer_account.key);
        store_proposal(proposal_id, proposal_data)?;

        // Log event
        msg!("Event: ProposalSubmitted {{ proposal_id: {}, proposer: {} }}", proposal_id, proposer_account.key);

        Ok(())
    }

    /// Votes on a proposal with a specified ID and vote value.
    /// Ensures the voter account is a signer.
    pub fn vote(accounts: &[AccountInfo], proposal_id: u64, vote: bool) -> ProgramResult {
        if accounts.len() < 1 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let account_info_iter = &mut accounts.iter();
        let voter_account = next_account_info(account_info_iter)?;

        if !voter_account.is_signer {
            msg!("Error: Voter account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        msg!("Voting on proposal ID: {} by {}", proposal_id, voter_account.key);
        record_vote(proposal_id, voter_account.key, vote)?;

        // Log event
        msg!("Event: Vote {{ proposal_id: {}, voter: {}, vote: {} }}", proposal_id, voter_account.key, vote);

        Ok(())
    }

    /// Executes a proposal with a specified ID.
    /// Ensures the executor account is a signer.
    pub fn execute_proposal(accounts: &[AccountInfo], proposal_id: u64) -> ProgramResult {
        if accounts.len() < 1 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let account_info_iter = &mut accounts.iter();
        let executor_account = next_account_info(account_info_iter)?;

        if !executor_account.is_signer {
            msg!("Error: Executor account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        msg!("Executing proposal ID: {} by {}", proposal_id, executor_account.key);
        execute_proposal_logic(proposal_id)?;

        // Log event
        msg!("Event: ProposalExecuted {{ proposal_id: {}, executor: {} }}", proposal_id, executor_account.key);

        Ok(())
    }

    /// Charity vote functionality.
    /// Ensures the voter account is a signer.
    pub fn charity_vote(accounts: &[AccountInfo], proposal_id: u64, vote: bool) -> ProgramResult {
        if accounts.len() < 1 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let account_info_iter = &mut accounts.iter();
        let voter_account = next_account_info(account_info_iter)?;

        if !voter_account.is_signer {
            msg!("Error: Voter account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        msg!("Charity vote on proposal ID: {} by {}", proposal_id, voter_account.key);
        record_vote(proposal_id, voter_account.key, vote)?;

        // Log event
        msg!("Event: CharityVote {{ proposal_id: {}, voter: {}, vote: {} }}", proposal_id, voter_account.key, vote);

        Ok(())
    }

    /// Future project vote functionality.
    /// Ensures the voter account is a signer.
    pub fn future_project_vote(accounts: &[AccountInfo], proposal_id: u64, vote: bool) -> ProgramResult {
        if accounts.len() < 1 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let account_info_iter = &mut accounts.iter();
        let voter_account = next_account_info(account_info_iter)?;

        if !voter_account.is_signer {
            msg!("Error: Voter account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        msg!("Future project vote on proposal ID: {} by {}", proposal_id, voter_account.key);
        record_vote(proposal_id, voter_account.key, vote)?;

        // Log event
        msg!("Event: FutureProjectVote {{ proposal_id: {}, voter: {}, vote: {} }}", proposal_id, voter_account.key, vote);

        Ok(())
    }
}

impl DHelixToken {
    /// Incentivized voting system functionality.
    /// Ensures the voter account is a signer.
    pub fn incentivized_voting_system(accounts: &[AccountInfo], proposal_id: u64, vote: bool) -> ProgramResult {
        if accounts.len() < 1 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let account_info_iter = &mut accounts.iter();
        let voter_account = next_account_info(account_info_iter)?;

        if !voter_account.is_signer {
            msg!("Error: Voter account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        msg!("Incentivized voting on proposal ID: {} by {}", proposal_id, voter_account.key);
        reward_voter(voter_account.key)?;

        // Log event
        msg!("Event: IncentivizedVote {{ proposal_id: {}, voter: {}, vote: {} }}", proposal_id, voter_account.key, vote);

        Ok(())
    }

    /// Dynamic staking rewards functionality.
    /// Ensures the staker account is a signer.
    pub fn dynamic_staking_rewards(accounts: &[AccountInfo], staking_duration: u64) -> ProgramResult {
        if accounts.len() < 1 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let account_info_iter = &mut accounts.iter();
        let staker_account = next_account_info(account_info_iter)?;

        if !staker_account.is_signer {
            msg!("Error: Staker account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        msg!("Calculating staking rewards for {} by staking duration {}", staker_account.key, staking_duration);
        calculate_rewards(staker_account.key, staking_duration)?;

        // Log event
        msg!("Event: StakingRewards {{ staker: {}, staking_duration: {} }}", staker_account.key, staking_duration);

        Ok(())
    }

    /// Token buyback program functionality.
    /// Ensures the buyback account is a signer.
    pub fn token_buyback_program(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        if accounts.len() < 1 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let account_info_iter = &mut accounts.iter();
        let buyback_account = next_account_info(account_info_iter)?;

        if !buyback_account.is_signer {
            msg!("Error: Buyback account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        msg!("Executing token buyback for {} tokens", amount);
        buyback_tokens(buyback_account.key, amount)?;

        // Log event
        msg!("Event: Buyback {{ amount: {}, buyer: {} }}", amount, buyback_account.key);

        Ok(())
    }

    /// Insurance pool functionality.
    /// Ensures the insurance account is a signer.
    pub fn insurance_pool(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        if accounts.len() < 1 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let account_info_iter = &mut accounts.iter();
        let insurance_account = next_account_info(account_info_iter)?;

        if !insurance_account.is_signer {
            msg!("Error: Insurance account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        msg!("Contributing {} to the insurance pool", amount);
        contribute_to_pool(insurance_account.key, amount)?;

        // Log event
        msg!("Event: InsuranceContribution {{ amount: {}, contributor: {} }}", amount, insurance_account.key);

        Ok(())
    }
}

fn get_current_time() -> Result<u64, ProgramError> {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH)
        .map_err(|_| ProgramError::InvalidArgument)?;
    Ok(since_the_epoch.as_secs())
}

fn halt_operations() {
    // Add logic to halt operations
    msg!("Operations halted");
}

fn store_proposal(proposal_id: u64, proposal_data: &[u8]) -> Result<(), ProgramError> {
    // Add logic to store the proposal
    msg!("Proposal stored: ID: {}, Data: {:?}", proposal_id, proposal_data);
    Ok(())
}

fn record_vote(proposal_id: u64, voter: &Pubkey, vote: bool) -> Result<(), ProgramError> {
    // Add logic to record the vote
    msg!("Vote recorded: Proposal ID: {}, Voter: {}, Vote: {}", proposal_id, voter, vote);
    Ok(())
}

fn execute_proposal_logic(proposal_id: u64) -> Result<(), ProgramError> {
    // Add logic to execute the proposal
    msg!("Proposal executed: ID: {}", proposal_id);
    Ok(())
}

fn reward_voter(voter: &Pubkey) -> Result<(), ProgramError> {
    // Add logic to reward the voter
    msg!("Voter rewarded: {}", voter);
    Ok(())
}

fn calculate_rewards(staker: &Pubkey, staking_duration: u64) -> Result<(), ProgramError> {
    // Add logic to calculate staking rewards
    msg!("Rewards calculated for staker: {}, Duration: {}", staker, staking_duration);
    Ok(())
}

fn buyback_tokens(buyer: &Pubkey, amount: u64) -> Result<(), ProgramError> {
    // Add logic to buy back tokens
    msg!("Tokens bought back: Buyer: {}, Amount: {}", buyer, amount);
    Ok(())
}

fn contribute_to_pool(contributor: &Pubkey, amount: u64) -> Result<(), ProgramError> {
    // Add logic to contribute to the insurance pool
    msg!("Contribution to pool: Contributor: {}, Amount: {}", contributor, amount);
    Ok(())
}

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.len() < 9 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let instruction = instruction_data[0];

    match instruction {
        0 => {
            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            DHelixToken::mint(accounts, amount)
        },
        1 => {
            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            DHelixToken::transfer(accounts, amount)
        },
        2 => {
            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            DHelixToken::burn(accounts, amount)
        },
        3 => {
            let proposal_id = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            DHelixDAO::submit_proposal(accounts, proposal_id, &instruction_data[9..])
        },
        4 => {
            let proposal_id = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            let vote = instruction_data[9] != 0;
            DHelixDAO::vote(accounts, proposal_id, vote)
        },
        5 => {
            let proposal_id = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            DHelixDAO::execute_proposal(accounts, proposal_id)
        },
        6 => DHelixToken::multisig(accounts, instruction_data[1]),
        7 => {
            let time = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            DHelixToken::time_lock(accounts, time)
        },
        8 => DHelixToken::emergency_stop(accounts),
        9 => {
            let proposal_id = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            let vote = instruction_data[9] != 0;
            DHelixDAO::charity_vote(accounts, proposal_id, vote)
        },
        10 => {
            let proposal_id = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            let vote = instruction_data[9] != 0;
            DHelixDAO::future_project_vote(accounts, proposal_id, vote)
        },
        11 => {
            let proposal_id = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            let vote = instruction_data[9] != 0;
            DHelixToken::incentivized_voting_system(accounts, proposal_id, vote)
        },
        12 => {
            let staking_duration = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            DHelixToken::dynamic_staking_rewards(accounts, staking_duration)
        },
        13 => {
            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            DHelixToken::token_buyback_program(accounts, amount)
        },
        14 => {
            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            DHelixToken::insurance_pool(accounts, amount)
        },
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::{
        account_info::AccountInfo,
        clock::Epoch,
        pubkey::Pubkey,
    };
    use std::cell::RefCell;
    use std::rc::Rc;

    fn create_account_info<'a>(
        key: &'a Pubkey,
        is_signer: bool,
        is_writable: bool,
        lamports: &'a mut u64,
        data: &'a mut [u8],
        owner: &'a Pubkey,
    ) -> AccountInfo<'a> {
        AccountInfo {
            key,
            is_signer,
            is_writable,
            lamports: Rc::new(RefCell::new(lamports)),
            data: Rc::new(RefCell::new(data)),
            owner,
            executable: false,
            rent_epoch: Epoch::default(),
        }
    }

    #[test]
    fn test_mint() {
        let program_id = Pubkey::new_unique();
        let mint_authority_pubkey = Pubkey::from_str("GSqP2u5zXbESXXxmLzJAs9cXpkbCSejyy5RSJsWVEADZ").unwrap();
        let mut mint_account_lamports = 500;
        let mut destination_account_lamports = 100;
        let mut mint_account_data = vec![0; TokenAccount::LEN];
        let mut destination_account_data = vec![0; TokenAccount::LEN];
        let mint_key = mint_authority_pubkey; // Set the mint account key to the authorized pubkey
        let destination_key = Pubkey::new_unique();
        let mint_account = create_account_info(&mint_key, true, true, &mut mint_account_lamports, &mut mint_account_data, &program_id);
        let destination_account = create_account_info(&destination_key, false, true, &mut destination_account_lamports, &mut destination_account_data, &program_id);
        let accounts = vec![mint_account, destination_account.clone()];

        // Initialize destination account as a TokenAccount
        let mut dest_token_account = TokenAccount {
            is_initialized: true,
            owner: destination_key,
            amount: 100,
        };
        TokenAccount::pack(dest_token_account.clone(), &mut destination_account.data.borrow_mut()).unwrap();

        // Test valid minting
        let amount = 100;
        let result = DHelixToken::mint(&accounts, amount);
        assert!(result.is_ok());
        dest_token_account.amount += amount;
        assert_eq!(TokenAccount::unpack(&destination_account.data.borrow()).unwrap(), dest_token_account);

        // Test overflow
        let result = DHelixToken::mint(&accounts, u64::MAX);
        assert!(result.is_err());

        // Test without signer
        let destination_account_no_signer = create_account_info(&destination_key, false, true, &mut destination_account_lamports, &mut destination_account_data, &program_id);
        let accounts_no_signer = vec![destination_account_no_signer];
        let result = DHelixToken::mint(&accounts_no_signer, amount);
        assert!(result.is_err());
    }

    #[test]
    fn test_transfer() {
        let program_id = Pubkey::new_unique();
        let mut source_account_lamports = 700;
        let mut destination_account_lamports = 100;
        let mut source_account_data = vec![0; TokenAccount::LEN];
        let mut destination_account_data = vec![0; TokenAccount::LEN];
        let source_key = Pubkey::new_unique();
        let destination_key = Pubkey::new_unique();
        let source_account = create_account_info(&source_key, true, true, &mut source_account_lamports, &mut source_account_data, &program_id);
        let destination_account = create_account_info(&destination_key, false, true, &mut destination_account_lamports, &mut destination_account_data, &program_id);
        let accounts = vec![source_account, destination_account.clone()];

        // Initialize source and destination accounts as TokenAccounts
        let mut src_token_account = TokenAccount {
            is_initialized: true,
            owner: source_key,
            amount: 700,
        };
        let mut dest_token_account = TokenAccount {
            is_initialized: true,
            owner: destination_key,
            amount: 100,
        };
        TokenAccount::pack(src_token_account.clone(), &mut accounts[0].data.borrow_mut()).unwrap();
        TokenAccount::pack(dest_token_account.clone(), &mut accounts[1].data.borrow_mut()).unwrap();

        // Test valid transfer
        let amount = 200;
        let result = DHelixToken::transfer(&accounts, amount);
        assert!(result.is_ok());
        src_token_account.amount -= amount;
        dest_token_account.amount += amount;
        assert_eq!(TokenAccount::unpack(&accounts[0].data.borrow()).unwrap(), src_token_account);
        assert_eq!(TokenAccount::unpack(&accounts[1].data.borrow()).unwrap(), dest_token_account);

        // Test underflow
        let result = DHelixToken::transfer(&accounts, 1000);
        assert!(result.is_err());

        // Test without signer
        let destination_account_no_signer = create_account_info(&destination_key, false, true, &mut destination_account_lamports, &mut destination_account_data, &program_id);
        let accounts_no_signer = vec![destination_account_no_signer];
        let result = DHelixToken::transfer(&accounts_no_signer, amount);
        assert!(result.is_err());
    }

    #[test]
    fn test_burn() {
        let program_id = Pubkey::new_unique();
        let burn_authority_pubkey = Pubkey::from_str("AxGavuYn6HHY95AjPyTaZHEpeKAgRJq4gAPJriC3iYP5").unwrap();
        let mut burn_account_lamports = 500;
        let mut burn_account_data = vec![0; TokenAccount::LEN];
        let burn_key = burn_authority_pubkey; // Set the burn account key to the authorized pubkey
        let burn_account = create_account_info(&burn_key, true, true, &mut burn_account_lamports, &mut burn_account_data, &program_id);
        let accounts = vec![burn_account.clone()];

        // Initialize burn account as a TokenAccount
        let mut burn_token_account = TokenAccount {
            is_initialized: true,
            owner: burn_key,
            amount: 500,
        };
        TokenAccount::pack(burn_token_account.clone(), &mut accounts[0].data.borrow_mut()).unwrap();

        // Test valid burn
        let amount = 200;
        let result = DHelixToken::burn(&accounts, amount);
        assert!(result.is_ok());
        burn_token_account.amount -= amount;
        assert_eq!(TokenAccount::unpack(&accounts[0].data.borrow()).unwrap(), burn_token_account);

        // Test underflow
        let result = DHelixToken::burn(&accounts, 1000);
        assert!(result.is_err());

        // Test without signer
        let burn_account_no_signer = create_account_info(&burn_key, false, true, &mut burn_account_lamports, &mut burn_account_data, &program_id);
        let accounts_no_signer = vec![burn_account_no_signer];
        let result = DHelixToken::burn(&accounts_no_signer, amount);
        assert!(result.is_err());
    }

    #[test]
    fn test_multisig() {
        let program_id = Pubkey::new_unique();
        let mut multisig_account_lamports = 300;
        let mut multisig_account_data = vec![0; 100];
        let multisig_key = Pubkey::new_unique();
        let multisig_account = create_account_info(&multisig_key, true, true, &mut multisig_account_lamports, &mut multisig_account_data, &program_id);
        
        let mut signer_lamports = 300;
        let mut signer_account_data = vec![0; 100];  // Separate data for signer account
        let signer_key = Pubkey::new_unique();
        let signer_account = create_account_info(&signer_key, true, true, &mut signer_lamports, &mut signer_account_data, &program_id);
        
        let accounts = vec![multisig_account.clone(), signer_account];

        // Test valid multisig
        let required_signatures = 1;
        let result = DHelixToken::multisig(&accounts, required_signatures);
        assert!(result.is_ok());

        // Test not enough signers
        let accounts_no_signer = vec![multisig_account];
        let result = DHelixToken::multisig(&accounts_no_signer, required_signatures);
        assert!(result.is_err());
    }

    #[test]
    fn test_time_lock() {
        let program_id = Pubkey::new_unique();
        let mut time_lock_account_lamports = 300;
        let mut time_lock_account_data = vec![0; 100];
        let time_lock_key = Pubkey::new_unique();
        let time_lock_account = create_account_info(&time_lock_key, true, true, &mut time_lock_account_lamports, &mut time_lock_account_data, &program_id);
        let accounts = vec![time_lock_account.clone()];

        let unlock_time = get_current_time().unwrap() + 1000; // Setting unlock time to future
        let result = DHelixToken::time_lock(&accounts, unlock_time);
        assert!(result.is_err()); // Should be locked

        let unlock_time = get_current_time().unwrap(); // Setting unlock time to current time
        let result = DHelixToken::time_lock(&accounts, unlock_time);
        assert!(result.is_ok()); // Should be unlocked
    }

    #[test]
    fn test_emergency_stop() {
        let program_id = Pubkey::new_unique();
        let mut emergency_stop_account_lamports = 300;
        let mut emergency_stop_account_data = vec![0; 100];
        let emergency_stop_key = Pubkey::new_unique();
        let emergency_stop_account = create_account_info(&emergency_stop_key, true, true, &mut emergency_stop_account_lamports, &mut emergency_stop_account_data, &program_id);
        let accounts = vec![emergency_stop_account.clone()];

        let result = DHelixToken::emergency_stop(&accounts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_submit_proposal() {
        let program_id = Pubkey::new_unique();
        let mut proposer_account_lamports = 300;
        let mut proposer_account_data = vec![0; 100];
        let proposer_key = Pubkey::new_unique();
        let proposer_account = create_account_info(&proposer_key, true, true, &mut proposer_account_lamports, &mut proposer_account_data, &program_id);
        let accounts = vec![proposer_account.clone()];

        let proposal_id = 1;
        let proposal_data = vec![0; 100];
        let result = DHelixDAO::submit_proposal(&accounts, proposal_id, &proposal_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_vote() {
        let program_id = Pubkey::new_unique();
        let mut voter_account_lamports = 300;
        let mut voter_account_data = vec![0; 100];
        let voter_key = Pubkey::new_unique();
        let voter_account = create_account_info(&voter_key, true, true, &mut voter_account_lamports, &mut voter_account_data, &program_id);
        let accounts = vec![voter_account.clone()];

        let proposal_id = 1;
        let vote = true;
        let result = DHelixDAO::vote(&accounts, proposal_id, vote);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_proposal() {
        let program_id = Pubkey::new_unique();
        let mut executor_account_lamports = 300;
        let mut executor_account_data = vec![0; 100];
        let executor_key = Pubkey::new_unique();
        let executor_account = create_account_info(&executor_key, true, true, &mut executor_account_lamports, &mut executor_account_data, &program_id);
        let accounts = vec![executor_account.clone()];

        let proposal_id = 1;
        let result = DHelixDAO::execute_proposal(&accounts, proposal_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_charity_vote() {
        let program_id = Pubkey::new_unique();
        let mut voter_account_lamports = 300;
        let mut voter_account_data = vec![0; 100];
        let voter_key = Pubkey::new_unique();
        let voter_account = create_account_info(&voter_key, true, true, &mut voter_account_lamports, &mut voter_account_data, &program_id);
        let accounts = vec![voter_account.clone()];

        let proposal_id = 1;
        let vote = true;
        let result = DHelixDAO::charity_vote(&accounts, proposal_id, vote);
        assert!(result.is_ok());
    }

    #[test]
    fn test_future_project_vote() {
        let program_id = Pubkey::new_unique();
        let mut voter_account_lamports = 300;
        let mut voter_account_data = vec![0; 100];
        let voter_key = Pubkey::new_unique();
        let voter_account = create_account_info(&voter_key, true, true, &mut voter_account_lamports, &mut voter_account_data, &program_id);
        let accounts = vec![voter_account.clone()];

        let proposal_id = 1;
        let vote = true;
        let result = DHelixDAO::future_project_vote(&accounts, proposal_id, vote);
        assert!(result.is_ok());
    }

    #[test]
    fn test_incentivized_voting_system() {
        let program_id = Pubkey::new_unique();
        let mut voter_account_lamports = 300;
        let mut voter_account_data = vec![0; 100];
        let voter_key = Pubkey::new_unique();
        let voter_account = create_account_info(&voter_key, true, true, &mut voter_account_lamports, &mut voter_account_data, &program_id);
        let accounts = vec![voter_account.clone()];

        let proposal_id = 1;
        let vote = true;
        let result = DHelixToken::incentivized_voting_system(&accounts, proposal_id, vote);
        assert!(result.is_ok());
    }

    #[test]
    fn test_dynamic_staking_rewards() {
        let program_id = Pubkey::new_unique();
        let mut staker_account_lamports = 300;
        let mut staker_account_data = vec![0; 100];
        let staker_key = Pubkey::new_unique();
        let staker_account = create_account_info(&staker_key, true, true, &mut staker_account_lamports, &mut staker_account_data, &program_id);
        let accounts = vec![staker_account.clone()];

        let staking_duration = 100;
        let result = DHelixToken::dynamic_staking_rewards(&accounts, staking_duration);
        assert!(result.is_ok());
    }

    #[test]
    fn test_token_buyback_program() {
        let program_id = Pubkey::new_unique();
        let mut buyback_account_lamports = 300;
        let mut buyback_account_data = vec![0; 100];
        let buyback_key = Pubkey::new_unique();
        let buyback_account = create_account_info(&buyback_key, true, true, &mut buyback_account_lamports, &mut buyback_account_data, &program_id);
        let accounts = vec![buyback_account.clone()];

        let amount = 100;
        let result = DHelixToken::token_buyback_program(&accounts, amount);
        assert!(result.is_ok());
    }

    #[test]
    fn test_insurance_pool() {
        let program_id = Pubkey::new_unique();
        let mut insurance_account_lamports = 300;
        let mut insurance_account_data = vec![0; 100];
        let insurance_key = Pubkey::new_unique();
        let insurance_account = create_account_info(&insurance_key, true, true, &mut insurance_account_lamports, &mut insurance_account_data, &program_id);
        let accounts = vec![insurance_account.clone()];

        let amount = 100;
        let result = DHelixToken::insurance_pool(&accounts, amount);
        assert!(result.is_ok());
    }
}
