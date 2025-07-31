//!
//! Asset Transfer Instruction
//! 资产转账指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use crate::events::asset_event::*; // 资产相关事件定义（Anchor事件）
use crate::services::asset_service::AssetService; // 资产业务逻辑服务层
use crate::state::baskets::BasketIndexState; // 篮子状态类型
use anchor_lang::prelude::*; // Anchor预导入，提供Solana合约开发的基础类型和宏

/// 资产转账指令账户上下文
/// - from: 源资产篮子账户，需可变
/// - to: 目标资产篮子账户，需可变
/// - authority: 操作人签名者
#[derive(Accounts)] // Anchor宏，自动生成账户校验与生命周期管理代码
pub struct TransferAsset<'info> {
    /// 源资产篮子账户，需可变，Anchor自动校验PDA
    #[account(mut)]
    pub from: Account<'info, BasketIndexState>,
    /// 目标资产篮子账户，需可变，Anchor自动校验PDA
    #[account(mut)]
    pub to: Account<'info, BasketIndexState>,
    /// 操作人签名者，需可变
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 资产转账指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 转账数量，单位为最小资产单位
/// - 返回: Anchor规范Result
pub fn transfer_asset(ctx: Context<TransferAsset>, amount: u64) -> anchor_lang::Result<()> {
    let from = &mut ctx.accounts.from; // 获取可变源资产篮子账户
    let to = &mut ctx.accounts.to; // 获取可变目标资产篮子账户
    from.validate()?; // 校验源资产篮子状态（如活跃、合法等），防止非法操作
    to.validate()?; // 校验目标资产篮子状态
    // 权限校验：必须是from的authority
    require_keys_eq!(from.authority, ctx.accounts.authority.key(), crate::errors::asset_error::AssetError::AuthorizationFailed);
    // 业务逻辑：from扣减，to增加，内部溢出检查
    if from.total_value < amount {
        return Err(crate::errors::asset_error::AssetError::InsufficientValue.into());
    }
    from.total_value -= amount;
    to.total_value = to.total_value.checked_add(amount).ok_or(crate::errors::asset_error::AssetError::TransferFailed)?;
    emit!(AssetTransferred {
        from_asset_id: from.id, // 事件：源资产ID
        to_asset_id: to.id, // 事件：目标资产ID
        amount, // 事件：转账数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，表示指令成功
} 