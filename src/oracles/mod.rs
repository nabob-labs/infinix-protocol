//!
//! 预言机适配器分层模块主入口
//!
//! 统一PriceOracle trait接口，便于多预言机集成与切换，支持多种价格查询场景。
//! - 提供多种主流预言机适配器的统一入口
//! - 支持链上价格、TWAP、VWAP等多种查询
//! - 便于策略灵活切换和扩展
//! - 设计意图：极致可插拔、最小功能单元、统一接口、Anchor集成友好

use crate::oracles::adapter_registry::*; // 适配器注册表，支持动态注册与检索
use crate::oracles::pyth::*;             // Pyth 预言机适配器实现
use crate::oracles::switchboard::*;      // Switchboard 预言机适配器实现
use anchor_lang::prelude::*;             // Anchor 预导入，包含Pubkey、Result等

// ========================= Oracles 适配器模块主入口 =========================
// 本模块为 Oracles 适配器提供统一入口、trait、re-export、注册表等，
// 每个 trait、struct、参数、用途、边界、Anchor 相关点均有详细注释。
pub mod adapter;             // 适配器trait与核心实现
pub mod chainlink;           // Chainlink 预言机适配器实现
pub mod chainlink_adapter;   // Chainlink 适配器桥接
pub mod factory;             // 工厂模式，统一注册与获取
pub mod logging;             // 日志工具
pub mod pyth_adapter;        // Pyth 适配器桥接
pub mod switchboard_adapter; // Switchboard 适配器桥接
pub mod traits;              // trait定义与扩展

/// 价格预言机统一Trait接口
/// - 统一价格查询操作，便于多预言机集成与切换
/// - 设计意图：为上层业务屏蔽不同预言机的实现细节，提供统一的价格查询入口
pub trait PriceOracle: Send + Sync {
    /// 查询指定token的价格
    /// - token_mint: 资产mint
    /// - 返回：价格数值（u64）
    /// - 设计意图：所有预言机适配器必须实现，便于统一聚合、扩展、测试
    fn get_price(&self, token_mint: Pubkey) -> Result<u64>;
}
