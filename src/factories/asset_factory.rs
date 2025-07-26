//!
//! asset_factory.rs - 资产工厂
//!
//! 本文件实现资产工厂及相关方法，严格遵循Rust、Solana、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::state::asset::{AssetManager, AssetType};
use crate::core::performance::ExecutionStats;

/// 资产工厂
/// - 负责资产管理器的创建与初始化
pub struct AssetFactory;

impl AssetFactory {
    /// 创建资产管理器
    /// - authority: 管理员
    /// - asset_type: 资产类型
    /// - fee_collector: 费用收集者
    /// - creation_fee_bps: 创建费用bps
    /// - redemption_fee_bps: 赎回费用bps
    /// - bump: PDA bump
    /// - 返回：资产管理器对象
    pub fn create_asset_manager(
        authority: Pubkey,
        asset_type: AssetType,
        fee_collector: Pubkey,
        creation_fee_bps: u16,
        redemption_fee_bps: u16,
        bump: u8,
    ) -> AssetManager {
        let mut manager = AssetManager {
            base: crate::state::common::BaseAccount::default(),
            asset_type,
            asset_count: 0,
            fee_collector,
            creation_fee_bps,
            redemption_fee_bps,
            total_value_locked: 0,
            execution_stats: ExecutionStats::default(),
            created_at: 0,
            updated_at: 0,
            bump,
        };
        // 可扩展初始化逻辑
        manager
    }
} 