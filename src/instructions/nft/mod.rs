//! NFT (Non-Fungible Token) 资产类型指令模块
//! 
//! 本模块提供NFT资产的完整功能指令集，包括：
//! - 基础操作：铸造、销毁、转账、查询
//! - NFT特有功能：上架、下架、竞价、接受竞价
//! - 交易操作：购买、出售、拍卖、报价
//! - 高级功能：碎片化、合并、租赁、质押
//! - 批量操作：批量交易、批量处理、批量管理、批量同步
//! 
//! 设计特点：
//! - 最小功能单元：每个指令功能单一，职责明确
//! - 细粒度设计：支持灵活组合和扩展
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证和边界检查
//! - 权限控制：细粒度的权限验证和管理
//! - 服务层抽象：核心业务逻辑委托给NftService
//! - 事件驱动：完整的事件发射和审计追踪
//! - 错误处理：全面的错误类型和处理机制

// 基础操作指令
pub mod mint;
pub mod burn;
pub mod transfer;
pub mod query;

// NFT特有功能指令
pub mod list;
pub mod delist;
pub mod bid;
pub mod accept_bid;

// 交易操作指令
pub mod buy;
pub mod sell;
pub mod auction;
pub mod offer;

// 高级功能指令
pub mod fractionalize;
pub mod merge;
pub mod rent;
pub mod stake;

// 批量操作指令
pub mod batch;

// 重新导出基础操作指令
pub use mint::*;
pub use burn::*;
pub use transfer::*;
pub use query::*;

// 重新导出NFT特有功能指令
pub use list::*;
pub use delist::*;
pub use bid::*;
pub use accept_bid::*;

// 重新导出交易操作指令
pub use buy::*;
pub use sell::*;
pub use auction::*;
pub use offer::*;

// 重新导出高级功能指令
pub use fractionalize::*;
pub use merge::*;
pub use rent::*;
pub use stake::*;

// 重新导出批量操作指令
pub use batch::*;

// 重新导出参数结构体
pub use mint::{MintNftParams, MintNft};
pub use burn::{BurnNftParams, BurnNft};
pub use transfer::{TransferNftParams, TransferNft};
pub use query::{QueryNftParams, QueryNft};
pub use list::{ListNftParams, ListNft};
pub use delist::{DelistNftParams, DelistNft};
pub use bid::{BidNftParams, BidNft};
pub use accept_bid::{AcceptBidParams, AcceptBid};
pub use buy::{BuyNftParams, BuyNft};
pub use sell::{SellNftParams, SellNft};
pub use auction::{AuctionNftParams, AuctionNft};
pub use offer::{OfferNftParams, OfferNft};
pub use fractionalize::{FractionalizeNftParams, FractionalizeNft};
pub use merge::{MergeNftParams, MergeNft};
pub use rent::{RentNftParams, RentNft};
pub use stake::{StakeNftParams, StakeNft};
pub use batch::{BatchTradeNftParams, BatchProcessNftParams, BatchManageNftParams, BatchSyncNftParams, BatchNft}; 