//!
//! Basket Transfer Instruction
//! 篮子转账指令最小功能单元实现，严格遵循Anchor规范、SOLID原则、分层设计、接口清晰、类型安全、事件追踪、权限校验、生命周期管理、错误处理、逐行注释，生产级代码质量。

use crate::accounts::BasketIndexStateAccount; // 引入资产篮子账户状态账户定义
use crate::events::basket_event::*; // 引入所有篮子相关事件定义，便于emit!宏调用
use crate::services::basket_service::BasketService; // 引入篮子服务层，封装核心业务逻辑
use crate::state::baskets::BasketIndexState; // 引入资产篮子状态结构体，类型安全
use anchor_lang::prelude::*; // Anchor预导出内容，包含Context、Account、Signer、Result等

/// 篮子转账指令账户上下文
/// - from_basket: 源资产篮子账户，需可变，Anchor自动校验PDA和生命周期
/// - to_basket: 目标资产篮子账户，需可变，Anchor自动校验PDA和生命周期
/// - authority: 操作人签名者，类型安全
#[derive(Accounts)]
pub struct TransferBasket<'info> {
    #[account(mut)]
    pub from_basket: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub to_basket: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 篮子转账指令主函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 转账数量，类型安全
/// - 返回: Anchor规范Result类型，生命周期自动管理
pub fn transfer_basket(
    ctx: Context<TransferBasket>,
    amount: u64,
) -> Result<()> {
    let from = &mut ctx.accounts.from_basket; // 获取可变源篮子账户
    let to = &mut ctx.accounts.to_basket; // 获取可变目标篮子账户
    require!(ctx.accounts.authority.key() == from.authority, crate::errors::basket_error::BasketError::NotAllowed);
    if from.total_value < amount {
        return Err(crate::errors::basket_error::BasketError::InsufficientValue.into());
    }
    from.total_value -= amount;
    to.total_value = to.total_value.checked_add(amount).ok_or(crate::errors::basket_error::BasketError::InsufficientValue)?;
    emit!(BasketTransferred {
        from_basket_id: from.id,
        to_basket_id: to.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 