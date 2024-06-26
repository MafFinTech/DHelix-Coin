use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    msg,
    sysvar::{clock::Clock, Sysvar, SysvarId},
};
use solana_program::program_pack::{IsInitialized, Pack, Sealed};
use arrayref::{array_ref, array_refs, array_mut_ref, mut_array_refs};
use std::convert::TryInto;
use std::str::FromStr;
use std::collections::HashMap;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::sysvar;

#[derive(BorshSerialize, BorshDeserialize, Debug, Default, PartialEq)]
pub struct ProposalsState {
    pub proposals: HashMap<u64, Vec<u8>>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default, PartialEq)]
pub struct VotesState {
    pub votes: HashMap<u64, Vec<(Pubkey, bool)>>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default, PartialEq)]
pub struct BalancesState {
    pub balances: HashMap<Pubkey, u64>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default, PartialEq)]
pub struct SystemState {
    pub halt: bool,
    pub insurance_pool: u64,
}

pub fn store_proposals_state(account: &AccountInfo, state: &ProposalsState) -> Result<(), ProgramError> {
    let data = state.try_to_vec()?; // Serialize state to bytes
    let data_len = data.len();

    // Check if the account data is large enough to hold the serialized state plus the length prefix
    if account.data_len() < data_len + 8 {
        msg!("Error: Account data is too small to hold the serialized proposals state");
        return Err(ProgramError::AccountDataTooSmall);
    }

    // Validate the serialized data length
    if data_len == 0 || data_len > account.data_len() - 8 {
        msg!("Error: Serialized data length is invalid");
        return Err(ProgramError::InvalidAccountData);
    }
    
    let mut data_ref = account.data.borrow_mut();
    
    data_ref[..data_len].copy_from_slice(&data);
    
    // Save the length of the serialized data at the end of the account data
    let length_bytes = (data_len as u64).to_le_bytes();
    let data_ref_len = data_ref.len(); // Calculate length before mutable borrow
    data_ref[data_ref_len - 8..].copy_from_slice(&length_bytes);
    
    Ok(())
}

pub fn load_proposals_state(account: &AccountInfo) -> Result<ProposalsState, ProgramError> {
    let account_data = account.data.borrow();
    let data_len_position = account_data.len().checked_sub(8).ok_or(ProgramError::InvalidAccountData)?;
    let serialized_len_bytes = &account_data[data_len_position..];
    
    if serialized_len_bytes.iter().all(|&b| b == 0) {
        return Err(ProgramError::InvalidAccountData);
    }
    
    let serialized_len = usize::from_le_bytes(serialized_len_bytes.try_into().map_err(|_| ProgramError::InvalidAccountData)?);
    
    if serialized_len == 0 || serialized_len > data_len_position {
        return Err(ProgramError::InvalidAccountData);
    }

    let serialized_state = &account_data[..serialized_len];
    ProposalsState::try_from_slice(serialized_state).map_err(|_| ProgramError::InvalidAccountData)
}

pub fn store_votes_state(account: &AccountInfo, state: &VotesState) -> Result<(), ProgramError> {
    let data = state.try_to_vec()?; // Serialize state to bytes
    let data_len = data.len();

    // Validate the serialized data length
    if data_len == 0 || data_len > account.data_len() - 8 {
        msg!("Error: Serialized data length is invalid");
        return Err(ProgramError::InvalidAccountData);
    }
    
    // Check if the account data is large enough to hold the serialized state plus the length prefix
    if account.data_len() < data_len + 8 {
        msg!("Error: Account data is too small to hold the serialized votes state");
        return Err(ProgramError::AccountDataTooSmall);
    }

    let mut data_ref = account.data.borrow_mut();
    
    data_ref[..data_len].copy_from_slice(&data);
    
    // Save the length of the serialized data at the end of the account data
    let length_bytes = (data_len as u64).to_le_bytes();
    let data_ref_len = data_ref.len(); // Calculate length before mutable borrow
    data_ref[data_ref_len - 8..].copy_from_slice(&length_bytes);
    
    Ok(())
}

pub fn load_votes_state(account: &AccountInfo) -> Result<VotesState, ProgramError> {
    let account_data = account.data.borrow();
    let data_len_position = account_data.len().checked_sub(8).ok_or(ProgramError::InvalidAccountData)?;
    let serialized_len_bytes = &account_data[data_len_position..];
    
    if serialized_len_bytes.iter().all(|&b| b == 0) {
        return Err(ProgramError::InvalidAccountData);
    }
    
    let serialized_len = usize::from_le_bytes(serialized_len_bytes.try_into().map_err(|_| ProgramError::InvalidAccountData)?);
    
    if serialized_len == 0 || serialized_len > data_len_position {
        return Err(ProgramError::InvalidAccountData);
    }

    let serialized_state = &account_data[..serialized_len];
    VotesState::try_from_slice(serialized_state).map_err(|_| ProgramError::InvalidAccountData)
}

pub fn store_balances_state(account: &AccountInfo, state: &BalancesState) -> Result<(), ProgramError> {
    let data = state.try_to_vec()?; // Serialize state to bytes
    let data_len = data.len();

    // Validate the serialized data length
    if data_len == 0 || data_len > account.data_len() - 8 {
        msg!("Error: Serialized data length is invalid");
        return Err(ProgramError::InvalidAccountData);
    }
    
    // Check if the account data is large enough to hold the serialized state plus the length prefix
    if account.data_len() < data_len + 8 {
        msg!("Error: Account data is too small to hold the serialized balances state");
        return Err(ProgramError::AccountDataTooSmall);
    }

    let mut data_ref = account.data.borrow_mut();
    
    data_ref[..data_len].copy_from_slice(&data);
    
    // Save the length of the serialized data at the end of the account data
    let length_bytes = (data_len as u64).to_le_bytes();
    let data_ref_len = data_ref.len(); // Calculate length before mutable borrow
    data_ref[data_ref_len - 8..].copy_from_slice(&length_bytes);
    
    Ok(())
}

pub fn load_balances_state(account: &AccountInfo) -> Result<BalancesState, ProgramError> {
    let account_data = account.data.borrow();
    let data_len_position = account_data.len().checked_sub(8).ok_or(ProgramError::InvalidAccountData)?;
    let serialized_len_bytes = &account_data[data_len_position..];
    
    if serialized_len_bytes.iter().all(|&b| b == 0) {
        return Err(ProgramError::InvalidAccountData);
    }
    
    let serialized_len = usize::from_le_bytes(serialized_len_bytes.try_into().map_err(|_| ProgramError::InvalidAccountData)?);
    
    if serialized_len == 0 || serialized_len > data_len_position {
        return Err(ProgramError::InvalidAccountData);
    }

    let serialized_state = &account_data[..serialized_len];
    BalancesState::try_from_slice(serialized_state).map_err(|_| ProgramError::InvalidAccountData)
}

pub fn store_system_state(account: &AccountInfo, state: &SystemState) -> Result<(), ProgramError> {
    let data = state.try_to_vec()?; // Serialize state to bytes
    let data_len = data.len();

    // Validate the serialized data length
    if data_len == 0 || data_len > account.data_len() - 8 {
        msg!("Error: Serialized data length is invalid");
        return Err(ProgramError::InvalidAccountData);
    }
    
    // Check if the account data is large enough to hold the serialized state plus the length prefix
    if account.data_len() < data_len + 8 {
        msg!("Error: Account data is too small to hold the serialized system state");
        return Err(ProgramError::AccountDataTooSmall);
    }

    let mut data_ref = account.data.borrow_mut();
    
    data_ref[..data_len].copy_from_slice(&data);
    
    // Save the length of the serialized data at the end of the account data
    let length_bytes = (data_len as u64).to_le_bytes();
    let data_ref_len = data_ref.len(); // Calculate length before mutable borrow
    data_ref[data_ref_len - 8..].copy_from_slice(&length_bytes);
    
    Ok(())
}

pub fn load_system_state(account: &AccountInfo) -> Result<SystemState, ProgramError> {
    let account_data = account.data.borrow();
    let data_len_position = account_data.len().checked_sub(8).ok_or(ProgramError::InvalidAccountData)?;
    let serialized_len_bytes = &account_data[data_len_position..];
    
    if serialized_len_bytes.iter().all(|&b| b == 0) {
        return Err(ProgramError::InvalidAccountData);
    }
    
    let serialized_len = usize::from_le_bytes(serialized_len_bytes.try_into().map_err(|_| ProgramError::InvalidAccountData)?);
    
    if serialized_len == 0 || serialized_len > data_len_position {
        return Err(ProgramError::InvalidAccountData);
    }

    let serialized_state = &account_data[..serialized_len];
    SystemState::try_from_slice(serialized_state).map_err(|_| ProgramError::InvalidAccountData)
}

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

// Helper function to ensure bounded vector resizing
fn safe_vector_resize(user_input_size: usize) -> Result<Vec<u8>, ProgramError> {
    const MAX_SIZE: usize = 1024;
    if user_input_size > MAX_SIZE {
        return Err(ProgramError::InvalidInstructionData);
    }
    Ok(vec![0; user_input_size])
}

// Safe array access to prevent index out-of-bounds
fn safe_array_access(arr: &[u8], index: usize) -> Result<u8, ProgramError> {
    arr.get(index).cloned().ok_or(ProgramError::InvalidAccountData)
}

// Ensure only authorized accounts can call this function
fn check_authorization(account: &AccountInfo, authorized_accounts: &[Pubkey]) -> Result<(), ProgramError> {
    if !authorized_accounts.contains(account.key) {
        return Err(DHelixError::Unauthorized.into());
    }
    Ok(())
}

