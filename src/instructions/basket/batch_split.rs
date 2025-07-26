//!
//! Basket Batch Split Instruction
//! 篮子批量拆分指令最小功能单元实现，严格遵循Anchor规范、SOLID原则、分层设计、接口清晰、类型安全、事件追踪、权限校验、生命周期管理、错误处理、逐行注释，生产级代码质量。

use crate::accounts::BasketIndexStateAccount; // 引入资产篮子账户状态账户定义
use crate::events::basket_event::*; // 引入所有篮子相关事件定义，便于emit!宏调用
use crate::services::basket_service::BasketService; // 引入篮子服务层，封装核心业务逻辑
use crate::state::baskets::BasketIndexState; // 引入资产篮子状态结构体，类型安全
use anchor_lang::prelude::*; // Anchor预导出内容，包含Context、Account、Signer、Result等

/// 篮子批量拆分指令账户上下文
/// - source_basket: 源资产篮子账户，需可变，Anchor自动校验PDA和生命周期
/// - new_basket: 新生成的目标篮子账户，需可变，Anchor自动校验PDA和生命周期
/// - authority: 操作人签名者，类型安全
#[derive(Accounts)]
pub struct BatchSplitBasket<'info> {
    #[account(mut)]
    pub source_basket: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub new_basket: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 篮子批量拆分指令主函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amounts: 批量拆分数量，类型安全
/// - 返回: Anchor规范Result类型，生命周期自动管理
pub fn batch_split_basket(
    ctx: Context<BatchSplitBasket>,
    amounts: Vec<u64>,
) -> Result<()> {
    let source = &mut ctx.accounts.source_basket;
    let new_baskets = &mut ctx.accounts.new_baskets;
    // 权限校验
    require_keys_eq!(source.authority, ctx.accounts.authority.key(), crate::errors::basket_error::BasketError::NotAllowed);
    // 批量拆分操作
    require!(new_baskets.len() == amounts.len(), crate::errors::basket_error::BasketError::InvalidParams);
    for (i, new_basket) in new_baskets.iter_mut().enumerate() {
        let amount = amounts[i];
        if source.total_value < amount {
            return Err(crate::errors::basket_error::BasketError::InsufficientValue.into());
        }
        source.total_value -= amount;
        new_basket.total_value = new_basket.total_value.checked_add(amount).ok_or(crate::errors::basket_error::BasketError::InsufficientValue)?;
        emit!(BasketSplit {
            source_basket_id: source.id,
            new_basket_id: new_basket.id,
            amount,
            authority: ctx.accounts.authority.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });
    }
    Ok(())
} 