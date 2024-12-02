//! This example shows how to use durable nonces to only pay a single
//! transaction landing service for executing some instruction set
//! instead of paying them all.

use nawnce::setup;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signer::Signer,
    system_instruction::{self, advance_nonce_account},
};

const TIP_ACCOUNT_ONE: Pubkey =
    solana_sdk::pubkey!("firstfirstfirstfirstfirstfirstfirstfirstfir");
const TIP_ACCOUNT_TWO: Pubkey =
    solana_sdk::pubkey!("secondsecondsecondsecondsecondsecondseconds");
fn main() {
    // Set up workspace
    let mut ctx = setup();

    // Generate some instruction set, adding an advance_nonce_ix at the
    // beginning.
    //
    // This could be a swap or transfer or whatever. For now just noop
    // transfer.
    let instruction_set = vec![
        // Instruction 1: Advance Nonce
        advance_nonce_account(&ctx.nonce_account, &ctx.payer.pubkey()),
        // Instruction 2-N: Desired instructions
        system_instruction::transfer(
            &ctx.payer.pubkey(),
            &ctx.payer.pubkey(),
            0,
        ),
    ];

    // Prepare transactions for both services
    let tip_amount = LAMPORTS_PER_SOL;
    let nonce = ctx.fetch_nonce();
    ctx.expire_blockhash();
    println!("Service 1 with nonce {nonce}");
    let tx_tip_1 = ctx.with_tip_and_nonce(
        instruction_set.clone(),
        &TIP_ACCOUNT_ONE,
        tip_amount,
        nonce,
    );
    let tx_tip_2 = ctx.with_tip_and_nonce(
        instruction_set,
        &TIP_ACCOUNT_TWO,
        tip_amount,
        nonce,
    );

    // Send to first
    let res = ctx
        .send_transaction(tx_tip_1)
        .unwrap();

    for log in res.logs {
        println!("    {log}");
    }

    // Send to second fails!
    assert!(ctx
        .send_transaction(tx_tip_2)
        .is_err());
    println!("Send to Service 2 failed!");
}
