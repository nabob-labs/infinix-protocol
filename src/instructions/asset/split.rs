//!
//! Asset Split Instruction
//! 资产拆分指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use crate::accounts::BasketIndexStateAccount; // 账户状态结构体定义
use crate::events::asset_event::*; // 资产相关事件定义（Anchor事件）
use crate::services::asset_service::AssetService; // 资产业务逻辑服务层
use crate::state::baskets::BasketIndexState; // 资产篮子状态
use crate::validation::asset_validation::AssetValidatable; // 资产校验trait
use anchor_lang::prelude::*; // Anchor预导入，提供Solana合约开发的基础类型和宏

/// 资产拆分指令账户上下文
/// - source: 源资产篮子账户，需可变
/// - new_asset: 新资产篮子账户，需可变
/// - authority: 操作人签名者
#[derive(Accounts)] // Anchor宏，自动生成账户校验与生命周期管理代码
pub struct SplitAsset<'info> {
    /// 源资产篮子账户，需可变，Anchor自动校验PDA
    #[account(mut)]
    pub source: Account<'info, BasketIndexState>,
    /// 新资产篮子账户，需可变，Anchor自动校验PDA
    #[account(mut)]
    pub new_asset: Account<'info, BasketIndexState>,
    /// 操作人签名者，需可变
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 资产拆分指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 拆分数量，单位为最小资产单位
/// - 返回: Anchor规范Result
pub fn split_asset(ctx: Context<SplitAsset>, amount: u64) -> Result<()> {
    let source = &mut ctx.accounts.source; // 获取可变源资产篮子账户
    let new_asset = &mut ctx.accounts.new_asset; // 获取可变新资产篮子账户
    source.validate()?; // 校验源资产篮子状态
    new_asset.validate()?; // 校验新资产篮子状态
    // 权限校验：必须是source的authority
    require_keys_eq!(source.authority, ctx.accounts.authority.key(), crate::errors::asset_error::AssetError::AuthorizationFailed);
    // 业务逻辑：source扣减，new_asset增加，内部溢出检查
    AssetService::split(source, new_asset, amount, ctx.accounts.authority.key())?;
    emit!(AssetSplit {
        source_asset_id: source.id, // 事件：源资产ID
        new_asset_id: new_asset.id, // 事件：新资产ID
        amount, // 事件：拆分数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，表示指令成功
} 