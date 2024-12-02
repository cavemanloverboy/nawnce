//! This example shows how to use durable nonces to expire a transaction
//! early!

use nawnce::setup;
use solana_sdk::{
    signer::Signer,
    system_instruction::{self, advance_nonce_account},
    transaction::Transaction,
};

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

    // Prepare transaction
    let nonce = ctx.fetch_nonce();
    ctx.expire_blockhash(); /* cannot create nonce account + advance in same slot */
    let transaction = Transaction::new_signed_with_payer(
        &instruction_set,
        None,
        &[&ctx.payer],
        nonce,
    );

    // Simulation succeeds (we simulate to avoid execution)
    let res = ctx
        .simulate_transaction(transaction.clone())
        .unwrap();
    println!("Transaction simulation");
    for log in res.meta.logs {
        println!("    {log}");
    }

    // This transaction would normally be valid for 150 slots!
    //
    // However, we can advance our nonce to ensure some mempool does
    // not keep our transaction around if we fail to land and we want
    // to cancel!
    let advance = Transaction::new_signed_with_payer(
        &[advance_nonce_account(
            &ctx.nonce_account,
            &ctx.payer.pubkey(),
        )],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer],
        nonce,
    );
    assert!(ctx
        .send_transaction(advance)
        .is_ok());

    // Now the simulation fails
    assert!(ctx
        .simulate_transaction(transaction.clone())
        .is_err());
    println!("Post-expiry simulation failed!");
}
