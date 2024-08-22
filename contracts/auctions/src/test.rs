use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{
    contract::{MarketplaceContract, MarketplaceContractClient},
    storage::ItemInfo,
};

#[test]
fn should_create_auction() {
    let env = Env::default();
    env.mock_all_auths();

    let seller = Address::generate(&env);
    let item_info = ItemInfo {
        item_address: Address::generate(&env),
        item_id: 1u64,
        minimum_price: Some(10),
        buy_now_price: Some(50),
    };
}
