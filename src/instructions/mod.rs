//! Unified entry for all instruction sets: index_token, basket, asset // 文档注释：所有指令集统一入口模块
// 统一指令集入口模块，re-export 所有指令子模块，便于主程序统一集成与调用 // 中文说明：统一导出所有指令子模块，主程序可直接引用
// - 每个子模块均为独立业务域指令集，实现最小功能单元、可插拔、可组合 // 说明：每个子模块独立，便于组合与扩展
// - 便于权限分离、业务扩展、审计与测试 // 说明：有助于权限管理、业务扩展和合规审计
pub mod asset;      // 资产相关指令集子模块声明
pub mod basket;     // 篮子相关指令集子模块声明
pub mod index_token;// 指数代币相关指令集子模块声明
pub mod algorithms; // 算法相关指令集子模块声明
pub mod strategies; // 策略相关指令集子模块声明
pub mod dex;        // DEX 相关指令集子模块声明
pub mod oracles;    // 预言机相关指令集子模块声明

// 统一 re-export，便于主入口直接调用所有指令 // 说明：统一导出主要指令集，主程序可直接使用
pub use asset::*;      // 资产指令集全部导出
pub use basket::*;     // 篮子指令集全部导出
pub use index_token::*;// 指数代币指令集全部导出
