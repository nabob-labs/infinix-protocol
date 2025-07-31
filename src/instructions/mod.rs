//! Unified entry for all instruction sets: index_token, basket, asset // 文档注释：所有指令集统一入口模块
// 统一指令集入口模块，re-export 所有指令子模块，便于主程序统一集成与调用 // 中文说明：统一导出所有指令子模块，主程序可直接引用
// - 每个子模块均为独立业务域指令集，实现最小功能单元、可插拔、可组合 // 说明：每个子模块独立，便于组合与扩展
// - 便于权限分离、业务扩展、审计与测试 // 说明：有助于权限管理、业务扩展和合规审计
pub mod asset;      // 资产相关指令集子模块声明
pub mod basket;     // 篮子相关指令集子模块声明
pub mod crypto;     // 加密货币相关指令集子模块声明
pub mod etf;        // ETF资产类型指令集子模块声明
pub mod index_token;// 指数代币相关指令集子模块声明
pub mod rwa;        // RWA资产估值指令集子模块声明
pub mod stablecoin; // 稳定币相关指令集子模块声明
pub mod stock;     // 股票相关指令集子模块声明
pub mod nft;       // NFT资产类型指令集子模块声明
pub mod lp_token;  // LP Token资产类型指令集子模块声明
pub mod governance_token; // Governance Token资产类型指令集子模块声明
pub mod staking_token; // Staking Token资产类型指令集子模块声明
pub mod yield_token; // Yield Token资产类型指令集子模块声明
pub mod synthetic_asset; // Synthetic Asset资产类型指令集子模块声明
pub mod options_token; // Options Token资产类型指令集子模块声明
pub mod futures_token; // Futures Token资产类型指令集子模块声明
pub mod perpetual_token; // Perpetual Token资产类型指令集子模块声明
pub mod margin_token; // Margin Token资产类型指令集子模块声明
pub mod adapter;    // 适配器管理指令集子模块声明
pub mod algorithms; // 算法相关指令集子模块声明
pub mod arbitrage; // 跨市场套利指令集子模块声明
pub mod batch_trade; // 批量交易指令集子模块声明
pub mod dex;        // DEX 相关指令集子模块声明
pub mod oracles;    // 预言机相关指令集子模块声明
pub mod strategies; // 策略相关指令集子模块声明
