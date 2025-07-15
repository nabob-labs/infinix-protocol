//! Structs for the Infinix program. Often used within an account.
pub mod auction_status;
pub mod basket_range;
pub mod fee_recipient;
pub mod fixed_size_string;
pub mod infinix_status;
pub mod infinix_token_amount;
pub mod open_auction_config;
pub mod prices;
pub mod rebalance_details;
pub mod roles;
pub mod token_amount;

pub use auction_status::*;
pub use basket_range::*;
pub use fee_recipient::*;
pub use fixed_size_string::*;
pub use infinix_status::*;
pub use infinix_token_amount::*;
pub use open_auction_config::*;
pub use prices::*;
pub use rebalance_details::*;
pub use roles::*;
pub use token_amount::*;
