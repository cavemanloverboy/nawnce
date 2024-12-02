//! This example shows how to use durable nonces for better priority
//! fee discovery!

use nawnce::setup;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    signer::Signer,
    system_instruction::{self, advance_nonce_account},
    transaction::Transaction,
};

fn main() {
    // Set up workspace
    let mut ctx = setup();
    let payer_key = ctx.payer.pubkey();
    let nonce_account = ctx.nonce_account;

    // Generate some instruction set, adding an advance_nonce_ix at the
    // beginning.
    //
    // This could be a swap or transfer or whatever. For now just noop
    // transfer.
    let instruction_set = |fee: u64| {
        vec![
            // Instruction 1: Advance Nonce
            advance_nonce_account(&nonce_account, &payer_key),
            // Instruction 2: Set Prio Fee
            ComputeBudgetInstruction::set_compute_unit_price(fee),
            // Instruction 3-N: Desired instructions
            system_instruction::transfer(&payer_key, &payer_key, 0),
        ]
    };

    // Get some median fee from some RPC provider
    let median_fee = 500_000;
    let try_fees = [median_fee / 10, median_fee / 2, median_fee];

    // Generate transactions for these with shared nonce
    let nonce = ctx.fetch_nonce();
    ctx.expire_blockhash(); /* cannot create nonce account + advance in same slot */
    let txs = try_fees.map(|fee| {
        Transaction::new_signed_with_payer(
            &instruction_set(fee),
            None,
            &[&ctx.payer],
            nonce,
        )
    });

    // Suppose we send all of them now, with a 20 ms delay between them
    // but only the middle one lands! (we paid less than median!)
    let res = {
        // ctx.send_transaction(txs[0].clone()); /* doesn't land */
        /* wait 20ms */
        ctx.send_transaction(txs[1].clone()) /* lands! */
        /* wait 20 ms */
        // ctx.send_transaction(txs[2].clone()); /* doesn't land */
    };
    assert!(res.is_ok());
    for log in res.unwrap().logs {
        println!("    {log}");
    }
    println!("Middle bid hit!");

    // Now the other two fail!
    assert!(ctx
        .send_transaction(txs[0].clone())
        .is_err());
    assert!(ctx
        .send_transaction(txs[2].clone())
        .is_err());
    println!("Other two fail!");
}
