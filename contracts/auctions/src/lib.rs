#![no_std]

mod contract;
mod error;
mod storage;

#[cfg(test)]
mod test;

pub mod collection {
    type NftId = u64;
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_nft_collections.wasm"
    );
}

pub mod token {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/soroban_token_contract.wasm"
    );
}
