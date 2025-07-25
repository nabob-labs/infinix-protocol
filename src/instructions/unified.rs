//! Portfolio/Basket/Asset 统一指令集 // 文档注释：投资组合/篮子/资产统一指令集入口
use anchor_lang::prelude::*; // 引入Anchor框架预导入模块，包含Solana程序开发常用类型与宏
use crate::state::baskets::{BasketIndexState, BasketConstituent, PriceFeed}; // 引入篮子指数状态、成分、价格结构体
use crate::state::asset::{AssetManager, AssetInstance, AssetType}; // 引入资产管理器、资产实例、资产类型结构体
use crate::algorithms::traits::{ExecutionStrategy, ExecutionParams, ExecutionResult, RiskManagement, RiskParams, RiskResult}; // 引入算法相关trait与参数类型
use crate::services::portfolio_service::PortfolioService; // 引入投资组合服务层，封装业务逻辑

// ===== 账户声明 =====
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct InitPortfolio<'info> { // 定义初始化投资组合指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(init, payer = authority, space = 8 + BasketIndexState::INIT_SPACE)] // Anchor属性，初始化账户，指定付费者和空间
    pub portfolio: Account<'info, BasketIndexState>, // 投资组合账户，需初始化，类型安全
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者，类型安全
    pub system_program: Program<'info, System>, // 系统程序账户，Anchor自动校验
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)] // 派生Anchor序列化/反序列化、克隆、调试特性，便于跨链/链上数据传递
pub struct InitPortfolioArgs { // 定义初始化投资组合参数结构体
    pub manager: Option<Pubkey>, // 可选管理人公钥
    pub id: u64, // 投资组合ID
    pub composition: Vec<BasketConstituent>, // 投资组合成分
    pub weights: Vec<u64>, // 各成分权重
    pub fee_collector: Pubkey, // 手续费收集者公钥
    pub creation_fee_bps: u16, // 创建费率（基点）
    pub redemption_fee_bps: u16, // 赎回费率（基点）
    pub enable_rebalancing: bool, // 是否启用再平衡
}

