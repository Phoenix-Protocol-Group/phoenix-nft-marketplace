#![no_std]

mod contract;
mod error;
mod storage;

pub mod ttl {
    pub const DAY_IN_LEDGERS: u32 = 17280;

    pub const BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
    pub const LIFETIME_THRESHOLD: u32 = BUMP_AMOUNT - DAY_IN_LEDGERS;

    pub const BALANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
    pub const BALANCE_LIFETIME_THRESHOLD: u32 = BALANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;
}

#[cfg(test)]
mod test;
