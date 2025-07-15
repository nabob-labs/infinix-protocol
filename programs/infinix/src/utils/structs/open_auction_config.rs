use super::PricesInAuction;

#[derive(Default, Copy, Clone)]
pub struct OpenAuctionConfig {
    pub price: PricesInAuction,
    pub sell_limit_spot: u128,
    pub buy_limit_spot: u128,
}
