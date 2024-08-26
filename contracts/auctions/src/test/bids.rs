use soroban_sdk::{testutils::Address as _, token, Address, Env};

use crate::{
    storage::ItemInfo,
    test::setup::{generate_marketplace_and_collection_client, WEEKLY},
};

#[test]
fn should_place_a_bid() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    let seller = Address::generate(&env);
    let bidder_a = Address::generate(&env);
    let bidder_b = Address::generate(&env);
    let bidder_c = Address::generate(&env);

    let (mp_client, nft_collection_client) =
        generate_marketplace_and_collection_client(env.clone(), seller.clone(), None, None);
    let token_client = token::Client::new(&env, &Address::generate(&env));

    let item_info = ItemInfo {
        collection_addr: nft_collection_client.address.clone(),
        item_id: 1u64,
        minimum_price: Some(10),
        buy_now_price: Some(50),
    };

    mp_client.create_auction(&item_info, &seller, &WEEKLY, &token_client.address);

    mp_client.place_bid(&1, &bidder_a, &10);
    assert_eq!(mp_client.get_highest_bid(&1), (Some(10u64), bidder_a));

    mp_client.place_bid(&1, &bidder_b, &20);
    assert_eq!(mp_client.get_highest_bid(&1), (Some(20u64), bidder_b));

    mp_client.place_bid(&1, &bidder_c, &40);
    assert_eq!(mp_client.get_highest_bid(&1), (Some(40u64), bidder_c));
}