/// Initializes a new portfolio.
pub fn init_portfolio(
    ctx: Context<InitPortfolio>, // Anchor账户上下文，自动校验权限与生命周期
    args: InitPortfolioArgs, // 初始化参数结构体
) -> Result<()> { // Anchor规范返回类型
    let portfolio = &mut ctx.accounts.portfolio; // 获取可变投资组合账户，生命周期由Anchor自动管理
    PortfolioService::initialize(
        portfolio, // 投资组合账户
        ctx.accounts.authority.key(), // 操作人公钥
        args.manager, // 管理人公钥
        args.id, // 投资组合ID
        args.composition, // 成分
        args.weights, // 权重
        args.fee_collector, // 手续费收集者
        args.creation_fee_bps, // 创建费率
        args.redemption_fee_bps, // 赎回费率
        args.enable_rebalancing, // 是否启用再平衡
        *ctx.bumps.get("portfolio").unwrap(), // 获取PDA bump种子
    ); // 调用服务层初始化逻辑
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct MintPortfolio<'info> { // 定义投资组合增发指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub portfolio: Account<'info, BasketIndexState>, // 投资组合账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn mint_portfolio(
    ctx: Context<MintPortfolio>, // Anchor账户上下文
    amount: u64, // 增发数量
) -> Result<()> { // Anchor规范返回类型
    let portfolio = &mut ctx.accounts.portfolio; // 获取可变投资组合账户
    let twap = crate::algorithms::execution::twap::TwapImpl; // 引入TWAP算法实现
    let exec_params = ExecutionParams {
        order_size: amount, // 订单数量
        market_impact: 0, // 市场冲击参数
        slippage_tolerance: 100, // 滑点容忍度
    };
    let optimized_amount = PortfolioService::mint(portfolio, amount, &twap, &exec_params)?; // 调用服务层增发逻辑，返回优化后数量
    emit!(PortfolioMinted { // 触发增发事件
        portfolio_id: portfolio.id, // 事件：投资组合ID
        amount: optimized_amount, // 事件：增发数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct BurnPortfolio<'info> { // 定义投资组合销毁指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub portfolio: Account<'info, BasketIndexState>, // 投资组合账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn burn_portfolio(
    ctx: Context<BurnPortfolio>, // Anchor账户上下文
    amount: u64, // 销毁数量
) -> Result<()> { // Anchor规范返回类型
    let portfolio = &mut ctx.accounts.portfolio; // 获取可变投资组合账户
    let risk = crate::algorithms::execution::risk_management::RiskManagementImpl; // 引入风险管理算法实现
    let risk_params = RiskParams {
        position_size: amount, // 持仓规模
        volatility: 10, // 波动率
        max_drawdown: 5, // 最大回撤
    };
    PortfolioService::burn(portfolio, amount, &risk, &risk_params)?; // 调用服务层销毁逻辑
    emit!(PortfolioBurned { // 触发销毁事件
        portfolio_id: portfolio.id, // 事件：投资组合ID
        amount, // 事件：销毁数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct RebalancePortfolio<'info> { // 定义投资组合再平衡指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub portfolio: Account<'info, BasketIndexState>, // 投资组合账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn rebalance_portfolio(
    ctx: Context<RebalancePortfolio>, // Anchor账户上下文
    new_weights: Vec<u64>, // 新权重向量
) -> Result<()> { // Anchor规范返回类型
    let portfolio = &mut ctx.accounts.portfolio; // 获取可变投资组合账户
    let router = crate::algorithms::execution::smart_routing::SmartRoutingImpl; // 引入智能路由算法实现
    let routing_params = crate::algorithms::traits::RoutingParams {
        input_mint: portfolio.composition.get(0).map(|c| c.token_mint).unwrap_or_default(), // 输入资产mint
        output_mint: portfolio.composition.get(1).map(|c| c.token_mint).unwrap_or_default(), // 输出资产mint
        amount_in: 100, // 输入数量
        dex_candidates: vec!["Jupiter".to_string(), "Orca".to_string()], // DEX候选列表
    };
    let best_dex = PortfolioService::rebalance(portfolio, new_weights, &router, &routing_params)?; // 调用服务层再平衡逻辑，返回最佳DEX
    emit!(PortfolioRebalanced { // 触发再平衡事件
        portfolio_id: portfolio.id, // 事件：投资组合ID
        best_dex, // 事件：最佳DEX名称
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct UpdateNavPortfolio<'info> { // 定义投资组合净值更新指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub portfolio: Account<'info, BasketIndexState>, // 投资组合账户，需可变
}

pub fn update_nav_portfolio(
    ctx: Context<UpdateNavPortfolio>, // Anchor账户上下文
    price_feeds: Vec<PriceFeed>, // 价格喂价数组
) -> Result<()> { // Anchor规范返回类型
    let portfolio = &mut ctx.accounts.portfolio; // 获取可变投资组合账户
    let nav = PortfolioService::update_nav(portfolio, price_feeds)?; // 调用服务层净值更新逻辑，返回每份净值
    emit!(PortfolioNavUpdated { // 触发净值更新事件
        portfolio_id: portfolio.id, // 事件：投资组合ID
        nav_per_token: nav, // 事件：每份净值
        timestamp: portfolio.updated_at, // 事件：更新时间戳
    });
    Ok(()) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct TransferPortfolio<'info> { // 定义投资组合转账指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub from_portfolio: Account<'info, BasketIndexState>, // 转出方投资组合账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub to_portfolio: Account<'info, BasketIndexState>, // 转入方投资组合账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn transfer_portfolio(
    ctx: Context<TransferPortfolio>, // Anchor账户上下文
    amount: u64, // 转账数量
) -> Result<()> { // Anchor规范返回类型
    let from = &mut ctx.accounts.from_portfolio; // 获取转出方账户
    let to = &mut ctx.accounts.to_portfolio; // 获取转入方账户
    // 可根据 PortfolioService 细化权限和校验
    require!(
        ctx.accounts.authority.key() == from.authority, // 校验操作人权限
        crate::error::ErrorCode::Unauthorized // 错误码：未授权
    );
    if from.total_value < amount { // 校验转出方余额充足
        return Err(crate::error::ErrorCode::InsufficientFunds.into()); // 不足则返回错误
    }
    from.total_value -= amount; // 扣减转出方余额
    to.total_value = to.total_value.checked_add(amount).ok_or(crate::error::ErrorCode::Overflow)?; // 增加转入方余额，防止溢出
    emit!(PortfolioTransferred { // 触发转账事件
        from_portfolio_id: from.id, // 事件：转出方ID
        to_portfolio_id: to.id, // 事件：转入方ID
        amount, // 事件：转账数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct QueryPortfolio<'info> { // 定义投资组合查询指令的账户上下文结构体
    pub portfolio: Account<'info, BasketIndexState>, // 查询目标投资组合账户
}

pub fn query_portfolio(
    ctx: Context<QueryPortfolio>, // Anchor账户上下文
) -> Result<u64> { // Anchor规范返回类型，返回u64供链上查询
    let portfolio = &ctx.accounts.portfolio; // 获取目标投资组合账户
    emit!(PortfolioQueried { // 触发查询事件
        portfolio_id: portfolio.id, // 事件：投资组合ID
        total_value: portfolio.total_value, // 事件：总价值
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(portfolio.total_value) // 返回总价值
}

// === 新增：报价指令 ===
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct QuotePortfolio<'info> { // 定义投资组合报价指令的账户上下文结构体
    pub portfolio: Account<'info, BasketIndexState>, // 查询目标投资组合账户
}

pub fn quote_portfolio(
    ctx: Context<QuotePortfolio>, // Anchor账户上下文
    params: crate::core::types::SwapParams, // 交易参数
    price_params: crate::core::types::PriceParams, // 价格参数
) -> Result<u64> { // Anchor规范返回类型，返回u64报价
    require!(params.amount_in > 0, anchor_lang::error::ErrorCode::Custom(8001)); // 校验买入数量大于0
    let price = crate::services::portfolio_service::PortfolioService::quote(&ctx.accounts.portfolio, &params, &price_params)?; // 调用服务层报价逻辑
    emit!(PortfolioQueried { // 触发查询事件
        portfolio_id: ctx.accounts.portfolio.id, // 事件：投资组合ID
        total_value: ctx.accounts.portfolio.total_value, // 事件：总价值
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(price) // 返回报价
}

// === 新增：执行买入指令 ===
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct ExecuteBuyPortfolio<'info> { // 定义执行买入指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub portfolio: Account<'info, BasketIndexState>, // 投资组合账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub buyer: Signer<'info>, // 买方签名者
}

pub fn execute_buy_portfolio(
    ctx: Context<ExecuteBuyPortfolio>, // Anchor账户上下文
    params: crate::core::types::SwapParams, // 交易参数
    price: u64, // 买入价格
) -> Result<()> { // Anchor规范返回类型
    require!(params.amount_in > 0, anchor_lang::error::ErrorCode::Custom(8002)); // 校验买入数量大于0
    crate::services::portfolio_service::PortfolioService::execute_buy(&mut ctx.accounts.portfolio, &params, price, ctx.accounts.buyer.key())?; // 调用服务层买入逻辑
    emit!(PortfolioMinted { // 触发增发事件
        portfolio_id: ctx.accounts.portfolio.id, // 事件：投资组合ID
        amount: params.amount_in, // 事件：买入数量
        authority: ctx.accounts.buyer.key(), // 事件：买方
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

// === 新增：执行卖出指令 ===
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct ExecuteSellPortfolio<'info> { // 定义执行卖出指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub portfolio: Account<'info, BasketIndexState>, // 投资组合账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub seller: Signer<'info>, // 卖方签名者
}

pub fn execute_sell_portfolio(
    ctx: Context<ExecuteSellPortfolio>, // Anchor账户上下文
    params: crate::core::types::SwapParams, // 交易参数
    price: u64, // 卖出价格
) -> Result<()> { // Anchor规范返回类型
    require!(params.amount_in > 0, anchor_lang::error::ErrorCode::Custom(8003)); // 校验卖出数量大于0
    crate::services::portfolio_service::PortfolioService::execute_sell(&mut ctx.accounts.portfolio, &params, price, ctx.accounts.seller.key())?; // 调用服务层卖出逻辑
    emit!(PortfolioBurned { // 触发销毁事件
        portfolio_id: ctx.accounts.portfolio.id, // 事件：投资组合ID
        amount: params.amount_in, // 事件：卖出数量
        authority: ctx.accounts.seller.key(), // 事件：卖方
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

// === 新增：执行交换指令 ===
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct ExecuteSwapPortfolio<'info> { // 定义执行交换指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub from_portfolio: Account<'info, BasketIndexState>, // 转出方投资组合账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub to_portfolio: Account<'info, BasketIndexState>, // 转入方投资组合账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn execute_swap_portfolio(
    ctx: Context<ExecuteSwapPortfolio>, // Anchor账户上下文
    from_amount: u64, // 转出数量
    to_amount: u64, // 转入数量
) -> Result<()> { // Anchor规范返回类型
    require!(from_amount > 0 && to_amount > 0, anchor_lang::error::ErrorCode::Custom(8004)); // 校验数量大于0
    crate::services::portfolio_service::PortfolioService::execute_swap(&mut ctx.accounts.from_portfolio, &mut ctx.accounts.to_portfolio, from_amount, to_amount, ctx.accounts.authority.key())?; // 调用服务层交换逻辑
    emit!(PortfolioTransferred { // 触发转账事件
        from_portfolio_id: ctx.accounts.from_portfolio.id, // 事件：转出方ID
        to_portfolio_id: ctx.accounts.to_portfolio.id, // 事件：转入方ID
        amount: from_amount, // 事件：转出数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

// === 新增：执行合并指令 ===
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct ExecuteCombinePortfolio<'info> { // 定义执行合并指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub target_portfolio: Account<'info, BasketIndexState>, // 目标投资组合账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub source_portfolio: Account<'info, BasketIndexState>, // 源投资组合账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn execute_combine_portfolio(
    ctx: Context<ExecuteCombinePortfolio>, // Anchor账户上下文
    amount: u64, // 合并数量
) -> Result<()> { // Anchor规范返回类型
    require!(amount > 0, anchor_lang::error::ErrorCode::Custom(8005)); // 校验合并数量大于0
    crate::services::portfolio_service::PortfolioService::execute_combine(&mut ctx.accounts.target_portfolio, &mut ctx.accounts.source_portfolio, amount, ctx.accounts.authority.key())?; // 调用服务层合并逻辑
    emit!(PortfolioTransferred { // 触发转账事件
        from_portfolio_id: ctx.accounts.source_portfolio.id, // 事件：源ID
        to_portfolio_id: ctx.accounts.target_portfolio.id, // 事件：目标ID
        amount, // 事件：合并数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

// === 新增：执行拆分指令 ===
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct ExecuteSplitPortfolio<'info> { // 定义执行拆分指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub source_portfolio: Account<'info, BasketIndexState>, // 源投资组合账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub new_portfolio: Account<'info, BasketIndexState>, // 新生成投资组合账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn execute_split_portfolio(
    ctx: Context<ExecuteSplitPortfolio>, // Anchor账户上下文
    amount: u64, // 拆分数量
) -> Result<()> { // Anchor规范返回类型
    require!(amount > 0, anchor_lang::error::ErrorCode::Custom(8006)); // 校验拆分数量大于0
    crate::services::portfolio_service::PortfolioService::execute_split(&mut ctx.accounts.source_portfolio, &mut ctx.accounts.new_portfolio, amount, ctx.accounts.authority.key())?; // 调用服务层拆分逻辑
    emit!(PortfolioTransferred { // 触发转账事件
        from_portfolio_id: ctx.accounts.source_portfolio.id, // 事件：源ID
        to_portfolio_id: ctx.accounts.new_portfolio.id, // 事件：新生成ID
        amount, // 事件：拆分数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

// 只保留统一入口和 glue 逻辑，所有 index_token、basket、asset 相关指令已迁移到细分模块 

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::baskets::BasketIndexState;
    use anchor_lang::prelude::*;

    fn default_portfolio() -> BasketIndexState {
        BasketIndexState {
            fee_collector: Pubkey::new_unique(), // 随机生成手续费收集者公钥
            composition: vec![Default::default()], // 默认成分
            weights: vec![10_000], // 默认权重
            is_active: true, // 激活状态
            total_value: 1000, // 默认总价值
            ..Default::default()
        }
    }

    #[test]
    fn test_transfer_portfolio_success() {
        let mut from = default_portfolio(); // 构造转出方
        let mut to = default_portfolio(); // 构造转入方
        let amount = 500; // 转账数量
        from.total_value = 1000; // 设置转出方余额
        to.total_value = 200; // 设置转入方余额
        // 模拟transfer逻辑
        assert!(from.total_value >= amount); // 校验余额充足
        from.total_value -= amount; // 扣减转出方
        to.total_value += amount; // 增加转入方
        assert_eq!(from.total_value, 500); // 校验转出方余额
        assert_eq!(to.total_value, 700); // 校验转入方余额
    }

    #[test]
    fn test_transfer_portfolio_insufficient_value() {
        let mut from = default_portfolio(); // 构造转出方
        let mut to = default_portfolio(); // 构造转入方
        let amount = 2000; // 转账数量
        from.total_value = 1000; // 设置转出方余额
        to.total_value = 200; // 设置转入方余额
        // 模拟transfer逻辑
        assert!(from.total_value < amount); // 校验余额不足
    }

    #[test]
    fn test_query_portfolio() {
        let portfolio = default_portfolio(); // 构造投资组合
        let value = portfolio.total_value; // 获取总价值
        assert_eq!(value, 1000); // 校验总价值
    }
} 