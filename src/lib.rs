use std::ops::{Deref, DerefMut};

use litesvm::LiteSVM;
use solana_sdk::{
    hash::Hash,
    instruction::Instruction,
    native_token::LAMPORTS_PER_SOL,
    nonce::{state::Versions, State},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_instruction, system_program,
    transaction::Transaction,
};

pub struct Workspace {
    pub svm: LiteSVM,
    pub payer: Keypair,
    pub nonce_account: Pubkey,
}

impl Workspace {
    pub fn with_tip_and_nonce(
        &mut self,
        mut instructions: Vec<Instruction>,
        to: &Pubkey,
        amount: u64,
        nonce: Hash,
    ) -> Transaction {
        // Add static tip instruction at end
        instructions.push(tip(&self.payer.pubkey(), to, amount));

        // Sign
        Transaction::new_signed_with_payer(
            &instructions,
            None,
            &[&self.payer],
            nonce,
        )
    }

    pub fn fetch_nonce(&mut self) -> Hash {
        let data = self
            .svm
            .get_account(&self.nonce_account)
            .unwrap()
            .data;
        let versions: Versions = bincode::deserialize(&data).unwrap();
        match versions {
            Versions::Legacy(_) => unreachable!(),
            Versions::Current(state) => match *state {
                State::Uninitialized => unreachable!(),
                State::Initialized(data) => {
                    assert_eq!(data.authority, self.payer.pubkey());
                    *data
                        .durable_nonce
                        .as_hash()
                }
            },
        }
    }
}

/// Initializes LiteSVM with payer and durable nonce account
pub fn setup() -> Workspace {
    // Initialize LiteSVM
    let mut svm = LiteSVM::new();

    // Generate and fund payer
    let payer = Keypair::new();
    let payer_key = payer.pubkey();
    svm.airdrop(&payer_key, 100 * LAMPORTS_PER_SOL)
        .unwrap();

    // Initialize nonce account
    let seed = "my nonce account";
    let nonce_account =
        Pubkey::create_with_seed(&payer_key, seed, &system_program::ID)
            .unwrap();
    let create_nonce_account_ixs =
        system_instruction::create_nonce_account_with_seed(
            &payer_key,
            &nonce_account,
            &payer_key,
            seed,
            &payer_key,
            LAMPORTS_PER_SOL,
        );
    let blockhash = svm.latest_blockhash();
    let transaction = Transaction::new_signed_with_payer(
        &create_nonce_account_ixs,
        None,
        &[&payer],
        blockhash,
    );
    svm.send_transaction(transaction)
        .unwrap();

    Workspace {
        svm,
        payer,
        nonce_account,
    }
}

/// Shorter transfer alias for clarity
#[inline(always)]
fn tip(from: &Pubkey, to: &Pubkey, amount: u64) -> Instruction {
    system_instruction::transfer(from, to, amount)
}

impl Deref for Workspace {
    type Target = LiteSVM;
    fn deref(&self) -> &Self::Target {
        &self.svm
    }
}

impl DerefMut for Workspace {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.svm
    }
}
