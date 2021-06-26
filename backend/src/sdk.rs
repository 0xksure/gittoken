use rithub::api::api::{self, ReviewComment};
use rithub::app::app;
use rithub::error::errors::Error;
use rocket_contrib::databases::postgres;
use solana_sdk::instruction::AccountMeta;
use std::borrow::Borrow;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::user::user::User;
use rithub::webhook::webhook::WebhookRequest;
use rocket_contrib::database;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::Instruction,
    message::Message,
    native_token::*,
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction, system_program, sysvar,
    transaction::Transaction,
};
use spl_token::state::Account;

fn read_json_array(fname: &str) -> Vec<u8> {
    let file = File::open(fname).expect("no such filename");
    let buf = BufReader::new(file);
    let newbu: Vec<u8> = buf
        .lines()
        .map(|l| l.expect("could not parse line").parse::<u8>().unwrap())
        .collect();
    newbu
}

struct NewAccount;

// create_account creates a new account for a user
// based on their wallet address
pub fn create_account(owner_addr: &str, addr: &str) -> Result<(), Error> {
    let client = RpcClient::new(String::from("http://localhost::8899"));

    let mint_authority = "2J3WneHkTnoxac83xAazUXKZ5JbYmt3Be4CPbnEu2i1s";
    let token_program_id = "DHZypXyN9vRh24S8UgP37DeQ9dpCndDVCTQMMDvuNX8g";
    let owner_pubkey = Pubkey::new(owner_addr.as_bytes());

    let create_account = system_instruction::create_account(
        &owner_pubkey,
        &Pubkey::new(addr.as_bytes()),
        0,
        Account::LEN as u64,
        &Pubkey::new(token_program_id.as_bytes()),
    );

    let message = Message::new(&[create_account], Some(&owner_pubkey));
    let transaction = Transaction::new_unsigned(message);
    let signature = match client.send_transaction(&transaction) {
        Ok(res) => res,
        Err(err) => return Err(Error::new(501, err.to_string())),
    };

    println!("signature: {}", signature);

    Ok(())
}

#[derive(Debug, Clone)]
struct TransferChecked {
    /// The amount of tokens to transfer.
    amount: u64,
    /// Expected number of base 10 digits to the right of the decimal place.
    decimals: u8,
}

pub fn transfer_token(from_addr: &str, to_addr: &str, amount: u64) -> Result<(), Error> {
    let client = RpcClient::new(String::from("http://localhost::8899"));
    let from_pubkey = Pubkey::new(from_addr.as_bytes());
    let to_pubkey = Pubkey::new(to_addr.as_bytes());

    let from_balance = match client.get_balance(&from_pubkey) {
        Ok(res) => res,
        Err(err) => return Err(Error::new(500, err.to_string())),
    };

    if from_addr.eq(to_addr) {
        return Err(Error::new(
            500,
            String::from("should avoid transfer to itself"),
        ));
    }

    if from_balance < amount {
        return Err(Error::new(500, String::from("not enough funds")));
    }

    let token_program_id = Pubkey::new("DHZypXyN9vRh24S8UgP37DeQ9dpCndDVCTQMMDvuNX8g".as_bytes());

    let mut accounts = Vec::new();
    let account_meta = AccountMeta::new(to_pubkey, false);
    accounts.push(account_meta);
    let amount: u64 = 10;
    let decimals: u8 = 0;

    let data = TransferChecked { amount, decimals };
    let mut buf = Vec::with_capacity(std::mem::size_of::<TransferChecked>());
    buf.push(12);
    buf.extend_from_slice(&data.amount.to_le_bytes());
    buf.push(decimals);

    let instruction = Instruction::new_with_bincode(token_program_id, &buf, accounts);
    let message = Message::new(&[instruction], Some(&from_pubkey));
    let transaction = Transaction::new_unsigned(message);
    client.send_transaction(&transaction);

    Ok(())
}