impl DHelixToken {
    pub fn mint(accounts: &[AccountInfo], amount: u64, authorized_accounts: &[Pubkey]) -> ProgramResult {
        if accounts.len() < 3 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        
        let account_info_iter = &mut accounts.iter();
        let mint_account = next_account_info(account_info_iter)?;
        let destination_account = next_account_info(account_info_iter)?;
        let _state_account = next_account_info(account_info_iter)?;
    
        check_authorization(mint_account, authorized_accounts)?;
    
        if !mint_account.is_signer {
            msg!("Error: Mint account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }
    
        if !destination_account.is_writable {
            msg!("Error: Destination account is not writable");
            return Err(DHelixError::InvalidDestinationAccount.into());
        }
    
        let mut destination_token_account = TokenAccount::unpack_unchecked(&destination_account.data.borrow())?;
        destination_token_account.amount = destination_token_account.amount.checked_add(amount).ok_or(DHelixError::OverflowError)?;
    
        TokenAccount::pack(destination_token_account, &mut destination_account.data.borrow_mut())?;
        msg!("Minted {} tokens to {}", amount, destination_account.key);
    
        // Log event
        msg!("Event: Mint {{ amount: {}, destination: {} }}", amount, destination_account.key);
    
        Ok(())
    }

    pub fn transfer(accounts: &[AccountInfo], amount: u64, authorized_accounts: &[Pubkey]) -> ProgramResult {
        if accounts.len() < 3 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
    
        let account_info_iter = &mut accounts.iter();
        let source_account = next_account_info(account_info_iter)?;
        let destination_account = next_account_info(account_info_iter)?;
        let _state_account = next_account_info(account_info_iter)?;
    
        check_authorization(source_account, authorized_accounts)?;
    
        if !source_account.is_signer {
            msg!("Error: Source account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }
    
        if !source_account.is_writable || !destination_account.is_writable {
            msg!("Error: Source or destination account is not writable");
            return Err(DHelixError::InvalidDestinationAccount.into());
        }
    
        let mut source_token_account = TokenAccount::unpack_unchecked(&source_account.data.borrow())?;
        let mut destination_token_account = TokenAccount::unpack_unchecked(&destination_account.data.borrow())?;
    
        // Ensure source account is initialized
        if !source_token_account.is_initialized {
            msg!("Error: Source account is not initialized");
            return Err(ProgramError::UninitializedAccount);
        }
    
        // Ensure destination account is initialized
        if !destination_token_account.is_initialized {
            msg!("Error: Destination account is not initialized");
            return Err(ProgramError::UninitializedAccount);
        }
    
        if source_token_account.amount < amount {
            msg!("Error: Insufficient funds in source account");
            return Err(DHelixError::InsufficientFunds.into());
        }
    
        source_token_account.amount = source_token_account.amount.checked_sub(amount).ok_or(DHelixError::UnderflowError)?;
        destination_token_account.amount = destination_token_account.amount.checked_add(amount).ok_or(DHelixError::OverflowError)?;
    
        TokenAccount::pack(source_token_account, &mut source_account.data.borrow_mut())?;
        TokenAccount::pack(destination_token_account, &mut destination_account.data.borrow_mut())?;
        
        msg!("Transferring {} tokens from {} to {}", amount, source_account.key, destination_account.key);
    
        // Log event
        msg!("Event: Transfer {{ amount: {}, source: {}, destination: {} }}", amount, source_account.key, destination_account.key);
    
        Ok(())
    }

    pub fn burn(accounts: &[AccountInfo], amount: u64, authorized_accounts: &[Pubkey]) -> ProgramResult {
        if accounts.len() < 2 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
    
        let account_info_iter = &mut accounts.iter();
        let burn_account = next_account_info(account_info_iter)?;
        let _state_account = next_account_info(account_info_iter)?;
    
        check_authorization(burn_account, authorized_accounts)?;
    
        if !burn_account.is_signer {
            msg!("Error: Burn account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }
    
        if !burn_account.is_writable {
            msg!("Error: Burn account is not writable");
            return Err(DHelixError::InvalidDestinationAccount.into());
        }
    
        let mut burn_token_account = TokenAccount::unpack_unchecked(&burn_account.data.borrow())?;
    
        // Ensure burn account is initialized
        if !burn_token_account.is_initialized {
            msg!("Error: Burn account is not initialized");
            return Err(ProgramError::UninitializedAccount);
        }
    
        if burn_token_account.amount < amount {
            msg!("Error: Insufficient funds in burn account");
            return Err(DHelixError::InsufficientFunds.into());
        }
    
        burn_token_account.amount = burn_token_account.amount.checked_sub(amount).ok_or(DHelixError::UnderflowError)?;
    
        TokenAccount::pack(burn_token_account, &mut burn_account.data.borrow_mut())?;
        
        msg!("Burning {} tokens from {}", amount, burn_account.key);
    
        // Log event
        msg!("Event: Burn {{ amount: {}, burner: {} }}", amount, burn_account.key);
    
        Ok(())
    }

    pub fn multisig(accounts: &[AccountInfo], required_signatures: u8, authorized_accounts: &[Pubkey]) -> ProgramResult {
        if accounts.len() < 3 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        
        let mut account_info_iter = accounts.iter();
        let multisig_account = next_account_info(&mut account_info_iter)?;
        let _state_account = next_account_info(&mut account_info_iter)?;
    
        check_authorization(multisig_account, authorized_accounts)?;
    
        if !multisig_account.is_signer {
            msg!("Error: Multisig account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }
    
        if !multisig_account.is_writable {
            msg!("Error: Multisig account is not writable");
            return Err(DHelixError::InvalidDestinationAccount.into());
        }
    
        let mut signature_count = 1; // Start with 1 to count multisig_account as a signer
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

    pub fn time_lock(accounts: &[AccountInfo], unlock_time: u64, authorized_accounts: &[Pubkey]) -> ProgramResult {
        if accounts.len() < 3 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
    
        let account_info_iter = &mut accounts.iter();
        let time_lock_account = next_account_info(account_info_iter)?;
        let clock_account = next_account_info(account_info_iter)?;
        let _state_account = next_account_info(account_info_iter)?;
    
        check_authorization(time_lock_account, authorized_accounts)?;
    
        if !time_lock_account.is_signer {
            msg!("Error: Time-lock account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }
    
        let clock = Clock::from_account_info(clock_account)?;
        let current_time = clock.unix_timestamp as u64;
        if current_time < unlock_time {
            return Err(DHelixError::AccountLocked.into());
        }
    
        msg!("Time-lock operation until {}", unlock_time);
        // Log event
        msg!("Event: TimeLock {{ account: {}, unlock_time: {} }}", time_lock_account.key, unlock_time);
    
        Ok(())
    }

    pub fn emergency_stop(accounts: &[AccountInfo], authorized_accounts: &[Pubkey]) -> ProgramResult {
        if accounts.len() < 2 {
            msg!("Error: Not enough account keys");
            return Err(ProgramError::NotEnoughAccountKeys);
        }
    
        let account_info_iter = &mut accounts.iter();
        let emergency_stop_account = next_account_info(account_info_iter)?;
        let system_state_account = next_account_info(account_info_iter)?;
    
        check_authorization(emergency_stop_account, authorized_accounts)?;
    
        if !emergency_stop_account.is_signer {
            msg!("Error: Emergency stop account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }
    
        match load_system_state(system_state_account) {
            Ok(mut state) => {
                state.halt = true;
                match store_system_state(system_state_account, &state) {
                    Ok(_) => {
                        msg!("Emergency stop operation successful");
                        // Log event
                        msg!("Event: EmergencyStop {{ account: {} }}", emergency_stop_account.key);
                        Ok(())
                    },
                    Err(e) => {
                        msg!("Error storing system state: {:?}", e);
                        Err(e)
                    }
                }
            },
            Err(e) => {
                msg!("Error loading system state: {:?}", e);
                Err(e)
            }
        }
    }
}

pub struct DHelixDAO;

impl DHelixDAO {
    pub fn submit_proposal(accounts: &[AccountInfo], proposal_id: u64, proposal_data: &[u8]) -> ProgramResult {
        if accounts.len() < 2 {
            msg!("Error: Not enough accounts");
            return Err(ProgramError::NotEnoughAccountKeys);
        }
    
        let account_info_iter = &mut accounts.iter();
        let proposer_account = next_account_info(account_info_iter)?;
        let proposals_state_account = next_account_info(account_info_iter)?;
    
        if !proposer_account.is_signer {
            msg!("Error: Proposer account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        if proposal_data.len() > 1024 {
            return Err(ProgramError::InvalidInstructionData);
        }
    
        msg!("Loading proposals state...");
        let mut state = match load_proposals_state(proposals_state_account) {
            Ok(state) => state,
            Err(e) => {
                msg!("Error loading proposals state: {:?}", e);
                return Err(e);
            }
        };
    
        if state.proposals.contains_key(&proposal_id) {
            msg!("Error: Proposal ID already exists");
            return Err(ProgramError::InvalidArgument);
        }
    
        msg!("Inserting proposal ID: {}", proposal_id);
        state.proposals.insert(proposal_id, proposal_data.to_vec());
    
        msg!("Storing proposals state...");
        match store_proposals_state(proposals_state_account, &state) {
            Ok(_) => msg!("Proposals state stored successfully"),
            Err(e) => {
                msg!("Error storing proposals state: {:?}", e);
                return Err(e);
            }
        };
    
        msg!("Submitting proposal ID: {} by {}", proposal_id, proposer_account.key);
    
        // Log event
        msg!("Event: ProposalSubmitted {{ proposal_id: {}, proposer: {} }}", proposal_id, proposer_account.key);
    
        Ok(())
    }

    pub fn vote(accounts: &[AccountInfo], proposal_id: u64, vote: bool) -> ProgramResult {
        if accounts.len() < 2 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let account_info_iter = &mut accounts.iter();
        let voter_account = next_account_info(account_info_iter)?;
        let votes_state_account = next_account_info(account_info_iter)?;

        if !voter_account.is_signer {
            msg!("Error: Voter account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut state = load_votes_state(votes_state_account)?;
        state.votes.entry(proposal_id).or_default().push((*voter_account.key, vote));
        store_votes_state(votes_state_account, &state)?;

        msg!("Voting on proposal ID: {} by {}", proposal_id, voter_account.key);

        // Log event
        msg!("Event: Vote {{ proposal_id: {}, voter: {}, vote: {} }}", proposal_id, voter_account.key, vote);

        Ok(())
    }

    pub fn execute_proposal(accounts: &[AccountInfo], proposal_id: u64) -> ProgramResult {
        if accounts.len() < 4 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
    
        let account_info_iter = &mut accounts.iter();
        let executor_account = next_account_info(account_info_iter)?;
        let proposals_state_account = next_account_info(account_info_iter)?;
        let token_account = next_account_info(account_info_iter)?; // Account to perform actions on
        let state_account = next_account_info(account_info_iter)?; // For storing states
    
        if !executor_account.is_signer {
            msg!("Error: Executor account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }
    
        let mut state = load_proposals_state(proposals_state_account)?;
        if let Some(data) = state.proposals.get(&proposal_id) {
            // Deserialize proposal data
            let action: u8 = data[0]; // First byte denotes the action type
            match action {
                0 => { // Mint tokens
                    let amount: u64 = u64::from_le_bytes(data[1..9].try_into().unwrap());
                    let mut token_account_state = TokenAccount::unpack(&token_account.data.borrow())?;
                    token_account_state.amount = token_account_state.amount.checked_add(amount).ok_or(DHelixError::OverflowError)?;
                    TokenAccount::pack(token_account_state, &mut token_account.data.borrow_mut())?;
                    msg!("Minted {} tokens to {}", amount, token_account.key);
                }
                1 => { // Transfer tokens
                    let amount: u64 = u64::from_le_bytes(data[1..9].try_into().unwrap());
                    let source_key = Pubkey::new(array_ref![data, 9, 32]);
                    let destination_key = Pubkey::new(array_ref![data, 41, 32]);
    
                    // Fetch the source and destination accounts based on the provided keys
                    let source_account = accounts.iter().find(|acc| acc.key == &source_key).ok_or(ProgramError::InvalidAccountData)?;
                    let destination_account = accounts.iter().find(|acc| acc.key == &destination_key).ok_or(ProgramError::InvalidAccountData)?;
    
                    let mut source_account_state = TokenAccount::unpack(&source_account.data.borrow())?;
                    let mut destination_account_state = TokenAccount::unpack(&destination_account.data.borrow())?;
    
                    if source_account_state.amount < amount {
                        return Err(DHelixError::InsufficientFunds.into());
                    }
    
                    source_account_state.amount = source_account_state.amount.checked_sub(amount).ok_or(DHelixError::UnderflowError)?;
                    destination_account_state.amount = destination_account_state.amount.checked_add(amount).ok_or(DHelixError::OverflowError)?;
    
                    TokenAccount::pack(source_account_state, &mut source_account.data.borrow_mut())?;
                    TokenAccount::pack(destination_account_state, &mut destination_account.data.borrow_mut())?;
    
                    msg!("Transferred {} tokens from {} to {}", amount, source_key, destination_key);
                }
                _ => {
                    return Err(ProgramError::InvalidInstructionData);
                }
            }
            // Remove the proposal from state after execution
            state.proposals.remove(&proposal_id);
            store_proposals_state(proposals_state_account, &state)?;
    
            // Log event
            msg!("Event: ProposalExecuted {{ proposal_id: {}, executor: {} }}", proposal_id, executor_account.key);
    
            Ok(())
        } else {
            Err(ProgramError::InvalidInstructionData)
        }
    }
    
    pub fn charity_vote(accounts: &[AccountInfo], proposal_id: u64, vote: bool) -> ProgramResult {
        if accounts.len() < 2 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
    
        let account_info_iter = &mut accounts.iter();
        let voter_account = next_account_info(account_info_iter)?;
        let votes_state_account = next_account_info(account_info_iter)?;
    
        if !voter_account.is_signer {
            msg!("Error: Voter account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }
    
        let mut state = load_votes_state(votes_state_account)?;
        state.votes.entry(proposal_id).or_default().push((*voter_account.key, vote));
        store_votes_state(votes_state_account, &state)?;
    
        msg!("Charity vote on proposal ID: {} by {}", proposal_id, voter_account.key);
    
        // Log event
        msg!("Event: CharityVote {{ proposal_id: {}, voter: {}, vote: {} }}", proposal_id, voter_account.key, vote);
    
        Ok(())
    }

    pub fn future_project_vote(accounts: &[AccountInfo], proposal_id: u64, vote: bool) -> ProgramResult {
        if accounts.len() < 2 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let account_info_iter = &mut accounts.iter();
        let voter_account = next_account_info(account_info_iter)?;
        let votes_state_account = next_account_info(account_info_iter)?;

        if !voter_account.is_signer {
            msg!("Error: Voter account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut state = load_votes_state(votes_state_account)?;
        state.votes.entry(proposal_id).or_default().push((*voter_account.key, vote));
        store_votes_state(votes_state_account, &state)?;

        msg!("Future project vote on proposal ID: {} by {}", proposal_id, voter_account.key);

        // Log event
        msg!("Event: FutureProjectVote {{ proposal_id: {}, voter: {}, vote: {} }}", proposal_id, voter_account.key, vote);

        Ok(())
    }
}

impl DHelixToken {
    pub fn incentivized_voting_system(accounts: &[AccountInfo], proposal_id: u64, vote: bool) -> ProgramResult {
        if accounts.len() < 3 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
    
        let account_info_iter = &mut accounts.iter();
        let voter_account = next_account_info(account_info_iter)?;
        let votes_state_account = next_account_info(account_info_iter)?;
        let balances_state_account = next_account_info(account_info_iter)?;
    
        if !voter_account.is_signer {
            msg!("Error: Voter account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }
    
        let mut votes_state = load_votes_state(votes_state_account)?;
        votes_state.votes.entry(proposal_id).or_default().push((*voter_account.key, vote));
        store_votes_state(votes_state_account, &votes_state)?;
    
        let mut balances_state = load_balances_state(balances_state_account)?;
        let reward_amount = 10; // Example reward amount
        let balance = balances_state.balances.entry(*voter_account.key).or_insert(0);
        *balance += reward_amount;
        store_balances_state(balances_state_account, &balances_state)?;
    
        msg!("Incentivized voting on proposal ID: {} by {}", proposal_id, voter_account.key);
    
        // Log event
        msg!("Event: IncentivizedVote {{ proposal_id: {}, voter: {}, vote: {} }}", proposal_id, voter_account.key, vote);
    
        Ok(())
    }

    pub fn dynamic_staking_rewards(accounts: &[AccountInfo], staking_duration: u64) -> ProgramResult {
        if accounts.len() < 2 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let account_info_iter = &mut accounts.iter();
        let staker_account = next_account_info(account_info_iter)?;
        let balances_state_account = next_account_info(account_info_iter)?;

        if !staker_account.is_signer {
            msg!("Error: Staker account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut balances_state = load_balances_state(balances_state_account)?;
        let reward_rate = 5; // Example reward rate per duration unit
        let reward_amount = staking_duration * reward_rate;
        let balance = balances_state.balances.entry(*staker_account.key).or_insert(0);
        *balance += reward_amount;
        store_balances_state(balances_state_account, &balances_state)?;

        msg!("Calculating staking rewards for {} by staking duration {}", staker_account.key, staking_duration);

        // Log event
        msg!("Event: StakingRewards {{ staker: {}, staking_duration: {} }}", staker_account.key, staking_duration);

        Ok(())
    }

    pub fn token_buyback_program(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        if accounts.len() < 2 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let account_info_iter = &mut accounts.iter();
        let buyback_account = next_account_info(account_info_iter)?;
        let balances_state_account = next_account_info(account_info_iter)?;

        if !buyback_account.is_signer {
            msg!("Error: Buyback account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut balances_state = load_balances_state(balances_state_account)?;
        let balance = balances_state.balances.entry(*buyback_account.key).or_insert(0);
        if *balance < amount {
            return Err(ProgramError::InsufficientFunds);
        }
        *balance -= amount;
        store_balances_state(balances_state_account, &balances_state)?;

        msg!("Executing token buyback for {} tokens", amount);

        // Log event
        msg!("Event: Buyback {{ amount: {}, buyer: {} }}", amount, buyback_account.key);

        Ok(())
    }

    pub fn insurance_pool(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        if accounts.len() < 3 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let account_info_iter = &mut accounts.iter();
        let insurance_account = next_account_info(account_info_iter)?;
        let system_state_account = next_account_info(account_info_iter)?;
        let balances_state_account = next_account_info(account_info_iter)?;

        if !insurance_account.is_signer {
            msg!("Error: Insurance account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut balances_state = load_balances_state(balances_state_account)?;
        let balance = balances_state.balances.entry(*insurance_account.key).or_insert(0);
        if *balance < amount {
            return Err(ProgramError::InsufficientFunds);
        }
        *balance -= amount;
        store_balances_state(balances_state_account, &balances_state)?;

        let mut system_state = load_system_state(system_state_account)?;
        system_state.insurance_pool += amount;
        store_system_state(system_state_account, &system_state)?;

        msg!("Contributing {} to the insurance pool", amount);

        // Log event
        msg!("Event: InsuranceContribution {{ amount: {}, contributor: {} }}", amount, insurance_account.key);

        Ok(())
    }
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

    // Example authorized accounts (should be configured as needed)
    let authorized_accounts = vec![
        Pubkey::from_str("AxGavuYn6HHY95AjPyTaZHEpeKAgRJq4gAPJriC3iYP5").unwrap(),
        Pubkey::from_str("GSqP2u5zXbESXXxmLzJAs9cXpkbCSejyy5RSJsWVEADZ").unwrap(),
    ];

    match instruction {
        0 => {
            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            DHelixToken::mint(accounts, amount, &authorized_accounts)
        },
        1 => {
            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            DHelixToken::transfer(accounts, amount, &authorized_accounts)
        },
        2 => {
            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            DHelixToken::burn(accounts, amount, &authorized_accounts)
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
        6 => {
            let required_signatures = instruction_data[1];
            DHelixToken::multisig(accounts, required_signatures, &authorized_accounts)
        },
        7 => {
            let unlock_time = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            DHelixToken::time_lock(accounts, unlock_time, &authorized_accounts)
        },
        8 => {
            DHelixToken::emergency_stop(accounts, &authorized_accounts)
        },
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
        _ => {
            msg!("Unknown instruction");
            Err(ProgramError::InvalidInstructionData)
        },
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

    fn create_account_info_with_clock<'a>(
        key: &'a Pubkey,
        is_signer: bool,
        is_writable: bool,
        lamports: &'a mut u64,
        data: &'a mut [u8],
        owner: &'a Pubkey,
        clock_lamports: &'a mut u64,
        clock_data: &'a mut [u8],
        sysvar_id: &'a Pubkey,
        clock_id: &'a Pubkey,
    ) -> (AccountInfo<'a>, AccountInfo<'a>) {
        let account_info = create_account_info(key, is_signer, is_writable, lamports, data, owner);

        let clock_account_info = AccountInfo {
            key: clock_id,
            is_signer: false,
            is_writable: false,
            lamports: Rc::new(RefCell::new(clock_lamports)),
            data: Rc::new(RefCell::new(clock_data)),
            owner: sysvar_id,
            executable: false,
            rent_epoch: Epoch::default(),
        };

        (account_info, clock_account_info)
    }

    fn initialize_state_account<'a>(
        key: &'a Pubkey,
        lamports: &'a mut u64,
        data: &'a mut Vec<u8>,
        owner: &'a Pubkey,
    ) -> AccountInfo<'a> {
        // Initialize account data with a serialized empty state
        let state = ProposalsState::default();
        let serialized_state = state.try_to_vec().unwrap();
        let serialized_state_len = serialized_state.len();
        data[..serialized_state_len].copy_from_slice(&serialized_state);
        // Save the length of the serialized data at the end of the account data
        let length_bytes = (serialized_state_len as u64).to_le_bytes();
        let data_len = data.len();
        data[data_len - 8..].copy_from_slice(&length_bytes);

        AccountInfo::new(key, false, true, lamports, data, owner, false, 0)
    }

    #[test]
    fn test_store_proposals_state_data_too_small() {
        let key = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let mut lamports = 0;
        let mut data = vec![0; 10]; // Intentionally too small
        let account = AccountInfo::new(&key, false, true, &mut lamports, &mut data, &owner, false, 0);
        let state = ProposalsState::default();

        let result = store_proposals_state(&account, &state);
        assert_eq!(result, Err(ProgramError::AccountDataTooSmall));
    }

    #[test]
    fn test_load_proposals_state_invalid_data() {
        let key = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let mut lamports = 0;
        let mut data = vec![0; 1024];
        let account = AccountInfo::new(&key, false, true, &mut lamports, &mut data, &owner, false, 0);

        let result = load_proposals_state(&account);
        assert_eq!(result, Err(ProgramError::InvalidAccountData));
    }

    #[test]
    fn test_safe_vector_resize_exceed_max_size() {
        let result = safe_vector_resize(2048); // Exceeding the max size
        assert_eq!(result, Err(ProgramError::InvalidInstructionData));
    }

    #[test]
    fn test_safe_array_access_out_of_bounds() {
        let array = vec![1, 2, 3];
        let result = safe_array_access(&array, 5); // Out of bounds access
        assert_eq!(result, Err(ProgramError::InvalidAccountData));
    }

    #[test]
    fn test_check_authorization_unauthorized() {
        let key = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let mut lamports = 0;
        let mut data = vec![0; 1024];
        let account = AccountInfo::new(&key, true, true, &mut lamports, &mut data, &owner, false, 0);
        let authorized_accounts = vec![Pubkey::new_unique()]; // Different account

        let result = check_authorization(&account, &authorized_accounts);
        assert_eq!(result, Err(DHelixError::Unauthorized.into()));
    }

    #[test]
    fn test_mint_invalid_account() {
        let program_id = Pubkey::new_unique();
        let mut mint_account_lamports = 500;
        let mut destination_account_lamports = 100;
        let mut state_account_lamports = 100;
        let mut mint_account_data = vec![0; TokenAccount::LEN];
        let mut destination_account_data = vec![0; TokenAccount::LEN];
        let mut state_account_data = vec![0; 1024]; // Adjust size as necessary
        let mint_key = Pubkey::new_unique(); // Unauthorized key
        let destination_key = Pubkey::new_unique();
        let state_key = Pubkey::new_unique();

        let mint_account = create_account_info(&mint_key, true, true, &mut mint_account_lamports, &mut mint_account_data, &program_id);
        let destination_account = create_account_info(&destination_key, false, true, &mut destination_account_lamports, &mut destination_account_data, &program_id);
        let state_account = initialize_state_account(&state_key, &mut state_account_lamports, &mut state_account_data, &program_id);
        let accounts = vec![mint_account, destination_account.clone(), state_account.clone()];

        // Test minting with unauthorized account
        let amount = 100;
        let result = DHelixToken::mint(&accounts, amount, &vec![]);
        assert!(result.is_err(), "Minting succeeded with an unauthorized account");
    }

    #[test]
    fn test_transfer_to_non_initialized_account() {
        let program_id = Pubkey::new_unique();
        let mut source_account_lamports = 700;
        let mut destination_account_lamports = 100;
        let mut state_account_lamports = 100;
        let mut source_account_data = vec![0; TokenAccount::LEN];
        let mut destination_account_data = vec![0; TokenAccount::LEN];
        let mut state_account_data = vec![0; 1024]; // Adjust size as necessary
        let source_key = Pubkey::new_unique();
        let destination_key = Pubkey::new_unique();
        let state_key = Pubkey::new_unique();

        let source_account = create_account_info(&source_key, true, true, &mut source_account_lamports, &mut source_account_data, &program_id);
        let destination_account = create_account_info(&destination_key, false, true, &mut destination_account_lamports, &mut destination_account_data, &program_id);
        let state_account = initialize_state_account(&state_key, &mut state_account_lamports, &mut state_account_data, &program_id);
        let accounts = vec![source_account, destination_account.clone(), state_account.clone()];

        // Initialize source account as a TokenAccount
        let src_token_account = TokenAccount {
            is_initialized: true,
            owner: source_key,
            amount: 700,
        };
        TokenAccount::pack(src_token_account.clone(), &mut accounts[0].data.borrow_mut()).unwrap();

        // Ensure destination account is not initialized
        let dest_token_account = TokenAccount {
            is_initialized: false,
            owner: destination_key,
            amount: 0,
        };
        TokenAccount::pack(dest_token_account.clone(), &mut accounts[1].data.borrow_mut()).unwrap();

        // Test transfer to non-initialized destination account
        let amount = 200;
        let result = DHelixToken::transfer(&accounts, amount, &vec![source_key]);
        assert!(result.is_err(), "Transfer succeeded to a non-initialized account");
    }

    #[test]
    fn test_burn_from_non_initialized_account() {
        let program_id = Pubkey::new_unique();
        let burn_authority_pubkey = Pubkey::from_str("AxGavuYn6HHY95AjPyTaZHEpeKAgRJq4gAPJriC3iYP5").unwrap();
        let mut burn_account_lamports = 500;
        let mut state_account_lamports = 100;
        let mut burn_account_data = vec![0; TokenAccount::LEN];
        let mut state_account_data = vec![0; 1024]; // Adjust size as necessary
        let burn_key = burn_authority_pubkey; // Set the burn account key to the authorized pubkey
        let state_key = Pubkey::new_unique();

        let burn_account = create_account_info(&burn_key, true, true, &mut burn_account_lamports, &mut burn_account_data, &program_id);
        let state_account = initialize_state_account(&state_key, &mut state_account_lamports, &mut state_account_data, &program_id);
        let accounts = vec![burn_account.clone(), state_account.clone()];

        // Test burn from non-initialized burn account
        let amount = 200;
        let result = DHelixToken::burn(&accounts, amount, &vec![burn_key]);
        assert!(result.is_err(), "Burn succeeded from a non-initialized account");
    }

    #[test]
    fn test_multisig_invalid_signers() {
        let program_id = Pubkey::new_unique();
        let mut multisig_account_lamports = 300;
        let mut signer1_lamports = 300;
        let mut state_account_lamports = 100;
        let mut multisig_account_data = vec![0; 100];
        let mut signer1_account_data = vec![0; 100];
        let mut state_account_data = vec![0; 1024]; // Adjust size as necessary
        let multisig_key = Pubkey::new_unique();
        let signer1_key = Pubkey::new_unique();
        let state_key = Pubkey::new_unique();

        let multisig_account = create_account_info(
            &multisig_key, true, true, &mut multisig_account_lamports, &mut multisig_account_data, &program_id);
        let signer1_account = create_account_info(
            &signer1_key, false, false, &mut signer1_lamports, &mut signer1_account_data, &program_id); // Not a signer
        let state_account = initialize_state_account(
            &state_key, &mut state_account_lamports, &mut state_account_data, &program_id);

        // Test invalid multisig with non-signer accounts
        let accounts = vec![multisig_account.clone(), signer1_account.clone(), state_account.clone()];
        let required_signatures = 2;
        let result = DHelixToken::multisig(&accounts, required_signatures, &vec![multisig_key]);
        assert!(result.is_err(), "Multisig succeeded with invalid signer accounts");
    }

    #[test]
    fn test_time_lock_expired() {
        let program_id = Pubkey::new_unique();
        let mut time_lock_account_lamports = 300;
        let mut clock_lamports = 0;
        let mut state_account_lamports = 100;
        let mut time_lock_account_data = vec![0; 100];
        let mut clock_data = vec![0; Clock::size_of()];
        let mut state_account_data = vec![0; 1024]; // Adjust size as necessary
        let time_lock_key = Pubkey::new_unique();
        let clock_key = Clock::id();
        let state_key = Pubkey::new_unique();

        let sysvar_id = sysvar::id();

        // Manually initialize clock data
        let clock = Clock {
            slot: 0,
            epoch_start_timestamp: 0,
            epoch: 0,
            leader_schedule_epoch: 0,
            unix_timestamp: 1_622_360_800, // Set a specific timestamp for testing
        };
        let clock_bytes = clock_data.as_mut_slice();
        clock_bytes[..8].copy_from_slice(&clock.slot.to_le_bytes());
        clock_bytes[8..16].copy_from_slice(&clock.epoch_start_timestamp.to_le_bytes());
        clock_bytes[16..24].copy_from_slice(&clock.epoch.to_le_bytes());
        clock_bytes[24..32].copy_from_slice(&clock.leader_schedule_epoch.to_le_bytes());
        clock_bytes[32..40].copy_from_slice(&clock.unix_timestamp.to_le_bytes());

        let (time_lock_account, clock_account) = create_account_info_with_clock(
            &time_lock_key,
            true, true, &mut time_lock_account_lamports, &mut time_lock_account_data, &program_id,
            &mut clock_lamports, &mut clock_data, &sysvar_id, &clock_key,
        );

        let state_account = initialize_state_account(&state_key, &mut state_account_lamports, &mut state_account_data, &program_id);
        let accounts = vec![time_lock_account.clone(), clock_account.clone(), state_account];

        let unlock_time = clock.unix_timestamp as u64 + 1000; // Setting unlock time to future
        let result = DHelixToken::time_lock(&accounts, unlock_time, &vec![time_lock_key]);
        assert!(result.is_err(), "Time-lock succeeded with an unlock time in the future");
    }

    #[test]
    fn test_emergency_stop_not_signer() {
        let program_id = Pubkey::new_unique();
        let mut emergency_stop_account_lamports = 300;
        let mut state_account_lamports = 100;
        let mut emergency_stop_account_data = vec![0; 100];
        let mut system_state_account_data = vec![0; 1032]; // Ensure this size is enough for the serialized state + length
        let emergency_stop_key = Pubkey::new_unique();
        let system_state_key = Pubkey::new_unique();

        let emergency_stop_account = create_account_info(
            &emergency_stop_key, false, true, &mut emergency_stop_account_lamports, &mut emergency_stop_account_data, &program_id); // Not a signer

        // Initialize system_state_account with a default SystemState
        let system_state = SystemState {
            halt: false,
            insurance_pool: 0,
        };
        let serialized_state = system_state.try_to_vec().unwrap();
        let serialized_state_len = serialized_state.len();
        system_state_account_data[..serialized_state_len].copy_from_slice(&serialized_state);

        let data_len = system_state_account_data.len(); // Store length in a local variable
        let length_bytes = (serialized_state_len as u64).to_le_bytes();
        system_state_account_data[data_len - 8..].copy_from_slice(&length_bytes);

        let system_state_account = AccountInfo::new(
            &system_state_key, false, true, &mut state_account_lamports, &mut system_state_account_data, &program_id, false, 0);

        let accounts = vec![emergency_stop_account.clone(), system_state_account];

        let result = DHelixToken::emergency_stop(&accounts, &vec![emergency_stop_key]);
        assert!(result.is_err(), "Emergency stop succeeded with a non-signer account");
    }

    #[test]
    fn test_submit_proposal_large_data() {
        let program_id = Pubkey::new_unique();
        let mut proposer_account_lamports = 300;
        let mut state_account_lamports = 100;
        let mut proposer_account_data = vec![0; 100];
        let mut proposals_state_account_data = vec![0; 1032]; // Adjust size to include space for length (1024 + 8 bytes)
        let proposer_key = Pubkey::new_unique();
        let proposals_state_key = Pubkey::new_unique();

        let proposer_account = create_account_info(&proposer_key, true, true, &mut proposer_account_lamports, &mut proposer_account_data, &program_id);
        let proposals_state_account = initialize_state_account(&proposals_state_key, &mut state_account_lamports, &mut proposals_state_account_data, &program_id);
        let accounts = vec![proposer_account.clone(), proposals_state_account.clone()];

        // Create large proposal data
        let proposal_id = 1;
        let proposal_data = vec![0; 2048];

        let result = DHelixDAO::submit_proposal(&accounts, proposal_id, &proposal_data);
        assert!(result.is_err(), "Submit proposal succeeded with large data");
    }

    #[test]
    fn test_vote_non_signer() {
        let program_id = Pubkey::new_unique();
        let mut voter_account_lamports = 300;
        let mut state_account_lamports = 100;
        let mut voter_account_data = vec![0; 100];
        let mut votes_state_account_data = vec![0; 1024]; // Adjust size as necessary
        let voter_key = Pubkey::new_unique();
        let votes_state_key = Pubkey::new_unique();

        let voter_account = create_account_info(&voter_key, false, true, &mut voter_account_lamports, &mut voter_account_data, &program_id); // Not a signer
        let votes_state_account = initialize_state_account(&votes_state_key, &mut state_account_lamports, &mut votes_state_account_data, &program_id);
        let accounts = vec![voter_account.clone(), votes_state_account.clone()];

        let proposal_id = 1;
        let vote = true;

        let result = DHelixDAO::vote(&accounts, proposal_id, vote);
        assert!(result.is_err(), "Vote succeeded with a non-signer account");
    }

    #[test]
    fn test_execute_proposal_non_existent() {
        let program_id = Pubkey::new_unique();
        let mut executor_account_lamports = 300;
        let mut state_account_lamports = 100;
        let mut executor_account_data = vec![0; 100];
        let mut proposals_state_account_data = vec![0; 1032]; // Adjust size as necessary
        let executor_key = Pubkey::new_unique();
        let proposals_state_key = Pubkey::new_unique();

        let executor_account = create_account_info(&executor_key, true, true, &mut executor_account_lamports, &mut executor_account_data, &program_id);
        let proposals_state_account = initialize_state_account(&proposals_state_key, &mut state_account_lamports, &mut proposals_state_account_data, &program_id);
        let accounts = vec![executor_account.clone(), proposals_state_account.clone()];

        let proposal_id = 1;

        let result = DHelixDAO::execute_proposal(&accounts, proposal_id);
        assert!(result.is_err(), "Execute proposal succeeded with a non-existent proposal ID");
    }

    #[test]
    fn test_mint() {
        let program_id = Pubkey::new_unique();
        let mint_authority_pubkey = Pubkey::from_str("GSqP2u5zXbESXXxmLzJAs9cXpkbCSejyy5RSJsWVEADZ").unwrap();
        let mut mint_account_lamports = 500;
        let mut destination_account_lamports = 100;
        let mut state_account_lamports = 100;
        let mut mint_account_data = vec![0; TokenAccount::LEN];
        let mut destination_account_data = vec![0; TokenAccount::LEN];
        let mut state_account_data = vec![0; 1024]; // Adjust size as necessary
        let mint_key = mint_authority_pubkey; // Set the mint account key to the authorized pubkey
        let destination_key = Pubkey::new_unique();
        let state_key = Pubkey::new_unique();

        let mint_account = create_account_info(&mint_key, true, true, &mut mint_account_lamports, &mut mint_account_data, &program_id);
        let destination_account = create_account_info(&destination_key, false, true, &mut destination_account_lamports, &mut destination_account_data, &program_id);
        let state_account = initialize_state_account(&state_key, &mut state_account_lamports, &mut state_account_data, &program_id);
        let accounts = vec![mint_account, destination_account.clone(), state_account.clone()];

        // Initialize destination account as a TokenAccount
        let mut dest_token_account = TokenAccount {
            is_initialized: true,
            owner: destination_key,
            amount: 100,
        };
        TokenAccount::pack(dest_token_account.clone(), &mut destination_account.data.borrow_mut()).unwrap();

        // Test valid minting
        let amount = 100;
        let result = DHelixToken::mint(&accounts, amount, &vec![mint_key]);
        assert!(result.is_ok());
        dest_token_account.amount += amount;
        assert_eq!(TokenAccount::unpack(&destination_account.data.borrow()).unwrap(), dest_token_account);

        // Test overflow
        let result = DHelixToken::mint(&accounts, u64::MAX, &vec![mint_key]);
        assert!(result.is_err());

        // Test without signer
        let result = DHelixToken::mint(&[destination_account.clone(), state_account.clone()], amount, &vec![destination_key]);
        assert!(result.is_err());
    }

    #[test]
    fn test_transfer() {
        let program_id = Pubkey::new_unique();
        let mut source_account_lamports = 700;
        let mut destination_account_lamports = 100;
        let mut state_account_lamports = 100;
        let mut source_account_data = vec![0; TokenAccount::LEN];
        let mut destination_account_data = vec![0; TokenAccount::LEN];
        let mut state_account_data = vec![0; 1024]; // Adjust size as necessary
        let source_key = Pubkey::new_unique();
        let destination_key = Pubkey::new_unique();
        let state_key = Pubkey::new_unique();

        let source_account = create_account_info(&source_key, true, true, &mut source_account_lamports, &mut source_account_data, &program_id);
        let destination_account = create_account_info(&destination_key, false, true, &mut destination_account_lamports, &mut destination_account_data, &program_id);
        let state_account = initialize_state_account(&state_key, &mut state_account_lamports, &mut state_account_data, &program_id);
        let accounts = vec![source_account, destination_account.clone(), state_account.clone()];

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
        let result = DHelixToken::transfer(&accounts, amount, &vec![source_key]);
        assert!(result.is_ok());
        src_token_account.amount -= amount;
        dest_token_account.amount += amount;
        assert_eq!(TokenAccount::unpack(&accounts[0].data.borrow()).unwrap(), src_token_account);
        assert_eq!(TokenAccount::unpack(&accounts[1].data.borrow()).unwrap(), dest_token_account);

        // Test underflow
        let result = DHelixToken::transfer(&accounts, 1000, &vec![source_key]);
        assert!(result.is_err());

        // Test without signer
        let result = DHelixToken::transfer(&[accounts[1].clone(), accounts[1].clone(), accounts[2].clone()], amount, &vec![destination_key]);
        assert!(result.is_err());
    }

    #[test]
    fn test_burn() {
        let program_id = Pubkey::new_unique();
        let burn_authority_pubkey = Pubkey::from_str("AxGavuYn6HHY95AjPyTaZHEpeKAgRJq4gAPJriC3iYP5").unwrap();
        let mut burn_account_lamports = 500;
        let mut state_account_lamports = 100;
        let mut burn_account_data = vec![0; TokenAccount::LEN];
        let mut state_account_data = vec![0; 1024]; // Adjust size as necessary
        let burn_key = burn_authority_pubkey; // Set the burn account key to the authorized pubkey
        let state_key = Pubkey::new_unique();

        let burn_account = create_account_info(&burn_key, true, true, &mut burn_account_lamports, &mut burn_account_data, &program_id);
        let state_account = initialize_state_account(&state_key, &mut state_account_lamports, &mut state_account_data, &program_id);
        let accounts = vec![burn_account.clone(), state_account.clone()];

        // Initialize burn account as a TokenAccount
        let mut burn_token_account = TokenAccount {
            is_initialized: true,
            owner: burn_key,
            amount: 500,
        };
        TokenAccount::pack(burn_token_account.clone(), &mut accounts[0].data.borrow_mut()).unwrap();

        // Test valid burn
        let amount = 200;
        let result = DHelixToken::burn(&accounts, amount, &vec![burn_key]);
        assert!(result.is_ok());
        burn_token_account.amount -= amount;
        assert_eq!(TokenAccount::unpack(&accounts[0].data.borrow()).unwrap(), burn_token_account);

        // Test underflow
        let result = DHelixToken::burn(&accounts, 1000, &vec![burn_key]);
        assert!(result.is_err());

        // Test without signer
        let result = DHelixToken::burn(&[accounts[1].clone(), accounts[1].clone()], amount, &vec![state_key]);
        assert!(result.is_err());
    }

    #[test]
    fn test_multisig() {
        let program_id = Pubkey::new_unique();
        let mut multisig_account_lamports = 300;
        let mut signer1_lamports = 300;
        let mut signer2_lamports = 300;
        let mut state_account_lamports = 100;
        let mut multisig_account_data = vec![0; 100];
        let mut signer1_account_data = vec![0; 100];
        let mut signer2_account_data = vec![0; 100];
        let mut state_account_data = vec![0; 1024]; // Adjust size as necessary
        let multisig_key = Pubkey::new_unique();
        let signer1_key = Pubkey::new_unique();
        let signer2_key = Pubkey::new_unique();
        let state_key = Pubkey::new_unique();

        let multisig_account = create_account_info(
            &multisig_key, true, true, &mut multisig_account_lamports, &mut multisig_account_data, &program_id);
        let signer1_account = create_account_info(
            &signer1_key, true, false, &mut signer1_lamports, &mut signer1_account_data, &program_id);
        let signer2_account = create_account_info(
            &signer2_key, true, false, &mut signer2_lamports, &mut signer2_account_data, &program_id);
        let state_account = initialize_state_account(
            &state_key, &mut state_account_lamports, &mut state_account_data, &program_id);

        // Test valid multisig with 1 required signature
        let accounts = vec![multisig_account.clone(), signer1_account.clone(), state_account.clone()];
        let required_signatures = 1;
        let result = DHelixToken::multisig(&accounts, required_signatures, &vec![multisig_key]);
        assert!(result.is_ok(), "Multisig failed with 1 required signature");

        // Test valid multisig with 2 required signatures
        let accounts = vec![multisig_account.clone(), signer1_account.clone(), signer2_account.clone(), state_account.clone()];
        let required_signatures = 2;
        let result = DHelixToken::multisig(&accounts, required_signatures, &vec![multisig_key]);
        assert!(result.is_ok(), "Multisig failed with 2 required signatures");

        // Test not enough signers
        let accounts = vec![multisig_account.clone(), signer1_account.clone(), state_account.clone()];
        let required_signatures = 2;
        let result = DHelixToken::multisig(&accounts, required_signatures, &vec![multisig_key]);
        assert!(result.is_err(), "Multisig succeeded with not enough signers");
    }

    #[test]
    fn test_time_lock() {
        let program_id = Pubkey::new_unique();
        let mut time_lock_account_lamports = 300;
        let mut clock_lamports = 0;
        let mut state_account_lamports = 100;
        let mut time_lock_account_data = vec![0; 100];
        let mut clock_data = vec![0; Clock::size_of()];
        let mut state_account_data = vec![0; 1024]; // Adjust size as necessary
        let time_lock_key = Pubkey::new_unique();
        let clock_key = Clock::id();
        let state_key = Pubkey::new_unique();

        let sysvar_id = sysvar::id();

        // Manually initialize clock data
        let clock = Clock {
            slot: 0,
            epoch_start_timestamp: 0,
            epoch: 0,
            leader_schedule_epoch: 0,
            unix_timestamp: 1_622_360_800, // Set a specific timestamp for testing
        };
        let clock_bytes = clock_data.as_mut_slice();
        clock_bytes[..8].copy_from_slice(&clock.slot.to_le_bytes());
        clock_bytes[8..16].copy_from_slice(&clock.epoch_start_timestamp.to_le_bytes());
        clock_bytes[16..24].copy_from_slice(&clock.epoch.to_le_bytes());
        clock_bytes[24..32].copy_from_slice(&clock.leader_schedule_epoch.to_le_bytes());
        clock_bytes[32..40].copy_from_slice(&clock.unix_timestamp.to_le_bytes());

        let (time_lock_account, clock_account) = create_account_info_with_clock(
            &time_lock_key,
            true, true, &mut time_lock_account_lamports, &mut time_lock_account_data, &program_id,
            &mut clock_lamports, &mut clock_data, &sysvar_id, &clock_key,
        );

        let state_account = initialize_state_account(&state_key, &mut state_account_lamports, &mut state_account_data, &program_id);
        let accounts = vec![time_lock_account.clone(), clock_account.clone(), state_account];

        let unlock_time = clock.unix_timestamp as u64 + 1000; // Setting unlock time to future
        let result = DHelixToken::time_lock(&accounts, unlock_time, &vec![time_lock_key]);
        assert!(result.is_err()); // Should be locked

        let unlock_time = clock.unix_timestamp as u64; // Setting unlock time to current time
        let result = DHelixToken::time_lock(&accounts, unlock_time, &vec![time_lock_key]);
        assert!(result.is_ok()); // Should be unlocked
    }

    #[test]
    fn test_emergency_stop() {
        let program_id = Pubkey::new_unique();
        let mut emergency_stop_account_lamports = 300;
        let mut state_account_lamports = 100;
        let mut emergency_stop_account_data = vec![0; 100];
        let mut system_state_account_data = vec![0; 1032]; // Ensure this size is enough for the serialized state + length
        let emergency_stop_key = Pubkey::new_unique();
        let system_state_key = Pubkey::new_unique();

        let emergency_stop_account = create_account_info(
            &emergency_stop_key, true, true, &mut emergency_stop_account_lamports, &mut emergency_stop_account_data, &program_id);

        // Initialize system_state_account with a default SystemState
        let system_state = SystemState {
            halt: false,
            insurance_pool: 0,
        };
        let serialized_state = system_state.try_to_vec().unwrap();
        let serialized_state_len = serialized_state.len();
        system_state_account_data[..serialized_state_len].copy_from_slice(&serialized_state);

        let data_len = system_state_account_data.len(); // Store length in a local variable
        let length_bytes = (serialized_state_len as u64).to_le_bytes();
        system_state_account_data[data_len - 8..].copy_from_slice(&length_bytes);

        let system_state_account = AccountInfo::new(
            &system_state_key, false, true, &mut state_account_lamports, &mut system_state_account_data, &program_id, false, 0);

        let accounts = vec![emergency_stop_account.clone(), system_state_account];

        let result = DHelixToken::emergency_stop(&accounts, &vec![emergency_stop_key]);
        assert!(result.is_ok(), "Emergency stop operation failed: {:?}", result);

        // Verify halt state
        let state = load_system_state(&accounts[1]).unwrap();
        assert!(state.halt, "Halt state was not set correctly");
    }

    #[test]
    fn test_submit_proposal() {
        let program_id = Pubkey::new_unique();
        let mut proposer_account_lamports = 300;
        let mut state_account_lamports = 100;
        let mut proposer_account_data = vec![0; 100];
        let mut proposals_state_account_data = vec![0; 1032]; // Adjust size to include space for length (1024 + 8 bytes)
        let proposer_key = Pubkey::new_unique();
        let proposals_state_key = Pubkey::new_unique();

        let proposer_account = create_account_info(&proposer_key, true, true, &mut proposer_account_lamports, &mut proposer_account_data, &program_id);
        let proposals_state_account = initialize_state_account(&proposals_state_key, &mut state_account_lamports, &mut proposals_state_account_data, &program_id);
        let accounts = vec![proposer_account.clone(), proposals_state_account.clone()];

        // Logging for initialization
        msg!("Initialized proposals state account with key: {}", proposals_state_key);

        let proposal_id = 1;
        let proposal_data = b"Future Project Proposal";

        let result = DHelixDAO::submit_proposal(&accounts, proposal_id, proposal_data);
        assert!(result.is_ok(), "Submit proposal failed: {:?}", result);

        let state = load_proposals_state(&accounts[1]).unwrap();
        assert!(state.proposals.contains_key(&proposal_id), "Proposal not found in state");
        assert_eq!(state.proposals[&proposal_id], proposal_data.to_vec(), "Proposal data mismatch");
    }

    #[test]
    fn test_vote() {
        let program_id = Pubkey::new_unique();
        let mut voter_account_lamports = 300;
        let mut state_account_lamports = 100;
        let mut voter_account_data = vec![0; 100];
        let mut votes_state_account_data = vec![0; 1024]; // Adjust size as necessary
        let voter_key = Pubkey::new_unique();
        let votes_state_key = Pubkey::new_unique();

        let voter_account = create_account_info(&voter_key, true, true, &mut voter_account_lamports, &mut voter_account_data, &program_id);
        let votes_state_account = initialize_state_account(&votes_state_key, &mut state_account_lamports, &mut votes_state_account_data, &program_id);
        let accounts = vec![voter_account.clone(), votes_state_account.clone()];

        let proposal_id = 1;
        let vote = true;

        let result = DHelixDAO::vote(&accounts, proposal_id, vote);
        assert!(result.is_ok(), "Vote failed: {:?}", result);

        let state = load_votes_state(&accounts[1]).unwrap();
        assert!(state.votes.contains_key(&proposal_id), "Vote not found in state");
        assert!(state.votes[&proposal_id].iter().any(|&(ref pk, v)| pk == &voter_key && v == vote), "Vote data mismatch");
    }

    #[test]
    fn test_execute_proposal_mint() {
        let program_id = Pubkey::new_unique();
        let mut executor_account_lamports = 300;
        let mut state_account_lamports = 100;
        let mut token_account_lamports = 100;
        let mut proposals_state_account_lamports = 100;
        let mut executor_account_data = vec![0; 100];
        let mut proposals_state_account_data = vec![0; 1032]; // Adjust size as necessary
        let mut token_account_data = vec![0; TokenAccount::LEN];
        let mut state_account_data = vec![0; 100];
        let executor_key = Pubkey::new_unique();
        let proposals_state_key = Pubkey::new_unique();
        let token_key = Pubkey::new_unique();
        let state_key = Pubkey::new_unique();

        let executor_account = create_account_info(&executor_key, true, true, &mut executor_account_lamports, &mut executor_account_data, &program_id);
        let proposals_state_account = initialize_state_account(&proposals_state_key, &mut proposals_state_account_lamports, &mut proposals_state_account_data, &program_id);
        let token_account = create_account_info(&token_key, false, true, &mut token_account_lamports, &mut token_account_data, &program_id);
        let state_account = create_account_info(&state_key, false, false, &mut state_account_lamports, &mut state_account_data, &program_id);
        let accounts = vec![executor_account.clone(), proposals_state_account.clone(), token_account.clone(), state_account.clone()];

        // Initialize token account
        let token_account_state = TokenAccount {
            is_initialized: true,
            owner: token_key,
            amount: 0,
        };
        TokenAccount::pack(token_account_state.clone(), &mut token_account.data.borrow_mut()).unwrap();

        // Create a mint proposal
        let proposal_id = 1;
        let amount: u64 = 1000;
        let mut proposal_data = vec![0]; // Mint action
        proposal_data.extend_from_slice(&amount.to_le_bytes());

        let mut state = load_proposals_state(&accounts[1]).unwrap();
        state.proposals.insert(proposal_id, proposal_data);
        store_proposals_state(&accounts[1], &state).unwrap();

        let result = DHelixDAO::execute_proposal(&accounts, proposal_id);
        assert!(result.is_ok(), "Execute proposal failed: {:?}", result);

        // Check if proposal execution logic was implemented correctly
        let state = load_proposals_state(&accounts[1]).unwrap();
        assert!(!state.proposals.contains_key(&proposal_id), "Proposal was not executed properly");

        let token_account_state = TokenAccount::unpack(&token_account.data.borrow()).unwrap();
        assert_eq!(token_account_state.amount, amount, "Tokens were not minted correctly");
    }

    #[test]
    fn test_execute_proposal_transfer() {
        let program_id = Pubkey::new_unique();
        let mut executor_account_lamports = 300;
        let mut state_account_lamports = 100;
        let mut proposals_state_account_lamports = 100;
        let mut source_account_lamports = 100;
        let mut destination_account_lamports = 100;
        let mut executor_account_data = vec![0; 100];
        let mut proposals_state_account_data = vec![0; 1032]; // Adjust size as necessary
        let mut source_account_data = vec![0; TokenAccount::LEN];
        let mut destination_account_data = vec![0; TokenAccount::LEN];
        let mut state_account_data = vec![0; 100];
        let executor_key = Pubkey::new_unique();
        let proposals_state_key = Pubkey::new_unique();
        let source_key = Pubkey::new_unique();
        let destination_key = Pubkey::new_unique();
        let state_key = Pubkey::new_unique();

        let executor_account = create_account_info(&executor_key, true, true, &mut executor_account_lamports, &mut executor_account_data, &program_id);
        let proposals_state_account = initialize_state_account(&proposals_state_key, &mut proposals_state_account_lamports, &mut proposals_state_account_data, &program_id);
        let source_account = create_account_info(&source_key, false, true, &mut source_account_lamports, &mut source_account_data, &program_id);
        let destination_account = create_account_info(&destination_key, false, true, &mut destination_account_lamports, &mut destination_account_data, &program_id);
        let state_account = create_account_info(&state_key, false, false, &mut state_account_lamports, &mut state_account_data, &program_id);
        let accounts = vec![executor_account.clone(), proposals_state_account.clone(), source_account.clone(), destination_account.clone(), state_account.clone()];

        // Initialize source and destination accounts
        let src_token_account = TokenAccount {
            is_initialized: true,
            owner: source_key,
            amount: 1000,
        };
        let dest_token_account = TokenAccount {
            is_initialized: true,
            owner: destination_key,
            amount: 0,
        };
        TokenAccount::pack(src_token_account.clone(), &mut source_account.data.borrow_mut()).unwrap();
        TokenAccount::pack(dest_token_account.clone(), &mut destination_account.data.borrow_mut()).unwrap();

        // Create a transfer proposal
        let proposal_id = 2;
        let amount: u64 = 500;
        let mut proposal_data = vec![1]; // Transfer action
        proposal_data.extend_from_slice(&amount.to_le_bytes());
        proposal_data.extend_from_slice(&source_key.to_bytes());
        proposal_data.extend_from_slice(&destination_key.to_bytes());

        let mut state = load_proposals_state(&accounts[1]).unwrap();
        state.proposals.insert(proposal_id, proposal_data);
        store_proposals_state(&accounts[1], &state).unwrap();

        let result = DHelixDAO::execute_proposal(&accounts, proposal_id);
        assert!(result.is_ok(), "Execute proposal failed: {:?}", result);

        // Check if proposal execution logic was implemented correctly
        let state = load_proposals_state(&accounts[1]).unwrap();
        assert!(!state.proposals.contains_key(&proposal_id), "Proposal was not executed properly");

        let src_token_account = TokenAccount::unpack(&source_account.data.borrow()).unwrap();
        let dest_token_account = TokenAccount::unpack(&destination_account.data.borrow()).unwrap();
        assert_eq!(src_token_account.amount, 500, "Tokens were not transferred correctly from source");
        assert_eq!(dest_token_account.amount, 500, "Tokens were not transferred correctly to destination");
    }

    #[test]
    fn test_charity_vote() {
        let program_id = Pubkey::new_unique();
        let mut voter_account_lamports = 300;
        let mut state_account_lamports = 100;
        let mut voter_account_data = vec![0; 100];
        let mut votes_state_account_data = vec![0; 1024]; // Adjust size as necessary
        let voter_key = Pubkey::new_unique();
        let votes_state_key = Pubkey::new_unique();

        let voter_account = create_account_info(&voter_key, true, true, &mut voter_account_lamports, &mut voter_account_data, &program_id);
        let votes_state_account = initialize_state_account(&votes_state_key, &mut state_account_lamports, &mut votes_state_account_data, &program_id);
        let accounts = vec![voter_account.clone(), votes_state_account.clone()];

        // Initialize state with some data
        let initial_state = VotesState {
            votes: HashMap::new(),
        };
        store_votes_state(&votes_state_account, &initial_state).unwrap();

        let proposal_id = 1;
        let vote = true;

        let result = DHelixDAO::charity_vote(&accounts, proposal_id, vote);
        assert!(result.is_ok(), "Charity vote failed: {:?}", result);

        let state = load_votes_state(&accounts[1]).unwrap();
        assert!(state.votes.contains_key(&proposal_id), "Charity vote not found in state");
        assert!(state.votes[&proposal_id].iter().any(|&(ref pk, v)| pk == &voter_key && v == vote), "Charity vote data mismatch");
    }

    #[test]
    fn test_future_project_vote() {
        let program_id = Pubkey::new_unique();
        let mut voter_account_lamports = 300;
        let mut state_account_lamports = 100;
        let mut voter_account_data = vec![0; 100];
        let mut votes_state_account_data = vec![0; 1024]; // Adjust size as necessary
        let voter_key = Pubkey::new_unique();
        let votes_state_key = Pubkey::new_unique();

        let voter_account = create_account_info(&voter_key, true, true, &mut voter_account_lamports, &mut voter_account_data, &program_id);
        let votes_state_account = initialize_state_account(&votes_state_key, &mut state_account_lamports, &mut votes_state_account_data, &program_id);
        let accounts = vec![voter_account.clone(), votes_state_account.clone()];

        let proposal_id = 1;
        let vote = true;

        let result = DHelixDAO::future_project_vote(&accounts, proposal_id, vote);
        assert!(result.is_ok(), "Future project vote failed: {:?}", result);

        let state = load_votes_state(&accounts[1]).unwrap();
        assert!(state.votes.contains_key(&proposal_id), "Future project vote not found in state");
        assert!(state.votes[&proposal_id].iter().any(|&(ref pk, v)| pk == &voter_key && v == vote), "Future project vote data mismatch");
    }

    #[test]
    fn test_incentivized_voting_system() {
        let program_id = Pubkey::new_unique();
        let mut voter_account_lamports = 300;
        let mut votes_state_account_lamports = 100;
        let mut balances_state_account_lamports = 100;
        let mut voter_account_data = vec![0; 100];
        let mut votes_state_account_data = vec![0; 1024]; // Adjust size as necessary
        let mut balances_state_account_data = vec![0; 1024]; // Adjust size as necessary
        let voter_key = Pubkey::new_unique();
        let votes_state_key = Pubkey::new_unique();
        let balances_state_key = Pubkey::new_unique();

        let voter_account = create_account_info(&voter_key, true, true, &mut voter_account_lamports, &mut voter_account_data, &program_id);
        let votes_state_account = initialize_state_account(&votes_state_key, &mut votes_state_account_lamports, &mut votes_state_account_data, &program_id);
        let balances_state_account = initialize_state_account(&balances_state_key, &mut balances_state_account_lamports, &mut balances_state_account_data, &program_id);

        let proposal_id = 1;
        let vote = true;

        {
            let accounts = vec![voter_account.clone(), votes_state_account.clone(), balances_state_account.clone()];

            let result = DHelixToken::incentivized_voting_system(&accounts, proposal_id, vote);
            assert!(result.is_ok(), "Incentivized voting system failed: {:?}", result);
        }

        {
            let state = load_votes_state(&votes_state_account).unwrap();
            let balance = load_balances_state(&balances_state_account).unwrap().balances.get(&voter_key).copied().unwrap_or(0);
            assert_eq!(balance, 10, "Reward amount mismatch");
            assert!(state.votes.contains_key(&proposal_id), "Incentivized vote not found in state");
            assert!(state.votes[&proposal_id].iter().any(|&(ref pk, v)| pk == &voter_key && v == vote), "Incentivized vote data mismatch");
        }
    }

    #[test]
    fn test_dynamic_staking_rewards() {
        let program_id = Pubkey::new_unique();
        let mut staker_account_lamports = 300;
        let mut balances_state_account_lamports = 100;
        let mut staker_account_data = vec![0; 100];
        let mut balances_state_account_data = vec![0; 1024]; // Adjust size as necessary
        let staker_key = Pubkey::new_unique();
        let balances_state_key = Pubkey::new_unique();

        let staker_account = create_account_info(&staker_key, true, true, &mut staker_account_lamports, &mut staker_account_data, &program_id);
        let balances_state_account = initialize_state_account(&balances_state_key, &mut balances_state_account_lamports, &mut balances_state_account_data, &program_id);
        let accounts = vec![staker_account.clone(), balances_state_account.clone()];

        let staking_duration = 100;

        let result = DHelixToken::dynamic_staking_rewards(&accounts, staking_duration);
        assert!(result.is_ok(), "Dynamic staking rewards failed: {:?}", result);

        let balance = load_balances_state(&accounts[1]).unwrap().balances.get(&staker_key).copied().unwrap_or(0);
        assert_eq!(balance, staking_duration * 5, "Staking reward amount mismatch");
    }

    #[test]
    fn test_token_buyback_program() {
        let program_id = Pubkey::new_unique();
        let mut buyback_account_lamports = 300;
        let mut balances_state_account_lamports = 100;
        let mut buyback_account_data = vec![0; 100];
        let mut balances_state_account_data = vec![0; 1024]; // Adjust size as necessary
        let buyback_key = Pubkey::new_unique();
        let balances_state_key = Pubkey::new_unique();

        let buyback_account = create_account_info(&buyback_key, true, true, &mut buyback_account_lamports, &mut buyback_account_data, &program_id);
        let balances_state_account = initialize_state_account(&balances_state_key, &mut balances_state_account_lamports, &mut balances_state_account_data, &program_id);
        let accounts = vec![buyback_account.clone(), balances_state_account.clone()];

        // Initialize buyback account balance
        let mut state = load_balances_state(&accounts[1]).unwrap();
        state.balances.insert(buyback_key, 100);
        store_balances_state(&accounts[1], &state).unwrap();

        let amount = 50;
        let result = DHelixToken::token_buyback_program(&accounts, amount);
        assert!(result.is_ok(), "Token buyback program failed: {:?}", result);

        let balance = load_balances_state(&accounts[1]).unwrap().balances.get(&buyback_key).copied().unwrap_or(0);
        assert_eq!(balance, 50, "Buyback balance mismatch");
    }

    #[test]
    fn test_insurance_pool() {
        let program_id = Pubkey::new_unique();
        let mut insurance_account_lamports = 300;
        let mut balances_state_account_lamports = 100;
        let mut system_state_account_lamports = 100;
        let mut insurance_account_data = vec![0; 100];
        let mut balances_state_account_data = vec![0; 1024]; // Adjust size as necessary
        let mut system_state_account_data = vec![0; 1032]; // Adjust size as necessary
        let insurance_key = Pubkey::new_unique();
        let balances_state_key = Pubkey::new_unique();
        let system_state_key = Pubkey::new_unique();

        let insurance_account = create_account_info(&insurance_key, true, true, &mut insurance_account_lamports, &mut insurance_account_data, &program_id);
        let balances_state_account = initialize_state_account(&balances_state_key, &mut balances_state_account_lamports, &mut balances_state_account_data, &program_id);

        // Initialize system_state_account with a default SystemState
        let system_state = SystemState {
            halt: false,
            insurance_pool: 0,
        };
        let serialized_state = system_state.try_to_vec().unwrap();
        let serialized_state_len = serialized_state.len();
        system_state_account_data[..serialized_state_len].copy_from_slice(&serialized_state);

        let data_len = system_state_account_data.len(); // Store length in a local variable
        let length_bytes = (serialized_state_len as u64).to_le_bytes();
        system_state_account_data[data_len - 8..].copy_from_slice(&length_bytes);

        let system_state_account = AccountInfo::new(
            &system_state_key, false, true, &mut system_state_account_lamports, &mut system_state_account_data, &program_id, false, 0);

        let accounts = vec![insurance_account.clone(), system_state_account.clone(), balances_state_account.clone()];

        // Initialize insurance account balance
        let mut state = load_balances_state(&accounts[2]).unwrap();
        state.balances.insert(insurance_key, 100);
        store_balances_state(&accounts[2], &state).unwrap();

        let amount = 50;
        let result = DHelixToken::insurance_pool(&accounts, amount);
        assert!(result.is_ok(), "Insurance pool failed: {:?}", result);

        // Check updated balance
        let balance = load_balances_state(&accounts[2]).unwrap().balances.get(&insurance_key).copied().unwrap_or(0);
        assert_eq!(balance, 50, "Insurance pool balance mismatch");

        // Check updated system state
        let system_state = load_system_state(&accounts[1]).unwrap();
        assert_eq!(system_state.insurance_pool, amount, "Insurance pool amount mismatch");
    }
}
