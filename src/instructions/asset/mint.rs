//!
//! Asset Mint Instruction
//! 资产增发指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use crate::accounts::BasketIndexStateAccount; // 账户状态结构体定义
use crate::events::asset_event::*; // 资产相关事件定义（Anchor事件）
use crate::services::asset_service::AssetService; // 资产业务逻辑服务层
use crate::state::baskets::BasketIndexState; // 资产篮子状态
use crate::validation::asset_validation::AssetValidatable; // 资产校验trait
use crate::core::types::{ExecutionParams, StrategyParams}; // 资产相关参数类型
use anchor_lang::prelude::*; // Anchor预导入，提供Solana合约开发的基础类型和宏

/// 资产增发指令账户上下文
/// - basket_index: 目标资产篮子账户，需可变
/// - authority: 操作人签名者
#[derive(Accounts)] // Anchor宏，自动生成账户校验与生命周期管理代码
pub struct MintAsset<'info> { // 定义资产增发指令的账户上下文结构体，'info生命周期由Anchor自动推断
    /// 目标资产篮子账户，需可变，Anchor自动校验PDA
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub basket_index: Account<'info, BasketIndexState>, // 资产篮子账户，类型安全，生命周期受Anchor管理
    /// 操作人签名者，需可变
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全，生命周期受Anchor管理
}

/// 资产增发指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 增发数量，单位为最小资产单位
/// - 返回: Anchor规范Result
pub fn mint_asset(ctx: Context<MintAsset>, amount: u64) -> Result<()> { // 资产增发指令主函数，ctx为账户上下文，amount为增发数量
    let basket_index = &mut ctx.accounts.basket_index; // 获取可变资产篮子账户，生命周期由Anchor自动管理
    basket_index.validate()?; // 校验资产篮子状态（如活跃、合法等），防止非法操作
    AssetService::mint(basket_index, amount)?; // 调用服务层增发逻辑，处理实际mint，内部包含溢出检查
    emit!(AssetMinted { // 触发资产增发事件，链上可追溯
        basket_id: basket_index.id, // 事件：资产篮子ID，便于链上追踪
        amount, // 事件：增发数量，记录操作明细
        authority: ctx.accounts.authority.key(), // 事件：操作人，便于审计
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳，防篡改
    });
    Ok(()) // Anchor规范返回，表示指令成功
}
