#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum AuctionStatus {
    #[default]
    APPROVED = 0,
    Open = 1,
    Closed = 2,
}