use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct OperationInput {
    pub operator: u8,
    pub entry_a: u32,
    pub entry_b: u32,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct Operation {
    pub operator: u8,
    pub entry_a: u32,
    pub entry_b: u32,
    pub result: u32,
    pub user: Pubkey,
    pub timestamp: i64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct UserCounter {
    pub operation_counter: u64,
}

impl UserCounter {
    pub fn size() -> usize {
        8
    }
}

impl Operation {
    pub fn size() -> usize {
        1 + 4 + 4 + 4 + 32 + 8
    }
}

entrypoint!(process_instructions);

pub fn process_instructions(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Calculator program started");

    let account_iter = &mut accounts.iter();
    let signer = next_account_info(account_iter)?;
    let user_counter_account = next_account_info(account_iter)?;
    let operation_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    msg!("Accounts parsed successfully");
    msg!("Signer: {}", signer.key);
    msg!("User counter account: {}", user_counter_account.key);
    msg!("Operation account: {}", operation_account.key);

    if !signer.is_signer {
        msg!("ERROR: Signer verification failed");
        return Err(ProgramError::MissingRequiredSignature);
    }

    msg!("Parsing instruction data: {:?}", instruction_data);
    let input = match OperationInput::try_from_slice(instruction_data) {
        Ok(input) => {
            msg!("Input parsed successfully: {:?}", input);
            input
        }
        Err(err) => {
            msg!("ERROR: Failed to parse instruction data: {:?}", err);
            return Err(ProgramError::InvalidInstructionData);
        }
    };

    let result = match input.operator {
        1 => input.entry_a + input.entry_b,
        2 => input.entry_a - input.entry_b,
        3 => input.entry_a * input.entry_b,
        4 => {
            if input.entry_b == 0 {
                msg!("ERROR: Division by zero");
                return Err(ProgramError::InvalidArgument);
            }
            input.entry_a / input.entry_b
        }
        _ => {
            msg!("ERROR: Invalid operator {}", input.operator);
            return Err(ProgramError::InvalidInstructionData);
        }
    };

    msg!("Calculation result: {}", result);

    msg!("Checking if user counter account exists");
    let mut operation_index = 0;

    if user_counter_account.data_len() == 0 {
        msg!("Creating new user counter account");
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(UserCounter::size());

        let (pda, bump) =
            Pubkey::find_program_address(&[b"user_counter", signer.key.as_ref()], program_id);

        msg!("User counter PDA: {}, bump: {}", pda, bump);

        if pda != *user_counter_account.key {
            msg!("ERROR: User counter account doesn't match PDA");
            msg!("Expected: {}", pda);
            msg!("Received: {}", user_counter_account.key);
            return Err(ProgramError::InvalidSeeds);
        }

        msg!(
            "Creating user counter account with {} lamports and {} bytes",
            lamports,
            UserCounter::size()
        );

        invoke_signed(
            &system_instruction::create_account(
                signer.key,
                user_counter_account.key,
                lamports,
                UserCounter::size() as u64,
                program_id,
            ),
            &[
                signer.clone(),
                user_counter_account.clone(),
                system_program.clone(),
            ],
            &[&[b"user_counter", signer.key.as_ref(), &[bump]]],
        )?;

        msg!("User counter account created successfully");

        let mut data = user_counter_account.try_borrow_mut_data()?;
        match (UserCounter {
            operation_counter: 0,
        })
        .serialize(&mut &mut data[..])
        {
            Ok(_) => msg!("User counter initialized to 0"),
            Err(err) => {
                msg!("ERROR: Failed to initialize user counter: {:?}", err);
                return Err(ProgramError::InvalidAccountData);
            }
        }
    } else {
        if user_counter_account.owner != program_id {
            msg!("User counter account not owned by this program");
            return Err(ProgramError::IllegalOwner);
        }

        msg!("Reading existing user counter");
        let counter_data = user_counter_account.data.borrow_mut();
        let user_counter = match UserCounter::try_from_slice(&counter_data) {
            Ok(counter) => {
                msg!("Counter value: {}", counter.operation_counter);
                counter
            }
            Err(err) => {
                msg!("ERROR: Failed to deserialize user counter: {:?}", err);
                return Err(ProgramError::InvalidAccountData);
            }
        };
        operation_index = user_counter.operation_counter;
    }

    msg!("Deriving operation PDA for index: {}", operation_index);
    let (operation_pda, bump) = Pubkey::find_program_address(
        &[
            b"operation",
            signer.key.as_ref(),
            &operation_index.to_le_bytes(),
        ],
        program_id,
    );

    msg!("Operation PDA: {}, bump: {}", operation_pda, bump);

    if *operation_account.key != operation_pda {
        msg!("ERROR: Operation account doesn't match PDA");
        msg!("Expected: {}", operation_pda);
        msg!("Received: {}", operation_account.key);
        return Err(ProgramError::InvalidSeeds);
    }

    msg!("Creating operation account");
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(Operation::size());

    msg!(
        "Creating operation account with {} lamports and {} bytes",
        lamports,
        Operation::size()
    );

    invoke_signed(
        &system_instruction::create_account(
            signer.key,
            operation_account.key,
            lamports,
            Operation::size() as u64,
            program_id,
        ),
        &[
            signer.clone(),
            operation_account.clone(),
            system_program.clone(),
        ],
        &[&[
            b"operation",
            signer.key.as_ref(),
            &operation_index.to_le_bytes(),
            &[bump],
        ]],
    )?;

    msg!("Operation account created successfully");

    let clock = Clock::get()?;
    let op = Operation {
        operator: input.operator,
        entry_a: input.entry_a,
        entry_b: input.entry_b,
        result,
        user: *signer.key,
        timestamp: clock.unix_timestamp,
    };

    msg!("Storing operation: {:?}", op);
    match op.serialize(&mut *operation_account.try_borrow_mut_data()?) {
        Ok(_) => msg!("Operation data stored successfully"),
        Err(err) => {
            msg!("ERROR: Failed to store operation data: {:?}", err);
            return Err(ProgramError::InvalidAccountData);
        }
    }
    msg!(
        "User counter account data len: {}",
        user_counter_account.data_len()
    );

    msg!("Updating user counter");
    let mut counter_data = user_counter_account.try_borrow_mut_data()?;
    let mut user_counter = match UserCounter::try_from_slice(&counter_data) {
        Ok(counter) => counter,
        Err(err) => {
            msg!(
                "ERROR: Failed to deserialize user counter for update: {:?}",
                err
            );
            return Err(ProgramError::InvalidAccountData);
        }
    };

    user_counter.operation_counter += 1;
    msg!("New counter value: {}", user_counter.operation_counter);

    match user_counter.serialize(&mut &mut counter_data[..]) {
        Ok(_) => msg!("User counter updated successfully"),
        Err(err) => {
            msg!("ERROR: Failed to update user counter: {:?}", err);
            return Err(ProgramError::InvalidAccountData);
        }
    }

    msg!("Operation complete!");
    Ok(())
}
