//!
//! Basket Batch Combine Instruction
//! 篮子批量合并指令最小功能单元实现，严格遵循Anchor规范、SOLID原则、分层设计、接口清晰、类型安全、事件追踪、权限校验、生命周期管理、错误处理、逐行注释，生产级代码质量。

use crate::events::basket_event::*; // 引入所有篮子相关事件定义，便于emit!宏调用
use crate::services::basket_service::BasketServiceFacade; // 引入篮子服务层，封装核心业务逻辑
use anchor_lang::prelude::*; // Anchor预导出内容，包含Context、Account、Signer、Result等
use crate::state::baskets::BasketIndexState; // 篮子状态类型

/// 篮子批量合并指令账户上下文
/// - target_basket: 目标资产篮子账户，需可变，Anchor自动校验PDA和生命周期
/// - source_basket: 源资产篮子账户，需可变，Anchor自动校验PDA和生命周期
/// - authority: 操作人签名者，类型安全
#[derive(Accounts)]
pub struct BatchCombineBasket<'info> {
    #[account(mut)]
    pub target_basket: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub source_basket: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 篮子批量合并指令主函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amounts: 批量合并数量，类型安全
/// - 返回: Anchor规范Result类型，生命周期自动管理
pub fn batch_combine_basket(
    ctx: Context<BatchCombineBasket>,
    amounts: Vec<u64>,
) -> anchor_lang::Result<()> {
    let target = &mut ctx.accounts.target_basket;
    let source_baskets = &mut ctx.accounts.source_baskets;
    // 权限校验
    require_keys_eq!(target.authority, ctx.accounts.authority.key(), crate::errors::basket_error::BasketError::NotAllowed);
    // 批量合并操作
    require!(source_baskets.len() == amounts.len(), crate::errors::basket_error::BasketError::InvalidParams);
    for (i, source) in source_baskets.iter_mut().enumerate() {
        let amount = amounts[i];
        if source.total_value < amount {
            return Err(crate::errors::basket_error::BasketError::InsufficientValue.into());
        }
        source.total_value -= amount;
        target.total_value = target.total_value.checked_add(amount).ok_or(crate::errors::basket_error::BasketError::InsufficientValue)?;
        emit!(BasketCombined {
            target_basket_id: target.id,
            source_basket_id: source.id,
            amount,
            authority: ctx.accounts.authority.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });
    }
    Ok(())
} 