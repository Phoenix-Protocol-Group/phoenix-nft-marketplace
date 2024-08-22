use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    Unauthorized = 0,
    AuctionNotFound = 1,
    IDMissmatch = 2,
    BidNotEnough = 3,
    AuctionNotFinished = 4,
    NotEnoughBalance = 5,
    InvalidInputs = 6,
    AuctionNotActive = 7,
    MinPriceNotReached = 8,
    MissingHighestBid = 9,
    AuctionNotPaused = 10,
    PaymentProcessingFailed = 11,
    NoBuyNowOption = 12,
}
