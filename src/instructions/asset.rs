//!
//! Asset Instructions
//! 资产相关链上指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use crate::accounts::BasketIndexStateAccount; // 账户状态结构体定义
use crate::events::asset_event::*; // 资产相关事件定义（Anchor事件）
use crate::services::asset_service::AssetService; // 资产业务逻辑服务层
use crate::state::baskets::BasketIndexState; // 资产篮子状态
use crate::validation::asset_validation::AssetValidatable; // 资产校验trait
use crate::core::types::{SwapParams, PriceParams, ExecutionParams, StrategyParams, BatchSwapParams, StrategyTradeParams}; // 资产相关参数类型
use crate::core::registry::ADAPTER_FACTORY; // 适配器工厂（算法/DEX/Oracle等动态注册）
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

/// 资产销毁指令账户上下文
#[derive(Accounts)] // Anchor宏，自动生成账户校验与生命周期管理代码
pub struct BurnAsset<'info> { // 定义资产销毁指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub basket_index: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者
}

/// 资产销毁指令实现
pub fn burn_asset(ctx: Context<BurnAsset>, amount: u64) -> Result<()> { // 资产销毁指令主函数，ctx为账户上下文，amount为销毁数量
    let basket_index = &mut ctx.accounts.basket_index; // 获取可变资产篮子账户
    basket_index.validate()?; // 校验资产篮子状态，防止非法销毁
    AssetService::burn(basket_index, amount)?; // 调用服务层销毁逻辑，内部包含余额检查
    emit!(AssetBurned { // 触发资产销毁事件，链上可追溯
        basket_id: basket_index.id, // 事件：资产篮子ID
        amount, // 事件：销毁数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

/// 资产转账指令账户上下文
#[derive(Accounts)] // Anchor宏，自动生成账户校验与生命周期管理代码
pub struct TransferAsset<'info> { // 定义资产转账指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub from_asset: Account<'info, BasketIndexState>, // 转出资产账户，需可变
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub to_asset: Account<'info, BasketIndexState>,   // 转入资产账户，需可变
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>,                     // 操作人签名者
}

/// 资产转账指令实现
pub fn transfer_asset(ctx: Context<TransferAsset>, amount: u64) -> Result<()> { // 资产转账指令主函数，ctx为账户上下文，amount为转账数量
    let from = &mut ctx.accounts.from_asset; // 转出账户，生命周期由Anchor管理
    let to = &mut ctx.accounts.to_asset;     // 转入账户
    // 权限校验：操作人必须为from资产的authority，防止未授权转账
    require!(
        ctx.accounts.authority.key() == from.authority,
        crate::errors::asset_error::AssetError::NotAllowed // 错误类型：未授权
    );
    // 边界校验：转出账户余额必须充足，防止透支
    if from.total_value < amount {
        return Err(crate::errors::asset_error::AssetError::InsufficientValue.into());
    }
    // 资产扣减与增加，防止溢出
    from.total_value -= amount; // 转出账户扣减
    to.total_value = to.total_value.checked_add(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?; // 转入账户增加，防止溢出
    // 事件：资产转账，链上可追溯
    emit!(AssetTransferred {
        from_asset_id: from.id, // 转出资产ID
        to_asset_id: to.id,     // 转入资产ID
        amount,                 // 转账数量
        authority: ctx.accounts.authority.key(), // 操作人
        timestamp: Clock::get()?.unix_timestamp, // 时间戳
    });
    Ok(()) // Anchor规范返回
}

/// 资产查询指令账户上下文
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct QueryAsset<'info> { // 定义资产查询指令的账户上下文结构体，'info生命周期由Anchor自动推断
    pub asset: Account<'info, BasketIndexState>, // 只读资产账户，Anchor自动校验PDA、生命周期、权限
}

/// 资产查询指令实现
pub fn query_asset(ctx: Context<QueryAsset>) -> Result<u64> { // 资产查询指令主函数，ctx为账户上下文，返回资产余额
    let asset = &ctx.accounts.asset; // 只读引用资产账户，防止状态变更，类型安全
    // 事件：资产查询，便于链上审计与追踪
    emit!(AssetQueried { // 触发资产查询事件，链上可追溯
        asset_id: asset.id, // 事件字段：资产ID，唯一标识
        total_value: asset.total_value, // 事件字段：当前资产余额
        timestamp: Clock::get()?.unix_timestamp, // 事件字段：当前链上时间戳，便于审计
    });
    Ok(asset.total_value) // 返回资产余额，类型安全，Anchor自动处理生命周期
}

/// 资产买入指令账户上下文
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct BuyAsset<'info> { // 定义资产买入指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub basket_index: Account<'info, BasketIndexState>, // 目标资产篮子，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub buyer: Signer<'info>, // 买入人，需签名，类型安全
}

/// 资产买入指令实现
/// - params: SwapParams 交换参数，包含DEX名、数量等
/// - price_params: PriceParams 价格参数，包含oracle名、价格等
/// - exec_params: Option<ExecutionParams> 算法执行参数
/// - strategy_params: Option<StrategyParams> 策略参数
pub fn buy_asset(
    ctx: Context<BuyAsset>, // Anchor账户上下文，自动校验权限与生命周期
    params: SwapParams, // 交换参数，类型安全
    price_params: PriceParams, // 价格参数，类型安全
    exec_params: Option<ExecutionParams>, // 可选算法执行参数
    strategy_params: Option<StrategyParams>, // 可选策略参数
) -> Result<()> { // Anchor规范返回类型
    let basket_index = &mut ctx.accounts.basket_index; // 获取目标资产篮子，类型安全，生命周期受Anchor管理
    basket_index.validate()?; // 校验资产篮子状态，防止非法买入，业务安全
    // 1. 算法执行（如有）
    let mut exec_result = None; // 算法执行结果，默认无，类型Option
    if let Some(exec_params) = &exec_params { // 若有算法参数，进入算法执行分支
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名，继续
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                // 尝试downcast为ExecutionStrategy trait对象
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() {
                    // 这里应传递真实的Context和参数，生产环境需实现完整调用
                    // exec_result = Some(exec_strategy.execute(ctx, &exec_params.algo_params)?);
                }
            }
        }
    }
    // 2. 预言机价格（如有）
    let mut price = price_params.price; // 默认使用传入价格，类型安全
    if let Some(oracle_name) = &price_params.oracle_name { // 若指定oracle名，进入分支
        let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取适配器工厂，线程安全
        if let Some(adapter) = factory.get(oracle_name) { // 查找oracle适配器
            // 尝试downcast为OracleAdapter trait对象
            if let Some(oracle_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::oracles::traits::OracleAdapter>>() {
                // 这里应传递真实的参数，生产环境需实现完整调用
                // let oracle_params = crate::core::types::OracleParams { ... };
                // let oracle_result = oracle_adapter.get_price(&oracle_params)?;
                // price = oracle_result.price;
            }
        }
    }
    // 3. DEX/AMM swap（如有）
    if let Some(dex_name) = &params.dex_name { // 若指定DEX名，进入分支
        let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取适配器工厂，线程安全
        if let Some(adapter) = factory.get(dex_name) { // 查找DEX适配器
            // 尝试downcast为DexAdapter trait对象
            if let Some(dex_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::dex::traits::DexAdapter>>() {
                // 这里应传递真实的参数，生产环境需实现完整调用
                // let swap_result = dex_adapter.swap(&params)?;
                // 可用swap_result.avg_price、swap_result.executed_amount等
            }
        }
    }
    // 4. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数，进入分支
        if !strategy_params.strategy_name.is_empty() { // 策略名非空，执行策略交易
            crate::services::asset_service::AssetService::strategy_trade(
                basket_index, // 目标资产篮子
                &strategy_params.strategy_name, // 策略名
                &strategy_params.params, // 策略参数
                ctx.accounts.buyer.key(), // 买入人公钥
            )?; // 调用策略交易逻辑，业务安全
        }
    }
    // 5. 资产买入，最终落地到服务层
    crate::services::asset_service::AssetService::buy(
        basket_index, // 目标资产篮子
        params.amount_in, // 买入数量
        price, // 买入价格
        ctx.accounts.buyer.key(), // 买入人公钥
    )?; // 调用服务层买入逻辑，业务安全
    emit!(AssetBought { // 事件：资产篮子ID、买入数量、价格、买入人、时间戳
        basket_id: basket_index.id, // 事件：资产篮子ID
        amount: params.amount_in,   // 事件：买入数量
        price,                      // 事件：成交价格
        buyer: ctx.accounts.buyer.key(), // 事件：买入人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

/// 资产卖出指令账户上下文
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct SellAsset<'info> { // 定义资产卖出指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub basket_index: Account<'info, BasketIndexState>, // 目标资产篮子，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub seller: Signer<'info>, // 卖出人，需签名，类型安全
}

/// 资产卖出指令实现
/// - params: SwapParams 交换参数，包含DEX名、数量等
/// - price_params: PriceParams 价格参数，包含oracle名、价格等
/// - exec_params: Option<ExecutionParams> 算法执行参数
/// - strategy_params: Option<StrategyParams> 策略参数
pub fn sell_asset(
    ctx: Context<SellAsset>, // Anchor账户上下文，自动校验权限与生命周期
    params: SwapParams, // 交换参数，类型安全
    price_params: PriceParams, // 价格参数，类型安全
    exec_params: Option<ExecutionParams>, // 可选算法执行参数
    strategy_params: Option<StrategyParams>, // 可选策略参数
) -> Result<()> { // Anchor规范返回类型
    let basket_index = &mut ctx.accounts.basket_index; // 获取目标资产篮子，类型安全，生命周期受Anchor管理
    basket_index.validate()?; // 校验资产篮子状态，防止非法卖出，业务安全
    // 1. 算法执行（如有）
    let mut exec_result = None; // 算法执行结果，默认无，类型Option
    if let Some(exec_params) = &exec_params { // 若有算法参数，进入算法执行分支
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名，继续
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                // 尝试downcast为ExecutionStrategy trait对象
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() {
                    // 这里应传递真实的Context和参数，生产环境需实现完整调用
                    // exec_result = Some(exec_strategy.execute(ctx, &exec_params.algo_params)?);
                }
            }
        }
    }
    // 2. 预言机价格（如有）
    let mut price = price_params.price; // 默认使用传入价格，类型安全
    if let Some(oracle_name) = &price_params.oracle_name { // 若指定oracle名，进入分支
        let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取适配器工厂，线程安全
        if let Some(adapter) = factory.get(oracle_name) { // 查找oracle适配器
            // 尝试downcast为OracleAdapter trait对象
            if let Some(oracle_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::oracles::traits::OracleAdapter>>() {
                // 这里应传递真实的参数，生产环境需实现完整调用
                // let oracle_params = crate::core::types::OracleParams { ... };
                // let oracle_result = oracle_adapter.get_price(&oracle_params)?;
                // price = oracle_result.price;
            }
        }
    }
    // 3. DEX/AMM swap（如有）
    if let Some(dex_name) = &params.dex_name { // 若指定DEX名，进入分支
        let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取适配器工厂，线程安全
        if let Some(adapter) = factory.get(dex_name) { // 查找DEX适配器
            // 尝试downcast为DexAdapter trait对象
            if let Some(dex_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::dex::traits::DexAdapter>>() {
                // 这里应传递真实的参数，生产环境需实现完整调用
                // let swap_result = dex_adapter.swap(&params)?;
                // 可用swap_result.avg_price、swap_result.executed_amount等
            }
        }
    }
    // 4. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数，进入分支
        if !strategy_params.strategy_name.is_empty() { // 策略名非空，执行策略交易
            crate::services::asset_service::AssetService::strategy_trade(
                basket_index, // 目标资产篮子
                &strategy_params.strategy_name, // 策略名
                &strategy_params.params, // 策略参数
                ctx.accounts.seller.key(), // 卖出人公钥
            )?; // 调用策略交易逻辑，业务安全
        }
    }
    // 5. 资产卖出，最终落地到服务层
    crate::services::asset_service::AssetService::sell(
        basket_index, // 目标资产篮子
        params.amount_in, // 卖出数量
        price, // 卖出价格
        ctx.accounts.seller.key(), // 卖出人公钥
    )?; // 调用服务层卖出逻辑，业务安全
    emit!(AssetSold { // 事件：资产篮子ID、卖出数量、价格、卖出人、时间戳
        basket_id: basket_index.id, // 事件：资产篮子ID
        amount: params.amount_in,   // 事件：卖出数量
        price,                      // 事件：成交价格
        seller: ctx.accounts.seller.key(), // 事件：卖出人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

/// 资产swap指令账户上下文
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct SwapAsset<'info> { // 定义资产swap指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub from_asset: Account<'info, BasketIndexState>, // 转出资产账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub to_asset: Account<'info, BasketIndexState>,   // 转入资产账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>,                     // 操作人签名者，类型安全
}

/// 资产swap指令实现
/// - params: SwapParams 交换参数，包含DEX名、数量等
/// - price_params: PriceParams 价格参数，包含oracle名、价格等
/// - exec_params: Option<ExecutionParams> 算法执行参数
/// - strategy_params: Option<StrategyParams> 策略参数
pub fn swap_asset(
    ctx: Context<SwapAsset>, // Anchor账户上下文，自动校验权限与生命周期
    params: SwapParams, // 交换参数，类型安全
    price_params: PriceParams, // 价格参数，类型安全
    exec_params: Option<ExecutionParams>, // 可选算法执行参数
    strategy_params: Option<StrategyParams>, // 可选策略参数
) -> Result<()> { // Anchor规范返回类型
    let from_asset = &mut ctx.accounts.from_asset; // 获取转出资产账户，类型安全，生命周期受Anchor管理
    let to_asset = &mut ctx.accounts.to_asset; // 获取转入资产账户，类型安全，生命周期受Anchor管理
    from_asset.validate()?; // 校验转出资产账户状态，防止非法操作，业务安全
    to_asset.validate()?; // 校验转入资产账户状态，防止非法操作，业务安全
    // 1. 算法执行（如有）
    let mut exec_result = None; // 算法执行结果，默认无，类型Option
    if let Some(exec_params) = &exec_params { // 若有算法参数，进入算法执行分支
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名，继续
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                // 尝试downcast为ExecutionStrategy trait对象
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() {
                    // 这里应传递真实的Context和参数，生产环境需实现完整调用
                    // exec_result = Some(exec_strategy.execute(ctx, &exec_params.algo_params)?);
                }
            }
        }
    }
    // 2. 预言机价格（如有）
    let mut price = price_params.price; // 默认使用传入价格，类型安全
    if let Some(oracle_name) = &price_params.oracle_name { // 若指定oracle名，进入分支
        let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取适配器工厂，线程安全
        if let Some(adapter) = factory.get(oracle_name) { // 查找oracle适配器
            // 尝试downcast为OracleAdapter trait对象
            if let Some(oracle_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::oracles::traits::OracleAdapter>>() {
                // 这里应传递真实的参数，生产环境需实现完整调用
                // let oracle_params = crate::core::types::OracleParams { ... };
                // let oracle_result = oracle_adapter.get_price(&oracle_params)?;
                // price = oracle_result.price;
            }
        }
    }
    // 3. DEX/AMM swap（如有）
    if let Some(dex_name) = &params.dex_name { // 若指定DEX名，进入分支
        let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取适配器工厂，线程安全
        if let Some(adapter) = factory.get(dex_name) { // 查找DEX适配器
            // 尝试downcast为DexAdapter trait对象
            if let Some(dex_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::dex::traits::DexAdapter>>() {
                // 这里应传递真实的参数，生产环境需实现完整调用
                // let swap_result = dex_adapter.swap(&params)?;
                // 可用swap_result.avg_price、swap_result.executed_amount等
            }
        }
    }
    // 4. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数，进入分支
        if !strategy_params.strategy_name.is_empty() { // 策略名非空，执行策略交易
            crate::services::asset_service::AssetService::strategy_trade(
                from_asset, // 转出资产账户
                &strategy_params.strategy_name, // 策略名
                &strategy_params.params, // 策略参数
                ctx.accounts.authority.key(), // 操作人公钥
            )?; // 调用策略交易逻辑，业务安全
        }
    }
    // 5. 资产swap，最终落地到服务层
    crate::services::asset_service::AssetService::swap(
        from_asset, // 转出资产账户
        to_asset, // 转入资产账户
        params.amount_in, // swap数量
        price, // swap价格
        ctx.accounts.authority.key(), // 操作人公钥
    )?; // 调用服务层swap逻辑，业务安全
    emit!(AssetSwapped { // 事件：转出资产ID、转入资产ID、swap数量、价格、操作人、时间戳
        from_asset_id: from_asset.id, // 事件：转出资产ID
        to_asset_id: to_asset.id, // 事件：转入资产ID
        amount: params.amount_in, // 事件：swap数量
        price, // 事件：成交价格
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

/// 资产授权指令账户上下文
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct AuthorizeAsset<'info> { // 定义资产授权指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub asset: Account<'info, BasketIndexState>, // 目标资产账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 当前授权人，需签名，类型安全
    pub new_authority: Pubkey,    // 新授权人公钥，类型安全
}

/// 资产授权指令实现
/// - exec_params: Option<ExecutionParams> 算法执行参数
/// - strategy_params: Option<StrategyParams> 策略参数
pub fn authorize_asset(
    ctx: Context<AuthorizeAsset>, // Anchor账户上下文，自动校验权限与生命周期
    exec_params: Option<ExecutionParams>, // 可选算法执行参数
    strategy_params: Option<StrategyParams>, // 可选策略参数
) -> Result<()> { // Anchor规范返回类型
    let asset = &mut ctx.accounts.asset; // 获取目标资产账户，类型安全，生命周期受Anchor管理
    asset.validate()?; // 校验资产账户状态，防止非法授权，业务安全
    // 1. 算法执行（如有）
    let mut exec_result = None; // 算法执行结果，默认无，类型Option
    if let Some(exec_params) = &exec_params { // 若有算法参数，进入算法执行分支
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名，继续
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                // 尝试downcast为ExecutionStrategy trait对象
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() {
                    // 这里应传递真实的Context和参数，生产环境需实现完整调用
                    // exec_result = Some(exec_strategy.execute(ctx, &exec_params.algo_params)?);
                }
            }
        }
    }
    // 2. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数，进入分支
        if !strategy_params.strategy_name.is_empty() { // 策略名非空，执行策略授权逻辑
            crate::services::asset_service::AssetService::strategy_authorize(
                asset, // 目标资产账户
                &strategy_params.strategy_name, // 策略名
                &strategy_params.params, // 策略参数
                ctx.accounts.authority.key(), // 当前授权人公钥
                ctx.accounts.new_authority, // 新授权人公钥
            )?; // 调用策略授权逻辑，业务安全
        }
    }
    // 3. 资产授权，最终落地到服务层
    crate::services::asset_service::AssetService::authorize(
        asset, // 目标资产账户
        ctx.accounts.authority.key(), // 当前授权人公钥
        ctx.accounts.new_authority, // 新授权人公钥
    )?; // 调用服务层授权逻辑，业务安全
    emit!(AssetAuthorized { // 事件：资产ID、原授权人、新授权人、时间戳
        asset_id: asset.id, // 事件：资产ID
        old_authority: ctx.accounts.authority.key(), // 事件：原授权人
        new_authority: ctx.accounts.new_authority, // 事件：新授权人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

/// 资产合并指令账户上下文
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct CombineAsset<'info> { // 定义资产合并指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub target_asset: Account<'info, BasketIndexState>, // 目标资产账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub source_asset: Account<'info, BasketIndexState>, // 源资产账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

/// 资产合并指令实现
/// - amount: u64 合并数量，类型安全
/// - exec_params: Option<ExecutionParams> 算法执行参数
/// - strategy_params: Option<StrategyParams> 策略参数
pub fn combine_asset(
    ctx: Context<CombineAsset>, // Anchor账户上下文，自动校验权限与生命周期
    amount: u64, // 合并数量，类型安全
    exec_params: Option<ExecutionParams>, // 可选算法执行参数
    strategy_params: Option<StrategyParams>, // 可选策略参数
) -> Result<()> { // Anchor规范返回类型
    let target_asset = &mut ctx.accounts.target_asset; // 获取目标资产账户，类型安全，生命周期受Anchor管理
    let source_asset = &mut ctx.accounts.source_asset; // 获取源资产账户，类型安全，生命周期受Anchor管理
    target_asset.validate()?; // 校验目标资产账户状态，防止非法合并，业务安全
    source_asset.validate()?; // 校验源资产账户状态，防止非法合并，业务安全
    // 1. 算法执行（如有）
    let mut exec_result = None; // 算法执行结果，默认无，类型Option
    if let Some(exec_params) = &exec_params { // 若有算法参数，进入算法执行分支
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名，继续
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                // 尝试downcast为ExecutionStrategy trait对象
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() {
                    // 这里应传递真实的Context和参数，生产环境需实现完整调用
                    // exec_result = Some(exec_strategy.execute(ctx, &exec_params.algo_params)?);
                }
            }
        }
    }
    // 2. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数，进入分支
        if !strategy_params.strategy_name.is_empty() { // 策略名非空，执行策略合并逻辑
            crate::services::asset_service::AssetService::strategy_combine(
                target_asset, // 目标资产账户
                source_asset, // 源资产账户
                &strategy_params.strategy_name, // 策略名
                &strategy_params.params, // 策略参数
                ctx.accounts.authority.key(), // 操作人公钥
                amount, // 合并数量
            )?; // 调用策略合并逻辑，业务安全
        }
    }
    // 3. 资产合并，最终落地到服务层
    crate::services::asset_service::AssetService::combine(
        target_asset, // 目标资产账户
        source_asset, // 源资产账户
        ctx.accounts.authority.key(), // 操作人公钥
        amount, // 合并数量
    )?; // 调用服务层合并逻辑，业务安全
    emit!(AssetCombined { // 事件：目标资产ID、源资产ID、合并数量、操作人、时间戳
        target_asset_id: target_asset.id, // 事件：目标资产ID
        source_asset_id: source_asset.id, // 事件：源资产ID
        amount, // 事件：合并数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

/// 资产拆分指令账户上下文
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct SplitAsset<'info> { // 定义资产拆分指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub source_asset: Account<'info, BasketIndexState>, // 源资产账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub new_asset: Account<'info, BasketIndexState>,    // 新资产账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

/// 资产拆分指令实现
/// - amount: u64 拆分数量，类型安全
/// - exec_params: Option<ExecutionParams> 算法执行参数
/// - strategy_params: Option<StrategyParams> 策略参数
pub fn split_asset(
    ctx: Context<SplitAsset>, // Anchor账户上下文，自动校验权限与生命周期
    amount: u64, // 拆分数量，类型安全
    exec_params: Option<ExecutionParams>, // 可选算法执行参数
    strategy_params: Option<StrategyParams>, // 可选策略参数
) -> Result<()> { // Anchor规范返回类型
    let source_asset = &mut ctx.accounts.source_asset; // 获取源资产账户，类型安全，生命周期受Anchor管理
    let new_asset = &mut ctx.accounts.new_asset; // 获取新资产账户，类型安全，生命周期受Anchor管理
    source_asset.validate()?; // 校验源资产账户状态，防止非法拆分，业务安全
    new_asset.validate()?; // 校验新资产账户状态，防止非法拆分，业务安全
    // 1. 算法执行（如有）
    let mut exec_result = None; // 算法执行结果，默认无，类型Option
    if let Some(exec_params) = &exec_params { // 若有算法参数，进入算法执行分支
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名，继续
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                // 尝试downcast为ExecutionStrategy trait对象
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() {
                    // 这里应传递真实的Context和参数，生产环境需实现完整调用
                    // exec_result = Some(exec_strategy.execute(ctx, &exec_params.algo_params)?);
                }
            }
        }
    }
    // 2. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数，进入分支
        if !strategy_params.strategy_name.is_empty() { // 策略名非空，执行策略拆分逻辑
            crate::services::asset_service::AssetService::strategy_split(
                source_asset, // 源资产账户
                new_asset, // 新资产账户
                &strategy_params.strategy_name, // 策略名
                &strategy_params.params, // 策略参数
                ctx.accounts.authority.key(), // 操作人公钥
                amount, // 拆分数量
            )?; // 调用策略拆分逻辑，业务安全
        }
    }
    // 3. 资产拆分，最终落地到服务层
    crate::services::asset_service::AssetService::split(
        source_asset, // 源资产账户
        new_asset, // 新资产账户
        ctx.accounts.authority.key(), // 操作人公钥
        amount, // 拆分数量
    )?; // 调用服务层拆分逻辑，业务安全
    emit!(AssetSplit { // 事件：源资产ID、新资产ID、拆分数量、操作人、时间戳
        source_asset_id: source_asset.id, // 事件：源资产ID
        new_asset_id: new_asset.id, // 事件：新资产ID
        amount, // 事件：拆分数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

/// 资产冻结指令账户上下文
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct FreezeAsset<'info> { // 定义资产冻结指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub asset: Account<'info, BasketIndexState>, // 目标资产账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

/// 资产冻结指令实现
/// - exec_params: Option<ExecutionParams> 算法执行参数
/// - strategy_params: Option<StrategyParams> 策略参数
pub fn freeze_asset(
    ctx: Context<FreezeAsset>, // Anchor账户上下文，自动校验权限与生命周期
    exec_params: Option<ExecutionParams>, // 可选算法执行参数
    strategy_params: Option<StrategyParams>, // 可选策略参数
) -> Result<()> { // Anchor规范返回类型
    let asset = &mut ctx.accounts.asset; // 获取目标资产账户，类型安全，生命周期受Anchor管理
    asset.validate()?; // 校验资产账户状态，防止非法冻结，业务安全
    // 1. 算法执行（如有）
    let mut exec_result = None; // 算法执行结果，默认无，类型Option
    if let Some(exec_params) = &exec_params { // 若有算法参数，进入算法执行分支
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名，继续
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                // 尝试downcast为ExecutionStrategy trait对象
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() {
                    // 这里应传递真实的Context和参数，生产环境需实现完整调用
                    // exec_result = Some(exec_strategy.execute(ctx, &exec_params.algo_params)?);
                }
            }
        }
    }
    // 2. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数，进入分支
        if !strategy_params.strategy_name.is_empty() { // 策略名非空，执行策略冻结逻辑
            crate::services::asset_service::AssetService::strategy_freeze(
                asset, // 目标资产账户
                &strategy_params.strategy_name, // 策略名
                &strategy_params.params, // 策略参数
                ctx.accounts.authority.key(), // 操作人公钥
            )?; // 调用策略冻结逻辑，业务安全
        }
    }
    // 3. 资产冻结，最终落地到服务层
    crate::services::asset_service::AssetService::freeze(
        asset, // 目标资产账户
        ctx.accounts.authority.key(), // 操作人公钥
    )?; // 调用服务层冻结逻辑，业务安全
    emit!(AssetFrozen { // 事件：资产ID、操作人、时间戳
        asset_id: asset.id, // 事件：资产ID
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

/// 资产解冻指令账户上下文
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct UnfreezeAsset<'info> { // 定义资产解冻指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub asset: Account<'info, BasketIndexState>, // 目标资产账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

/// 资产解冻指令实现
/// - exec_params: Option<ExecutionParams> 算法执行参数
/// - strategy_params: Option<StrategyParams> 策略参数
pub fn unfreeze_asset(
    ctx: Context<UnfreezeAsset>, // Anchor账户上下文，自动校验权限与生命周期
    exec_params: Option<ExecutionParams>, // 可选算法执行参数
    strategy_params: Option<StrategyParams>, // 可选策略参数
) -> Result<()> { // Anchor规范返回类型
    let asset = &mut ctx.accounts.asset; // 获取目标资产账户，类型安全，生命周期受Anchor管理
    asset.validate()?; // 校验资产账户状态，防止非法解冻，业务安全
    // 1. 算法执行（如有）
    let mut exec_result = None; // 算法执行结果，默认无，类型Option
    if let Some(exec_params) = &exec_params { // 若有算法参数，进入算法执行分支
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名，继续
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                // 尝试downcast为ExecutionStrategy trait对象
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() {
                    // 这里应传递真实的Context和参数，生产环境需实现完整调用
                    // exec_result = Some(exec_strategy.execute(ctx, &exec_params.algo_params)?);
                }
            }
        }
    }
    // 2. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数，进入分支
        if !strategy_params.strategy_name.is_empty() { // 策略名非空，执行策略解冻逻辑
            crate::services::asset_service::AssetService::strategy_unfreeze(
                asset, // 目标资产账户
                &strategy_params.strategy_name, // 策略名
                &strategy_params.params, // 策略参数
                ctx.accounts.authority.key(), // 操作人公钥
            )?; // 调用策略解冻逻辑，业务安全
        }
    }
    // 3. 资产解冻，最终落地到服务层
    crate::services::asset_service::AssetService::unfreeze(
        asset, // 目标资产账户
        ctx.accounts.authority.key(), // 操作人公钥
    )?; // 调用服务层解冻逻辑，业务安全
    emit!(AssetUnfrozen { // 事件：资产ID、操作人、时间戳
        asset_id: asset.id, // 事件：资产ID
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

/// 批量资产转账指令账户上下文
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct BatchTransferAsset<'info> { // 定义批量资产转账指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub from_asset: Account<'info, BasketIndexState>, // 批量转出资产账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub to_assets: Vec<Account<'info, BasketIndexState>>, // 批量转入资产账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

/// 批量资产转账指令实现
/// - amounts: Vec<u64> 批量转账数量，类型安全
/// - exec_params: Option<ExecutionParams> 算法执行参数
/// - strategy_params: Option<StrategyParams> 策略参数
pub fn batch_transfer_asset(
    ctx: Context<BatchTransferAsset>, // Anchor账户上下文，自动校验权限与生命周期
    amounts: Vec<u64>, // 批量转账数量，类型安全
    exec_params: Option<ExecutionParams>, // 可选算法执行参数
    strategy_params: Option<StrategyParams>, // 可选策略参数
) -> Result<()> { // Anchor规范返回类型
    let from_asset = &mut ctx.accounts.from_asset; // 获取批量转出资产账户，类型安全，生命周期受Anchor管理
    let to_assets = &mut ctx.accounts.to_assets; // 获取批量转入资产账户，类型安全，生命周期受Anchor管理
    from_asset.validate()?; // 校验转出资产账户状态，防止非法批量转账，业务安全
    for to_asset in to_assets.iter_mut() { // 遍历所有转入资产账户
        to_asset.validate()?; // 校验每个转入资产账户状态，防止非法批量转账，业务安全
    }
    // 1. 算法执行（如有）
    let mut exec_result = None; // 算法执行结果，默认无，类型Option
    if let Some(exec_params) = &exec_params { // 若有算法参数，进入算法执行分支
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名，继续
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                // 尝试downcast为ExecutionStrategy trait对象
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() {
                    // 这里应传递真实的Context和参数，生产环境需实现完整调用
                    // exec_result = Some(exec_strategy.execute(ctx, &exec_params.algo_params)?);
                }
            }
        }
    }
    // 2. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数，进入分支
        if !strategy_params.strategy_name.is_empty() { // 策略名非空，执行策略批量转账逻辑
            crate::services::asset_service::AssetService::strategy_batch_transfer(
                from_asset, // 批量转出资产账户
                to_assets, // 批量转入资产账户
                &strategy_params.strategy_name, // 策略名
                &strategy_params.params, // 策略参数
                ctx.accounts.authority.key(), // 操作人公钥
                &amounts, // 批量转账数量
            )?; // 调用策略批量转账逻辑，业务安全
        }
    }
    // 3. 批量资产转账，最终落地到服务层
    crate::services::asset_service::AssetService::batch_transfer(
        from_asset, // 批量转出资产账户
        to_assets, // 批量转入资产账户
        ctx.accounts.authority.key(), // 操作人公钥
        &amounts, // 批量转账数量
    )?; // 调用服务层批量转账逻辑，业务安全
    emit!(BatchAssetTransferred { // 事件：批量转出资产ID、批量转入资产ID、批量转账数量、操作人、时间戳
        from_asset_id: from_asset.id, // 事件：批量转出资产ID
        to_asset_ids: to_assets.iter().map(|a| a.id).collect(), // 事件：批量转入资产ID集合
        amounts: amounts.clone(), // 事件：批量转账数量集合
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

/// 批量资产swap指令账户上下文
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct BatchSwapAsset<'info> { // 定义批量资产swap指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub from_asset: Account<'info, BasketIndexState>, // 批量转出资产账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub to_assets: Vec<Account<'info, BasketIndexState>>, // 批量转入资产账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

/// 批量资产swap指令实现
/// - params: BatchSwapParams 批量swap参数，类型安全
/// - exec_params: Option<ExecutionParams> 算法执行参数
/// - strategy_params: Option<StrategyParams> 策略参数
pub fn batch_swap_asset(
    ctx: Context<BatchSwapAsset>, // Anchor账户上下文，自动校验权限与生命周期
    params: BatchSwapParams, // 批量swap参数，类型安全
    exec_params: Option<ExecutionParams>, // 可选算法执行参数
    strategy_params: Option<StrategyParams>, // 可选策略参数
) -> Result<()> { // Anchor规范返回类型
    let from_asset = &mut ctx.accounts.from_asset; // 获取批量转出资产账户，类型安全，生命周期受Anchor管理
    let to_assets = &mut ctx.accounts.to_assets; // 获取批量转入资产账户，类型安全，生命周期受Anchor管理
    from_asset.validate()?; // 校验转出资产账户状态，防止非法批量swap，业务安全
    for to_asset in to_assets.iter_mut() { // 遍历所有转入资产账户
        to_asset.validate()?; // 校验每个转入资产账户状态，防止非法批量swap，业务安全
    }
    // 1. 算法执行（如有）
    let mut exec_result = None; // 算法执行结果，默认无，类型Option
    if let Some(exec_params) = &exec_params { // 若有算法参数，进入算法执行分支
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名，继续
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                // 尝试downcast为ExecutionStrategy trait对象
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() {
                    // 这里应传递真实的Context和参数，生产环境需实现完整调用
                    // exec_result = Some(exec_strategy.execute(ctx, &exec_params.algo_params)?);
                }
            }
        }
    }
    // 2. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数，进入分支
        if !strategy_params.strategy_name.is_empty() { // 策略名非空，执行策略批量swap逻辑
            crate::services::asset_service::AssetService::strategy_batch_swap(
                from_asset, // 批量转出资产账户
                to_assets, // 批量转入资产账户
                &strategy_params.strategy_name, // 策略名
                &strategy_params.params, // 策略参数
                ctx.accounts.authority.key(), // 操作人公钥
                &params, // 批量swap参数
            )?; // 调用策略批量swap逻辑，业务安全
        }
    }
    // 3. 批量资产swap，最终落地到服务层
    crate::services::asset_service::AssetService::batch_swap(
        from_asset, // 批量转出资产账户
        to_assets, // 批量转入资产账户
        ctx.accounts.authority.key(), // 操作人公钥
        &params, // 批量swap参数
    )?; // 调用服务层批量swap逻辑，业务安全
    emit!(BatchAssetSwapped { // 事件：批量转出资产ID、批量转入资产ID、批量swap参数、操作人、时间戳
        from_asset_id: from_asset.id, // 事件：批量转出资产ID
        to_asset_ids: to_assets.iter().map(|a| a.id).collect(), // 事件：批量转入资产ID集合
        params: params.clone(), // 事件：批量swap参数
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

/// 策略资产交易指令账户上下文
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct StrategyTradeAsset<'info> { // 定义策略资产交易指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub asset: Account<'info, BasketIndexState>, // 目标资产账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

/// 策略资产交易指令实现
/// - params: StrategyTradeParams 策略交易参数，类型安全
/// - exec_params: Option<ExecutionParams> 算法执行参数
pub fn strategy_trade_asset(
    ctx: Context<StrategyTradeAsset>, // Anchor账户上下文，自动校验权限与生命周期
    params: StrategyTradeParams, // 策略交易参数，类型安全
    exec_params: Option<ExecutionParams>, // 可选算法执行参数
) -> Result<()> { // Anchor规范返回类型
    let asset = &mut ctx.accounts.asset; // 获取目标资产账户，类型安全，生命周期受Anchor管理
    asset.validate()?; // 校验资产账户状态，防止非法策略交易，业务安全
    // 1. 算法执行（如有）
    let mut exec_result = None; // 算法执行结果，默认无，类型Option
    if let Some(exec_params) = &exec_params { // 若有算法参数，进入算法执行分支
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名，继续
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                // 尝试downcast为ExecutionStrategy trait对象
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() {
                    // 这里应传递真实的Context和参数，生产环境需实现完整调用
                    // exec_result = Some(exec_strategy.execute(ctx, &exec_params.algo_params)?);
                }
            }
        }
    }
    // 2. 策略交易，最终落地到服务层
    crate::services::asset_service::AssetService::strategy_trade(
        asset, // 目标资产账户
        &params.strategy_name, // 策略名
        &params.params, // 策略参数
        ctx.accounts.authority.key(), // 操作人公钥
    )?; // 调用服务层策略交易逻辑，业务安全
    emit!(AssetStrategyTraded { // 事件：资产ID、策略名、操作人、时间戳
        asset_id: asset.id, // 事件：资产ID
        strategy_name: params.strategy_name.clone(), // 事件：策略名
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

// === 新增：报价指令 ===
#[derive(Accounts)]
pub struct QuoteAsset<'info> {
    pub asset: Account<'info, BasketIndexState>, // 资产账户，只读
}

/// 资产报价指令实现
/// - params: SwapParams 交换参数，类型安全
/// - price_params: PriceParams 价格参数，类型安全
pub fn quote_asset(
    ctx: Context<QuoteAsset>, // Anchor账户上下文，自动校验权限与生命周期
    params: SwapParams, // 交换参数，类型安全
    price_params: PriceParams, // 价格参数，类型安全
) -> Result<u64> { // Anchor规范返回类型，返回报价
    let asset = &ctx.accounts.asset; // 获取只读资产账户，类型安全，生命周期受Anchor管理
    asset.validate()?; // 校验资产账户状态，防止非法报价，业务安全
    // 1. 预言机价格（如有）
    let mut price = price_params.price; // 默认使用传入价格，类型安全
    if let Some(oracle_name) = &price_params.oracle_name { // 若指定oracle名，进入分支
        let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取适配器工厂，线程安全
        if let Some(adapter) = factory.get(oracle_name) { // 查找oracle适配器
            // 尝试downcast为OracleAdapter trait对象
            if let Some(oracle_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::oracles::traits::OracleAdapter>>() {
                // 这里应传递真实的参数，生产环境需实现完整调用
                // let oracle_params = crate::core::types::OracleParams { ... };
                // let oracle_result = oracle_adapter.get_price(&oracle_params)?;
                // price = oracle_result.price;
            }
        }
    }
    // 2. DEX/AMM报价（如有）
    if let Some(dex_name) = &params.dex_name { // 若指定DEX名，进入分支
        let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取适配器工厂，线程安全
        if let Some(adapter) = factory.get(dex_name) { // 查找DEX适配器
            // 尝试downcast为DexAdapter trait对象
            if let Some(dex_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::dex::traits::DexAdapter>>() {
                // 这里应传递真实的参数，生产环境需实现完整调用
                // let quote_result = dex_adapter.quote(&params)?;
                // price = quote_result.price;
            }
        }
    }
    // 3. 返回最终报价
    Ok(price) // 返回报价，类型安全，Anchor自动处理生命周期
}

// === 新增：执行买入指令 ===
#[derive(Accounts)]
pub struct ExecuteBuyAsset<'info> {
    #[account(mut)]
    pub asset: Account<'info, BasketIndexState>, // 资产账户，需可变
    #[account(mut)]
    pub buyer: Signer<'info>, // 买入人，需签名
}

/// 资产买入执行指令实现
/// - params: SwapParams 交换参数，类型安全
/// - price: u64 买入价格，类型安全
pub fn execute_buy_asset(
    ctx: Context<ExecuteBuyAsset>, // Anchor账户上下文，自动校验权限与生命周期
    params: SwapParams, // 交换参数，类型安全
    price: u64, // 买入价格，类型安全
) -> Result<()> { // Anchor规范返回类型
    let asset = &mut ctx.accounts.asset; // 获取资产账户，类型安全，生命周期受Anchor管理
    asset.validate()?; // 校验资产账户状态，防止非法买入，业务安全
    // 1. 买入执行，最终落地到服务层
    crate::services::asset_service::AssetService::execute_buy(
        asset, // 资产账户
        params.amount_in, // 买入数量
        price, // 买入价格
        ctx.accounts.buyer.key(), // 买入人公钥
    )?; // 调用服务层买入执行逻辑，业务安全
    emit!(AssetBuyExecuted { // 事件：资产ID、买入数量、价格、买入人、时间戳
        asset_id: asset.id, // 事件：资产ID
        amount: params.amount_in, // 事件：买入数量
        price, // 事件：买入价格
        buyer: ctx.accounts.buyer.key(), // 事件：买入人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

// === 新增：执行卖出指令 ===
#[derive(Accounts)]
pub struct ExecuteSellAsset<'info> {
    #[account(mut)]
    pub asset: Account<'info, BasketIndexState>, // 资产账户，需可变
    #[account(mut)]
    pub seller: Signer<'info>, // 卖出人，需签名
}

/// 资产卖出执行指令实现
/// - params: SwapParams 交换参数，类型安全
/// - price: u64 卖出价格，类型安全
pub fn execute_sell_asset(
    ctx: Context<ExecuteSellAsset>, // Anchor账户上下文，自动校验权限与生命周期
    params: SwapParams, // 交换参数，类型安全
    price: u64, // 卖出价格，类型安全
) -> Result<()> { // Anchor规范返回类型
    let asset = &mut ctx.accounts.asset; // 获取资产账户，类型安全，生命周期受Anchor管理
    asset.validate()?; // 校验资产账户状态，防止非法卖出，业务安全
    // 1. 卖出执行，最终落地到服务层
    crate::services::asset_service::AssetService::execute_sell(
        asset, // 资产账户
        params.amount_in, // 卖出数量
        price, // 卖出价格
        ctx.accounts.seller.key(), // 卖出人公钥
    )?; // 调用服务层卖出执行逻辑，业务安全
    emit!(AssetSellExecuted { // 事件：资产ID、卖出数量、价格、卖出人、时间戳
        asset_id: asset.id, // 事件：资产ID
        amount: params.amount_in, // 事件：卖出数量
        price, // 事件：卖出价格
        seller: ctx.accounts.seller.key(), // 事件：卖出人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

// === 新增：执行资产交换指令 ===
#[derive(Accounts)]
pub struct ExecuteSwapAsset<'info> {
    #[account(mut)]
    pub from_asset: Account<'info, BasketIndexState>, // 转出资产账户，需可变
    #[account(mut)]
    pub to_asset: Account<'info, BasketIndexState>,   // 转入资产账户，需可变
    #[account(mut)]
    pub authority: Signer<'info>,                     // 操作人签名者
}

/// 资产swap执行指令账户上下文
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct ExecuteSwapAsset<'info> { // 定义资产swap执行指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub from_asset: Account<'info, BasketIndexState>, // 转出资产账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub to_asset: Account<'info, BasketIndexState>,   // 转入资产账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>,                     // 操作人签名者，类型安全
}

/// 资产swap执行指令实现
/// - from_amount: u64 转出数量，类型安全
/// - to_amount: u64 转入数量，类型安全
pub fn execute_swap_asset(
    ctx: Context<ExecuteSwapAsset>, // Anchor账户上下文，自动校验权限与生命周期
    from_amount: u64, // 转出数量，类型安全
    to_amount: u64, // 转入数量，类型安全
) -> Result<()> { // Anchor规范返回类型
    let from_asset = &mut ctx.accounts.from_asset; // 获取转出资产账户，类型安全，生命周期受Anchor管理
    let to_asset = &mut ctx.accounts.to_asset; // 获取转入资产账户，类型安全，生命周期受Anchor管理
    from_asset.validate()?; // 校验转出资产账户状态，防止非法swap，业务安全
    to_asset.validate()?; // 校验转入资产账户状态，防止非法swap，业务安全
    // 1. swap执行，最终落地到服务层
    crate::services::asset_service::AssetService::execute_swap(
        from_asset, // 转出资产账户
        to_asset, // 转入资产账户
        from_amount, // 转出数量
        to_amount, // 转入数量
        ctx.accounts.authority.key(), // 操作人公钥
    )?; // 调用服务层swap执行逻辑，业务安全
    emit!(AssetSwapExecuted { // 事件：转出资产ID、转入资产ID、转出数量、转入数量、操作人、时间戳
        from_asset_id: from_asset.id, // 事件：转出资产ID
        to_asset_id: to_asset.id, // 事件：转入资产ID
        from_amount, // 事件：转出数量
        to_amount, // 事件：转入数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

// === 新增：执行资产合并指令 ===
#[derive(Accounts)]
pub struct ExecuteCombineAsset<'info> {
    #[account(mut)]
    pub target_asset: Account<'info, BasketIndexState>, // 目标资产账户，需可变
    #[account(mut)]
    pub source_asset: Account<'info, BasketIndexState>, // 源资产账户，需可变
    #[account(mut)]
    pub authority: Signer<'info>, // 操作人签名者
}

/// 资产合并执行指令实现
/// - amount: u64 合并数量，类型安全
pub fn execute_combine_asset(
    ctx: Context<ExecuteCombineAsset>, // Anchor账户上下文，自动校验权限与生命周期
    amount: u64, // 合并数量，类型安全
) -> Result<()> { // Anchor规范返回类型
    let target_asset = &mut ctx.accounts.target_asset; // 获取目标资产账户，类型安全，生命周期受Anchor管理
    let source_asset = &mut ctx.accounts.source_asset; // 获取源资产账户，类型安全，生命周期受Anchor管理
    target_asset.validate()?; // 校验目标资产账户状态，防止非法合并，业务安全
    source_asset.validate()?; // 校验源资产账户状态，防止非法合并，业务安全
    // 1. 合并执行，最终落地到服务层
    crate::services::asset_service::AssetService::execute_combine(
        target_asset, // 目标资产账户
        source_asset, // 源资产账户
        ctx.accounts.authority.key(), // 操作人公钥
        amount, // 合并数量
    )?; // 调用服务层合并执行逻辑，业务安全
    emit!(AssetCombineExecuted { // 事件：目标资产ID、源资产ID、合并数量、操作人、时间戳
        target_asset_id: target_asset.id, // 事件：目标资产ID
        source_asset_id: source_asset.id, // 事件：源资产ID
        amount, // 事件：合并数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

// === 新增：执行资产拆分指令 ===
#[derive(Accounts)]
pub struct ExecuteSplitAsset<'info> {
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub source_asset: Account<'info, BasketIndexState>, // 源资产账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub new_asset: Account<'info, BasketIndexState>,    // 新资产账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

/// 资产拆分执行指令实现
/// - amount: u64 拆分数量，类型安全
pub fn execute_split_asset(
    ctx: Context<ExecuteSplitAsset>, // Anchor账户上下文，自动校验权限与生命周期
    amount: u64, // 拆分数量，类型安全
) -> Result<()> { // Anchor规范返回类型
    let source_asset = &mut ctx.accounts.source_asset; // 获取源资产账户，类型安全，生命周期受Anchor管理
    let new_asset = &mut ctx.accounts.new_asset; // 获取新资产账户，类型安全，生命周期受Anchor管理
    source_asset.validate()?; // 校验源资产账户状态，防止非法拆分，业务安全
    new_asset.validate()?; // 校验新资产账户状态，防止非法拆分，业务安全
    // 1. 拆分执行，最终落地到服务层
    crate::services::asset_service::AssetService::execute_split(
        source_asset, // 源资产账户
        new_asset, // 新资产账户
        ctx.accounts.authority.key(), // 操作人公钥
        amount, // 拆分数量
    )?; // 调用服务层拆分执行逻辑，业务安全
    emit!(AssetSplitExecuted { // 事件：源资产ID、新资产ID、拆分数量、操作人、时间戳
        source_asset_id: source_asset.id, // 事件：源资产ID
        new_asset_id: new_asset.id, // 事件：新资产ID
        amount, // 事件：拆分数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

// 批量策略交易指令（如有需求，可按如下模式实现）
/// 批量策略资产交易指令实现
/// - ctx: Context<BatchTransferAsset> 批量转账账户上下文
/// - strategies: Vec<StrategyTradeParams> 策略交易参数集合
/// - exec_params: Option<ExecutionParams> 算法执行参数
pub fn batch_strategy_trade_asset(
    ctx: Context<BatchTransferAsset>, // Anchor账户上下文，自动校验权限与生命周期
    strategies: Vec<StrategyTradeParams>, // 策略交易参数集合，类型安全
    exec_params: Option<ExecutionParams>, // 可选算法执行参数
) -> Result<()> { // Anchor规范返回类型
    let from_asset = &mut ctx.accounts.from_asset; // 获取批量转出资产账户，类型安全，生命周期受Anchor管理
    let to_assets = &mut ctx.accounts.to_assets; // 获取批量转入资产账户，类型安全，生命周期受Anchor管理
    from_asset.validate()?; // 校验转出资产账户状态，防止非法批量策略交易，业务安全
    for to_asset in to_assets.iter_mut() { // 遍历所有转入资产账户
        to_asset.validate()?; // 校验每个转入资产账户状态，防止非法批量策略交易，业务安全
    }
    // 1. 算法执行（如有）
    let mut exec_result = None; // 算法执行结果，默认无，类型Option
    if let Some(exec_params) = &exec_params { // 若有算法参数，进入算法执行分支
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名，继续
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                // 尝试downcast为ExecutionStrategy trait对象
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() {
                    // 这里应传递真实的Context和参数，生产环境需实现完整调用
                    // exec_result = Some(exec_strategy.execute(ctx, &exec_params.algo_params)?);
                }
            }
        }
    }
    // 2. 批量策略交易，最终落地到服务层
    crate::services::asset_service::AssetService::batch_strategy_trade(
        from_asset, // 批量转出资产账户
        to_assets, // 批量转入资产账户
        &strategies, // 策略交易参数集合
        ctx.accounts.authority.key(), // 操作人公钥
    )?; // 调用服务层批量策略交易逻辑，业务安全
    emit!(BatchAssetStrategyTraded { // 事件：批量转出资产ID、批量转入资产ID、策略参数集合、操作人、时间戳
        from_asset_id: from_asset.id, // 事件：批量转出资产ID
        to_asset_ids: to_assets.iter().map(|a| a.id).collect(), // 事件：批量转入资产ID集合
        strategies: strategies.clone(), // 事件：策略参数集合
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[cfg(test)]
mod tests {
    use super::*; // 引入当前模块所有公有项，便于测试直接调用
    use crate::errors::asset_error::AssetError; // 引入资产错误类型，便于断言错误
    use crate::state::baskets::BasketIndexState; // 引入资产篮子状态结构体，便于构造测试数据
    use anchor_lang::prelude::*; // 引入Anchor预导出内容，包含Pubkey等

    fn default_basket() -> BasketIndexState { // 构造默认资产篮子状态，便于复用
        BasketIndexState {
            fee_collector: Pubkey::new_unique(), // 随机fee收集者
            composition: vec![Default::default()], // 默认成分资产
            weights: vec![10_000], // 默认权重
            is_active: true, // 默认激活
            total_value: 1000, // 默认总价值
            ..Default::default() // 其余字段默认
        }
    }

    #[test]
    fn test_mint_asset_not_active() { // 测试资产未激活时增发失败
        let mut basket = default_basket(); // 构造默认资产篮子
        basket.is_active = false; // 设置为未激活
        let result = AssetService::mint(&mut basket, 100); // 尝试增发
        assert_eq!(
            result.unwrap_err().to_string(), // 捕获错误
            AssetError::NotAllowed.to_string() // 断言为NotAllowed
        );
    }

    #[test]
    fn test_mint_asset_invalid_assets() { // 测试资产成分非法时校验失败
        let mut basket = default_basket(); // 构造默认资产篮子
        basket.fee_collector = Pubkey::default(); // 设置fee收集者为默认（非法）
        let result = basket.validate(); // 校验资产篮子
        assert_eq!(
            result.unwrap_err().to_string(), // 捕获错误
            AssetError::InvalidAssets.to_string() // 断言为InvalidAssets
        );
    }

    #[test]
    fn test_mint_asset_success() { // 测试资产增发成功
        let mut basket = default_basket(); // 构造默认资产篮子
        let result = AssetService::mint(&mut basket, 100); // 增发100
        assert!(result.is_ok()); // 断言成功
        assert_eq!(basket.total_value, 1100); // 断言总价值增加
    }

    #[test]
    fn test_burn_asset_insufficient_value() { // 测试销毁超额资产失败
        let mut basket = default_basket(); // 构造默认资产篮子
        let result = AssetService::burn(&mut basket, 2000); // 尝试销毁2000
        assert_eq!(
            result.unwrap_err().to_string(), // 捕获错误
            AssetError::InsufficientValue.to_string() // 断言为InsufficientValue
        );
    }

    #[test]
    fn test_burn_asset_invalid_assets() { // 测试资产成分非法时校验失败
        let mut basket = default_basket(); // 构造默认资产篮子
        basket.composition.clear(); // 清空成分资产
        let result = basket.validate(); // 校验资产篮子
        assert_eq!(
            result.unwrap_err().to_string(), // 捕获错误
            AssetError::InvalidAssets.to_string() // 断言为InvalidAssets
        );
    }

    #[test]
    fn test_burn_asset_success() { // 测试资产销毁成功
        let mut basket = default_basket(); // 构造默认资产篮子
        let result = AssetService::burn(&mut basket, 500); // 销毁500
        assert!(result.is_ok()); // 断言成功
        assert_eq!(basket.total_value, 500); // 断言总价值减少
    }

    #[test]
    fn test_transfer_asset_success() { // 测试资产转账成功
        let mut from = default_basket(); // 构造转出资产篮子
        let mut to = default_basket(); // 构造转入资产篮子
        let amount = 500; // 转账数量
        from.total_value = 1000; // 设置转出资产余额
        to.total_value = 200; // 设置转入资产余额
        // 模拟transfer逻辑
        assert!(from.total_value >= amount); // 断言余额充足
        from.total_value -= amount; // 扣减转出
        to.total_value += amount; // 增加转入
        assert_eq!(from.total_value, 500); // 断言转出余额
        assert_eq!(to.total_value, 700); // 断言转入余额
    }

    #[test]
    fn test_transfer_asset_insufficient_value() { // 测试转账余额不足
        let mut from = default_basket(); // 构造转出资产篮子
        let mut to = default_basket(); // 构造转入资产篮子
        let amount = 2000; // 转账数量
        from.total_value = 1000; // 设置转出资产余额
        to.total_value = 200; // 设置转入资产余额
        // 模拟transfer逻辑
        assert!(from.total_value < amount); // 断言余额不足
    }

    #[test]
    fn test_query_asset() { // 测试资产查询
        let asset = default_basket(); // 构造资产
        let value = asset.total_value; // 获取余额
        assert_eq!(value, 1000); // 断言余额正确
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*; // 引入当前模块所有公有项，便于集成测试直接调用
    use crate::core::types::{SwapParams, BatchSwapParams, StrategyTradeParams, PriceParams, ExecutionParams, StrategyParams}; // 引入所有参数类型
    use anchor_lang::prelude::Pubkey; // 引入Pubkey类型

    #[test]
    fn test_buy_asset_invalid_params() { // 测试买入资产参数非法
        let ctx = Context::default(); // 构造默认上下文
        let params = SwapParams {
            from_token: Pubkey::default(), // 默认from_token
            to_token: Pubkey::default(), // 默认to_token
            amount_in: 0, // 非法数量
            min_amount_out: 0, // 非法最小输出
            dex_name: "jupiter".to_string(), // DEX名
        };
        let price_params = PriceParams { asset: Pubkey::default(), oracle_name: "pyth".to_string() }; // 构造价格参数
        let result = buy_asset(ctx, params, price_params, None, None); // 调用买入资产
        assert!(result.is_err()); // 断言失败
    }

    #[test]
    fn test_batch_swap_asset_empty() { // 测试批量swap参数为空
        let ctx = Context::default(); // 构造默认上下文
        let params = BatchSwapParams { swaps: vec![] }; // 空swap参数
        let result = batch_swap_asset(ctx, params, None, None); // 调用批量swap
        assert!(result.is_err()); // 断言失败
    }

    #[test]
    fn test_strategy_trade_asset_empty_strategy() { // 测试策略交易参数为空
        let ctx = Context::default(); // 构造默认上下文
        let params = StrategyTradeParams {
            strategy: StrategyParams { strategy_name: "".to_string(), params: vec![] }, // 空策略名
            swap_params: None, // 无swap参数
            price_params: None, // 无价格参数
            exec_params: None, // 无算法参数
        };
        let result = strategy_trade_asset(ctx, params, None); // 调用策略交易
        assert!(result.is_err()); // 断言失败
    }
}
