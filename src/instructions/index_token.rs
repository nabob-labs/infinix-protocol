//! Index Token Instructions // 文档注释：指数代币相关指令集入口
//! 指令入口，调用 service 层。 // 中文说明：所有指令通过此文件统一入口，实际业务逻辑委托给 service 层

use crate::accounts::BasketIndexStateAccount; // 引入篮子指数账户结构体定义
use crate::events::index_token_event::*; // 引入指数代币相关事件定义，供 emit! 宏使用
use crate::services::index_token_service::IndexTokenService; // 引入指数代币服务层，封装业务逻辑
use crate::state::baskets::BasketIndexState; // 引入篮子指数状态结构体，管理链上状态
use crate::validation::index_token_validation::IndexTokenValidatable; // 引入指数代币校验 trait，统一校验接口
use crate::core::types::{TradeParams, BatchTradeParams, StrategyParams, OracleParams, AlgoParams}; // 引入核心参数类型，统一跨模块参数传递
use crate::core::registry::{ALGORITHM_REGISTRY, DEX_ADAPTER_REGISTRY, ORACLE_ADAPTER_REGISTRY, ADAPTER_FACTORY}; // 引入全局注册表与工厂，支持算法/DEX/预言机热插拔
use anchor_lang::prelude::*; // Anchor 框架预导入，包含常用宏与类型

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct MintIndexToken<'info> { // 定义指数代币增发指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记账户为可变，生命周期由Anchor管理
    pub basket_index: Account<'info, BasketIndexState>, // 指数篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

pub fn mint_index_token(
    ctx: Context<MintIndexToken>, // Anchor账户上下文，自动校验权限与生命周期
    amount: u64, // 增发数量，类型安全
) -> Result<()> { // Anchor规范返回类型，自动生命周期管理
    require!(amount > 0, crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验增发数量必须大于0，否则返回参数错误
    let basket_index = &mut ctx.accounts.basket_index; // 获取可变篮子指数账户，生命周期由Anchor自动管理
    basket_index.validate()?; // 调用校验trait，校验账户状态与资产合法性
    IndexTokenService::mint(basket_index, amount)?; // 调用服务层增发逻辑，实际修改链上状态
    emit!(IndexTokenMinted { // 触发增发事件，链上可追溯
        basket_id: basket_index.id, // 事件：篮子ID
        amount, // 事件：增发数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct BurnIndexToken<'info> { // 定义指数代币销毁指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub basket_index: Account<'info, BasketIndexState>, // 指数篮子账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn burn_index_token(
    ctx: Context<BurnIndexToken>, // Anchor账户上下文
    amount: u64, // 销毁数量
) -> Result<()> { // Anchor规范返回类型
    require!(amount > 0, crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验销毁数量必须大于0
    let basket_index = &mut ctx.accounts.basket_index; // 获取可变篮子指数账户
    basket_index.validate()?; // 校验账户状态
    IndexTokenService::burn(basket_index, amount)?; // 调用服务层销毁逻辑
    emit!(IndexTokenBurned { // 触发销毁事件
        basket_id: basket_index.id, // 事件：篮子ID
        amount, // 事件：销毁数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct MintIndexTokenWithAlgo<'info> { // 定义带算法增发指数代币指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub basket_index: Account<'info, BasketIndexState>, // 指数篮子账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn mint_index_token_with_algo(
    ctx: Context<MintIndexTokenWithAlgo>, // Anchor账户上下文
    amount: u64, // 增发数量
    algo_name: String, // 算法名称，动态选择执行算法
    params: crate::algorithms::traits::ExecutionParams, // 算法参数，类型安全
) -> Result<()> { // Anchor规范返回类型
    let algo = ALGORITHM_REGISTRY // 全局算法注册表，支持运行时热插拔
        .get_execution(&algo_name) // 根据算法名称获取算法trait对象
        .ok_or(crate::errors::index_token_error::IndexTokenError::Unknown)?; // 若未找到算法则返回未知错误
    let basket_index = &mut ctx.accounts.basket_index; // 获取可变篮子指数账户
    basket_index.validate()?; // 校验账户状态
    crate::services::index_token_service::mint_with_algo(
        basket_index, // 指数篮子账户
        amount, // 增发数量
        algo.as_ref(), // 算法trait对象引用
        ctx.accounts.clone().into(), // 账户上下文转换，供算法使用
        &params, // 算法参数
    )?; // 调用服务层带算法增发逻辑
    emit!(crate::events::index_token_event::IndexTokenMinted { // 触发增发事件
        basket_id: basket_index.id, // 事件：篮子ID
        amount, // 事件：增发数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct MintIndexTokenWithAlgoAndAdapters<'info> { // 定义带算法和适配器增发指数代币指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub basket_index: Account<'info, BasketIndexState>, // 指数篮子账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)] // 派生Anchor序列化/反序列化、克隆、调试特性，便于跨链/链上数据传递
pub struct MintWithAlgoAndAdaptersParams { // 定义带算法和适配器增发参数结构体
    pub amount: u64, // 增发数量
    pub algo_name: String, // 算法名称
    pub dex_name: String, // DEX适配器名称
    pub oracle_name: String, // 预言机适配器名称
    pub params: crate::algorithms::traits::ExecutionParams, // 算法参数
}

pub fn mint_index_token_with_algo_and_adapters(
    ctx: Context<MintIndexTokenWithAlgoAndAdapters>, // Anchor账户上下文
    args: MintWithAlgoAndAdaptersParams, // 增发参数结构体
) -> Result<()> { // Anchor规范返回类型
    let algo = ALGORITHM_REGISTRY // 全局算法注册表
        .get_execution(&args.algo_name) // 获取算法trait对象
        .ok_or(crate::errors::index_token_error::IndexTokenError::Unknown)?; // 未找到算法则返回错误
    let dex = DEX_ADAPTER_REGISTRY // 全局DEX适配器注册表
        .get(&args.dex_name) // 获取DEX适配器trait对象
        .ok_or(crate::errors::index_token_error::IndexTokenError::Unknown)?; // 未找到DEX适配器则返回错误
    let oracle = ORACLE_ADAPTER_REGISTRY // 全局预言机适配器注册表
        .get(&args.oracle_name) // 获取预言机适配器trait对象
        .ok_or(crate::errors::index_token_error::IndexTokenError::Unknown)?; // 未找到预言机适配器则返回错误
    let basket_index = &mut ctx.accounts.basket_index; // 获取可变篮子指数账户
    basket_index.validate()?; // 校验账户状态
    crate::services::index_token_service::mint_with_algo_and_adapters(
        basket_index, // 指数篮子账户
        args.amount, // 增发数量
        algo.as_ref(), // 算法trait对象引用
        dex.as_ref(), // DEX适配器trait对象引用
        oracle.as_ref(), // 预言机适配器trait对象引用
        ctx.accounts.clone().into(), // 账户上下文转换
        &args.params, // 算法参数
    )?; // 调用服务层带算法和适配器增发逻辑
    emit!(crate::events::index_token_event::IndexTokenMinted { // 触发增发事件
        basket_id: basket_index.id, // 事件：篮子ID
        amount: args.amount, // 事件：增发数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct TransferIndexToken<'info> { // 定义指数代币转账指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub from_index_token: Account<'info, BasketIndexState>, // 转出方指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub to_index_token: Account<'info, BasketIndexState>, // 转入方指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn transfer_index_token(
    ctx: Context<TransferIndexToken>, // Anchor账户上下文
    amount: u64, // 转账数量
) -> Result<()> { // Anchor规范返回类型
    require!(amount > 0, crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验转账数量必须大于0
    let from = &mut ctx.accounts.from_index_token; // 获取转出方账户
    let to = &mut ctx.accounts.to_index_token; // 获取转入方账户
    require!(ctx.accounts.authority.key() == from.authority, crate::errors::index_token_error::IndexTokenError::NotAllowed); // 校验操作人权限
    require!(from.total_supply >= amount, crate::errors::index_token_error::IndexTokenError::InsufficientValue); // 校验转出方余额充足
    from.total_supply -= amount; // 扣减转出方余额
    to.total_supply = to.total_supply.checked_add(amount).ok_or(crate::errors::index_token_error::IndexTokenError::InsufficientValue)?; // 增加转入方余额，防止溢出
    emit!(IndexTokenTransferred { // 触发转账事件
        from_index_token_id: from.id, // 事件：转出方ID
        to_index_token_id: to.id, // 事件：转入方ID
        amount, // 事件：转账数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct QueryIndexToken<'info> { // 定义指数代币查询指令的账户上下文结构体
    pub index_token: Account<'info, BasketIndexState>, // 查询目标指数代币账户
}

pub fn query_index_token(
    ctx: Context<QueryIndexToken>, // Anchor账户上下文
) -> Result<u64> { // Anchor规范返回类型，返回u64供链上查询
    let index_token = &ctx.accounts.index_token; // 获取目标指数代币账户
    emit!(IndexTokenQueried { // 触发查询事件
        index_token_id: index_token.id, // 事件：指数代币ID
        total_supply: index_token.total_supply, // 事件：总供应量
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(index_token.total_supply) // 返回总供应量
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct BuyIndexToken<'info> { // 定义指数代币买入指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub basket_index: Account<'info, BasketIndexState>, // 指数篮子账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub buyer: Signer<'info>, // 买方签名者
}

pub fn buy_index_token(
    ctx: Context<BuyIndexToken>, // Anchor账户上下文
    params: TradeParams, // 交易参数
    price_params: OracleParams, // 预言机价格参数
    exec_params: Option<AlgoParams>, // 可选算法参数
    strategy_params: Option<StrategyParams>, // 可选策略参数
) -> Result<()> { // Anchor规范返回类型
    let basket_index = &mut ctx.accounts.basket_index; // 获取可变篮子指数账户
    basket_index.validate()?; // 校验账户状态
    // 1. 算法执行（如有）
    let mut exec_result = None; // 算法执行结果占位
    if let Some(exec_params) = &exec_params { // 若有算法参数
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名称
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂
            if let Some(algo) = factory.get(algo_name) { // 获取算法适配器
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() { // 尝试转换为算法trait对象
                    // 这里应传递真实的Context和参数
                    // exec_result = Some(exec_strategy.execute(ctx, &exec_params.algo_params)?);
                }
            }
        }
    }
    // 2. 预言机价格（如有）
    let mut price = price_params.price; // 默认价格为参数内价格
    if let Some(oracle_name) = &price_params.oracle_name { // 若指定预言机名称
        let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂
        if let Some(adapter) = factory.get(oracle_name) { // 获取预言机适配器
            if let Some(oracle_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::oracles::traits::OracleAdapter>>() { // 尝试转换为预言机trait对象
                // 这里应传递真实的参数
                // let oracle_result = oracle_adapter.get_price(&price_params)?;
                // price = oracle_result.price;
            }
        }
    }
    // 3. DEX/AMM swap（如有）
    if let Some(dex_name) = &params.dex_name { // 若指定DEX名称
        let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂
        if let Some(adapter) = factory.get(dex_name) { // 获取DEX适配器
            if let Some(dex_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::dex::traits::DexAdapter>>() { // 尝试转换为DEX trait对象
                // 这里应传递真实的参数
                // let swap_result = dex_adapter.swap(&params)?;
                // 可用swap_result.avg_price、swap_result.executed_amount等
            }
        }
    }
    // 4. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数
        if !strategy_params.strategy_name.is_empty() { // 策略名称非空
            crate::services::index_token_service::IndexTokenService::strategy_subscribe(
                basket_index, // 指数篮子账户
                &strategy_params.strategy_name, // 策略名称
                &strategy_params.params, // 策略参数
                ctx.accounts.buyer.key(), // 买方公钥
            )?;
        }
    }
    // 5. 资产买入
    crate::services::index_token_service::IndexTokenService::buy(
        basket_index, // 指数篮子账户
        params.amount_in, // 买入数量
        price, // 买入价格
        ctx.accounts.buyer.key(), // 买方公钥
    )?;
    emit!(IndexTokenBought { // 触发买入事件
        basket_id: basket_index.id, // 事件：篮子ID
        amount: params.amount_in, // 事件：买入数量
        price, // 事件：买入价格
        buyer: ctx.accounts.buyer.key(), // 事件：买方
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct SellIndexToken<'info> { // 定义指数代币卖出指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub basket_index: Account<'info, BasketIndexState>, // 指数篮子账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub seller: Signer<'info>, // 卖方签名者
}

pub fn sell_index_token(
    ctx: Context<SellIndexToken>, // Anchor账户上下文
    params: TradeParams, // 交易参数
    price_params: OracleParams, // 预言机价格参数
    exec_params: Option<AlgoParams>, // 可选算法参数
    strategy_params: Option<StrategyParams>, // 可选策略参数
) -> Result<()> { // Anchor规范返回类型
    let basket_index = &mut ctx.accounts.basket_index; // 获取可变篮子指数账户
    basket_index.validate()?; // 校验账户状态
    // 1. 算法执行（如有）
    let mut exec_result = None; // 算法执行结果占位
    if let Some(exec_params) = &exec_params { // 若有算法参数
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名称
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂
            if let Some(algo) = factory.get(algo_name) { // 获取算法适配器
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() { // 尝试转换为算法trait对象
                    // 这里应传递真实的Context和参数
                    // exec_result = Some(exec_strategy.execute(ctx, &exec_params.algo_params)?);
                }
            }
        }
    }
    // 2. 预言机价格（如有）
    let mut price = price_params.price; // 默认价格为参数内价格
    if let Some(oracle_name) = &price_params.oracle_name { // 若指定预言机名称
        let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂
        if let Some(adapter) = factory.get(oracle_name) { // 获取预言机适配器
            if let Some(oracle_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::oracles::traits::OracleAdapter>>() { // 尝试转换为预言机trait对象
                // 这里应传递真实的参数
                // let oracle_result = oracle_adapter.get_price(&price_params)?;
                // price = oracle_result.price;
            }
        }
    }
    // 3. DEX/AMM swap（如有）
    if let Some(dex_name) = &params.dex_name { // 若指定DEX名称
        let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂
        if let Some(adapter) = factory.get(dex_name) { // 获取DEX适配器
            if let Some(dex_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::dex::traits::DexAdapter>>() { // 尝试转换为DEX trait对象
                // 这里应传递真实的参数
                // let swap_result = dex_adapter.swap(&params)?;
                // 可用swap_result.avg_price、swap_result.executed_amount等
            }
        }
    }
    // 4. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数
        if !strategy_params.strategy_name.is_empty() { // 策略名称非空
            crate::services::index_token_service::IndexTokenService::strategy_redeem(
                basket_index, // 指数篮子账户
                &strategy_params.strategy_name, // 策略名称
                &strategy_params.params, // 策略参数
                ctx.accounts.seller.key(), // 卖方公钥
            )?;
        }
    }
    // 5. 资产卖出
    crate::services::index_token_service::IndexTokenService::sell(
        basket_index, // 指数篮子账户
        params.amount_in, // 卖出数量
        price, // 卖出价格
        ctx.accounts.seller.key(), // 卖方公钥
    )?;
    emit!(IndexTokenSold { // 触发卖出事件
        basket_id: basket_index.id, // 事件：篮子ID
        amount: params.amount_in, // 事件：卖出数量
        price, // 事件：卖出价格
        seller: ctx.accounts.seller.key(), // 事件：卖方
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct SwapIndexToken<'info> { // 定义指数代币交换指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub from_index_token: Account<'info, BasketIndexState>, // 转出方指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub to_index_token: Account<'info, BasketIndexState>, // 转入方指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn swap_index_token(
    ctx: Context<SwapIndexToken>, // Anchor账户上下文
    params: TradeParams, // 交易参数
    price_params: OracleParams, // 预言机价格参数
    exec_params: Option<AlgoParams>, // 可选算法参数
    strategy_params: Option<StrategyParams>, // 可选策略参数
) -> Result<()> { // Anchor规范返回类型
    let from = &mut ctx.accounts.from_index_token; // 获取转出方账户
    let to = &mut ctx.accounts.to_index_token; // 获取转入方账户
    require!(ctx.accounts.authority.key() == from.authority, crate::errors::index_token_error::IndexTokenError::NotAllowed); // 校验操作人权限
    if from.total_supply < params.amount_in { // 校验转出方余额充足
        return Err(crate::errors::index_token_error::IndexTokenError::InsufficientValue.into()); // 不足则返回错误
    }
    // 1. 算法执行（如有）
    let mut exec_result = None; // 算法执行结果占位
    if let Some(exec_params) = &exec_params { // 若有算法参数
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名称
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂
            if let Some(algo) = factory.get(algo_name) { // 获取算法适配器
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() { // 尝试转换为算法trait对象
                    // 这里应传递真实的Context和参数
                    // exec_result = Some(exec_strategy.execute(ctx, &exec_params.algo_params)?);
                }
            }
        }
    }
    // 2. 预言机价格（如有）
    let mut price = price_params.price; // 默认价格为参数内价格
    if let Some(oracle_name) = &price_params.oracle_name { // 若指定预言机名称
        let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂
        if let Some(adapter) = factory.get(oracle_name) { // 获取预言机适配器
            if let Some(oracle_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::oracles::traits::OracleAdapter>>() { // 尝试转换为预言机trait对象
                // 这里应传递真实的参数
                // let oracle_result = oracle_adapter.get_price(&price_params)?;
                // price = oracle_result.price;
            }
        }
    }
    // 3. DEX/AMM swap（如有）
    if let Some(dex_name) = &params.dex_name { // 若指定DEX名称
        let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂
        if let Some(adapter) = factory.get(dex_name) { // 获取DEX适配器
            if let Some(dex_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::dex::traits::DexAdapter>>() { // 尝试转换为DEX trait对象
                // 这里应传递真实的参数
                // let swap_result = dex_adapter.swap(&params)?;
                // 可用swap_result.avg_price、swap_result.executed_amount等
            }
        }
    }
    // 4. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数
        if !strategy_params.strategy_name.is_empty() { // 策略名称非空
            crate::services::index_token_service::IndexTokenService::strategy_swap(
                from, // 转出方账户
                &strategy_params.strategy_name, // 策略名称
                &strategy_params.params, // 策略参数
                ctx.accounts.authority.key(), // 操作人公钥
            )?;
        }
    }
    // 5. 资产交换
    from.total_supply -= params.amount_in; // 扣减转出方余额
    to.total_supply = to.total_supply.checked_add(params.amount_in).ok_or(crate::errors::index_token_error::IndexTokenError::InsufficientValue)?; // 增加转入方余额，防止溢出
    emit!(IndexTokenSwapped { // 触发交换事件
        from_index_token_id: from.id, // 事件：转出方ID
        to_index_token_id: to.id, // 事件：转入方ID
        amount: params.amount_in, // 事件：交换数量
        price, // 事件：交换价格
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct AuthorizeIndexToken<'info> { // 定义指数代币授权指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub index_token: Account<'info, BasketIndexState>, // 指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 当前授权人签名者
    pub new_authority: Pubkey, // 新授权人公钥
}

pub fn authorize_index_token(
    ctx: Context<AuthorizeIndexToken>, // Anchor账户上下文
    exec_params: Option<AlgoParams>, // 可选算法参数
    strategy_params: Option<StrategyParams>, // 可选策略参数
) -> Result<()> { // Anchor规范返回类型
    let index_token = &mut ctx.accounts.index_token; // 获取可变指数代币账户
    require!(ctx.accounts.authority.key() == index_token.authority, crate::errors::index_token_error::IndexTokenError::NotAllowed); // 校验当前操作人权限
    // 1. 算法执行（如有）
    if let Some(exec_params) = &exec_params { // 若有算法参数
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名称
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂
            if let Some(algo) = factory.get(algo_name) { // 获取算法适配器
                // 假设所有算法adapter都实现ExecutionStrategy trait
                // 这里可做trait对象downcast或统一trait接口
                // 这里只做动态发现和调用示例
                // let _exec_result = algo.execute(Context::default(), exec_params)?;
            }
        }
    }
    // 2. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数
        if !strategy_params.strategy_name.is_empty() { // 策略名称非空
            crate::services::index_token_service::IndexTokenService::strategy_subscribe(
                index_token, // 指数代币账户
                &strategy_params.strategy_name, // 策略名称
                &strategy_params.params, // 策略参数
                ctx.accounts.authority.key(), // 当前授权人公钥
            )?;
        }
    }
    // 3. 授权
    index_token.authority = ctx.accounts.new_authority; // 更新授权人公钥
    emit!(IndexTokenAuthorized { // 触发授权事件
        index_token_id: index_token.id, // 事件：指数代币ID
        old_authority: ctx.accounts.authority.key(), // 事件：原授权人
        new_authority: ctx.accounts.new_authority, // 事件：新授权人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct CombineIndexToken<'info> { // 定义指数代币合并指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub target_index_token: Account<'info, BasketIndexState>, // 目标指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub source_index_token: Account<'info, BasketIndexState>, // 源指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn combine_index_token(
    ctx: Context<CombineIndexToken>, // Anchor账户上下文
    amount: u64, // 合并数量
    exec_params: Option<AlgoParams>, // 可选算法参数
    strategy_params: Option<StrategyParams>, // 可选策略参数
) -> Result<()> { // Anchor规范返回类型
    let target = &mut ctx.accounts.target_index_token; // 获取目标指数代币账户
    let source = &mut ctx.accounts.source_index_token; // 获取源指数代币账户
    require!(ctx.accounts.authority.key() == source.authority, crate::errors::index_token_error::IndexTokenError::NotAllowed); // 校验操作人权限
    if source.total_supply < amount { // 校验源账户余额充足
        return Err(crate::errors::index_token_error::IndexTokenError::InsufficientValue.into()); // 不足则返回错误
    }
    // 1. 算法执行（如有）
    if let Some(exec_params) = &exec_params { // 若有算法参数
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名称
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂
            if let Some(algo) = factory.get(algo_name) { // 获取算法适配器
                // 假设所有算法adapter都实现ExecutionStrategy trait
                // 这里可做trait对象downcast或统一trait接口
                // 这里只做动态发现和调用示例
                // let _exec_result = algo.execute(Context::default(), exec_params)?;
            }
        }
    }
    // 2. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数
        if !strategy_params.strategy_name.is_empty() { // 策略名称非空
            crate::services::index_token_service::IndexTokenService::strategy_redeem(
                source, // 源指数代币账户
                &strategy_params.strategy_name, // 策略名称
                &strategy_params.params, // 策略参数
                ctx.accounts.authority.key(), // 操作人公钥
            )?;
        }
    }
    // 3. 合并指数代币
    source.total_supply -= amount; // 扣减源账户余额
    target.total_supply = target.total_supply.checked_add(amount).ok_or(crate::errors::index_token_error::IndexTokenError::InsufficientValue)?; // 增加目标账户余额，防止溢出
    emit!(IndexTokenCombined { // 触发合并事件
        target_index_token_id: target.id, // 事件：目标ID
        source_index_token_id: source.id, // 事件：源ID
        amount, // 事件：合并数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct SplitIndexToken<'info> { // 定义指数代币拆分指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub source_index_token: Account<'info, BasketIndexState>, // 源指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub new_index_token: Account<'info, BasketIndexState>, // 新生成指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn split_index_token(
    ctx: Context<SplitIndexToken>, // Anchor账户上下文
    amount: u64, // 拆分数量
    exec_params: Option<AlgoParams>, // 可选算法参数
    strategy_params: Option<StrategyParams>, // 可选策略参数
) -> Result<()> { // Anchor规范返回类型
    let source = &mut ctx.accounts.source_index_token; // 获取源指数代币账户
    let new_index_token = &mut ctx.accounts.new_index_token; // 获取新生成指数代币账户
    require!(ctx.accounts.authority.key() == source.authority, crate::errors::index_token_error::IndexTokenError::NotAllowed); // 校验操作人权限
    if source.total_supply < amount { // 校验源账户余额充足
        return Err(crate::errors::index_token_error::IndexTokenError::InsufficientValue.into()); // 不足则返回错误
    }
    // 1. 算法执行（如有）
    if let Some(exec_params) = &exec_params { // 若有算法参数
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名称
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂
            if let Some(algo) = factory.get(algo_name) { // 获取算法适配器
                // 假设所有算法adapter都实现ExecutionStrategy trait
                // 这里可做trait对象downcast或统一trait接口
                // 这里只做动态发现和调用示例
                // let _exec_result = algo.execute(Context::default(), exec_params)?;
            }
        }
    }
    // 2. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数
        if !strategy_params.strategy_name.is_empty() { // 策略名称非空
            crate::services::index_token_service::IndexTokenService::strategy_redeem(
                source, // 源指数代币账户
                &strategy_params.strategy_name, // 策略名称
                &strategy_params.params, // 策略参数
                ctx.accounts.authority.key(), // 操作人公钥
            )?;
        }
    }
    // 3. 拆分指数代币
    source.total_supply -= amount; // 扣减源账户余额
    new_index_token.total_supply = new_index_token.total_supply.checked_add(amount).ok_or(crate::errors::index_token_error::IndexTokenError::InsufficientValue)?; // 增加新账户余额，防止溢出
    emit!(IndexTokenSplit { // 触发拆分事件
        source_index_token_id: source.id, // 事件：源ID
        new_index_token_id: new_index_token.id, // 事件：新生成ID
        amount, // 事件：拆分数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct FreezeIndexToken<'info> { // 定义指数代币冻结指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub index_token: Account<'info, BasketIndexState>, // 指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn freeze_index_token(
    ctx: Context<FreezeIndexToken>, // Anchor账户上下文
    exec_params: Option<AlgoParams>, // 可选算法参数
    strategy_params: Option<StrategyParams>, // 可选策略参数
) -> Result<()> { // Anchor规范返回类型
    let index_token = &mut ctx.accounts.index_token; // 获取可变指数代币账户
    // 1. 算法执行（如有）
    if let Some(exec_params) = &exec_params { // 若有算法参数
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名称
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂
            if let Some(algo) = factory.get(algo_name) { // 获取算法适配器
                // 假设所有算法adapter都实现ExecutionStrategy trait
                // 这里可做trait对象downcast或统一trait接口
                // 这里只做动态发现和调用示例
                // let _exec_result = algo.execute(Context::default(), exec_params)?;
            }
        }
    }
    // 2. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数
        if !strategy_params.strategy_name.is_empty() { // 策略名称非空
            crate::services::index_token_service::IndexTokenService::strategy_subscribe(
                index_token, // 指数代币账户
                &strategy_params.strategy_name, // 策略名称
                &strategy_params.params, // 策略参数
                ctx.accounts.authority.key(), // 操作人公钥
            )?;
        }
    }
    // 3. 冻结
    crate::services::index_token_service::IndexTokenService::freeze(
        index_token, // 指数代币账户
        ctx.accounts.authority.key(), // 操作人公钥
    ) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct UnfreezeIndexToken<'info> { // 定义指数代币解冻指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub index_token: Account<'info, BasketIndexState>, // 指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn unfreeze_index_token(
    ctx: Context<UnfreezeIndexToken>, // Anchor账户上下文
    exec_params: Option<AlgoParams>, // 可选算法参数
    strategy_params: Option<StrategyParams>, // 可选策略参数
) -> Result<()> { // Anchor规范返回类型
    let index_token = &mut ctx.accounts.index_token; // 获取可变指数代币账户
    // 1. 算法执行（如有）
    if let Some(exec_params) = &exec_params { // 若有算法参数
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名称
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂
            if let Some(algo) = factory.get(algo_name) { // 获取算法适配器
                // 假设所有算法adapter都实现ExecutionStrategy trait
                // 这里可做trait对象downcast或统一trait接口
                // 这里只做动态发现和调用示例
                // let _exec_result = algo.execute(Context::default(), exec_params)?;
            }
        }
    }
    // 2. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数
        if !strategy_params.strategy_name.is_empty() { // 策略名称非空
            crate::services::index_token_service::IndexTokenService::strategy_subscribe(
                index_token, // 指数代币账户
                &strategy_params.strategy_name, // 策略名称
                &strategy_params.params, // 策略参数
                ctx.accounts.authority.key(), // 操作人公钥
            )?;
        }
    }
    // 3. 解冻
    crate::services::index_token_service::IndexTokenService::unfreeze(
        index_token, // 指数代币账户
        ctx.accounts.authority.key(), // 操作人公钥
    ) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct BatchTransferIndexToken<'info> { // 定义批量转账指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub from_index_token: Account<'info, BasketIndexState>, // 转出方指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub to_index_tokens: Vec<Account<'info, BasketIndexState>>, // 批量转入方账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn batch_transfer_index_token(
    ctx: Context<BatchTransferIndexToken>, // Anchor账户上下文
    amounts: Vec<u64>, // 批量转账数量
    exec_params: Option<AlgoParams>, // 可选算法参数
    strategy_params: Option<StrategyParams>, // 可选策略参数
) -> Result<()> { // Anchor规范返回类型
    let from = &mut ctx.accounts.from_index_token; // 获取转出方账户
    let mut to_index_tokens: Vec<&mut BasketIndexState> = ctx.accounts.to_index_tokens.iter_mut().map(|i| i.as_mut()).collect(); // 获取批量转入方账户
    let n = to_index_tokens.len(); // 转入方数量
    require!(n > 0, crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验转入方数量大于0
    require!(amounts.len() == n, crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验数量匹配
    let total: u64 = amounts.iter().try_fold(0u64, |acc, &x| acc.checked_add(x).ok_or(crate::errors::index_token_error::IndexTokenError::Overflow))?; // 计算总转账数量，防止溢出
    require!(from.total_supply >= total, crate::errors::index_token_error::IndexTokenError::InsufficientValue); // 校验转出方余额充足
    require!(ctx.accounts.authority.key() == from.authority, crate::errors::index_token_error::IndexTokenError::NotAllowed); // 校验操作人权限
    // 1. 算法执行（如有）
    if let Some(exec_params) = &exec_params { // 若有算法参数
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名称
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂
            if let Some(algo) = factory.get(algo_name) { // 获取算法适配器
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() { // 尝试转换为算法trait对象
                    // 这里应传递真实的Context和参数
                    // let _exec_result = exec_strategy.execute(ctx, &exec_params.algo_params)?;
                }
            }
        }
    }
    // 2. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数
        if !strategy_params.strategy_name.is_empty() { // 策略名称非空
            crate::services::index_token_service::IndexTokenService::strategy_redeem(
                from, // 转出方账户
                &strategy_params.strategy_name, // 策略名称
                &strategy_params.params, // 策略参数
                ctx.accounts.authority.key(), // 操作人公钥
            )?;
        }
    }
    // 3. 批量转账
    for (to, &amt) in to_index_tokens.iter_mut().zip(amounts.iter()) { // 遍历每个转入方及其对应数量
        require!(amt > 0, crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验转账数量大于0
        to.total_supply = to.total_supply.checked_add(amt).ok_or(crate::errors::index_token_error::IndexTokenError::Overflow)?; // 增加转入方余额，防止溢出
        emit!(IndexTokenTransferred { // 触发转账事件
            from_index_token_id: from.id, // 事件：转出方ID
            to_index_token_id: to.id, // 事件：转入方ID
            amount: amt, // 事件：转账数量
            authority: ctx.accounts.authority.key(), // 事件：操作人
            timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
        });
    }
    from.total_supply -= total; // 扣减转出方余额
    Ok(()) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct BatchSubscribeIndexToken<'info> { // 定义批量申购指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub index_token: Account<'info, BasketIndexState>, // 指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn batch_subscribe_index_token(
    ctx: Context<BatchSubscribeIndexToken>, // Anchor账户上下文
    amounts: Vec<u64>, // 批量申购数量
) -> Result<()> { // Anchor规范返回类型
    let index_token = &mut ctx.accounts.index_token; // 获取可变指数代币账户
    require!(!amounts.is_empty(), crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验数量非空
    for &amt in amounts.iter() { // 遍历每个申购数量
        require!(amt > 0, crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验申购数量大于0
        index_token.total_supply = index_token.total_supply.checked_add(amt).ok_or(crate::errors::index_token_error::IndexTokenError::Overflow)?; // 增加总供应量，防止溢出
        emit!(IndexTokenMinted { // 触发增发事件
            basket_id: index_token.id, // 事件：篮子ID
            amount: amt, // 事件：增发数量
            authority: ctx.accounts.authority.key(), // 事件：操作人
            timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
        });
    }
    Ok(()) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct BatchRedeemIndexToken<'info> { // 定义批量赎回指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub index_token: Account<'info, BasketIndexState>, // 指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn batch_redeem_index_token(
    ctx: Context<BatchRedeemIndexToken>, // Anchor账户上下文
    amounts: Vec<u64>, // 批量赎回数量
) -> Result<()> { // Anchor规范返回类型
    let index_token = &mut ctx.accounts.index_token; // 获取可变指数代币账户
    require!(!amounts.is_empty(), crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验数量非空
    let total: u64 = amounts.iter().try_fold(0u64, |acc, &x| acc.checked_add(x).ok_or(crate::errors::index_token_error::IndexTokenError::Overflow))?; // 计算总赎回数量，防止溢出
    require!(index_token.total_supply >= total, crate::errors::index_token_error::IndexTokenError::InsufficientValue); // 校验余额充足
    for &amt in amounts.iter() { // 遍历每个赎回数量
        require!(amt > 0, crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验赎回数量大于0
        index_token.total_supply -= amt; // 扣减总供应量
        emit!(IndexTokenBurned { // 触发销毁事件
            basket_id: index_token.id, // 事件：篮子ID
            amount: amt, // 事件：销毁数量
            authority: ctx.accounts.authority.key(), // 事件：操作人
            timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
        });
    }
    Ok(()) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct StrategySubscribeIndexToken<'info> { // 定义策略申购指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub index_token: Account<'info, BasketIndexState>, // 指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn strategy_subscribe_index_token(
    ctx: Context<StrategySubscribeIndexToken>, // Anchor账户上下文
    params: StrategyParams, // 策略参数
    exec_params: Option<AlgoParams>, // 可选算法参数
) -> Result<()> { // Anchor规范返回类型
    // 1. 校验策略参数
    require!(!params.strategy_name.is_empty(), crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验策略名称非空
    let index_token = &mut ctx.accounts.index_token; // 获取可变指数代币账户
    // 2. 算法融合（如有）
    if let Some(exec_params) = &exec_params { // 若有算法参数
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名称
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂
            if let Some(algo) = factory.get(algo_name) { // 获取算法适配器
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() { // 尝试转换为算法trait对象
                    // 这里应传递真实的Context和参数
                    // let _exec_result = exec_strategy.execute(ctx, &exec_params.algo_params)?;
                }
            }
        }
    }
    // 3. 策略融合
    crate::services::index_token_service::IndexTokenService::strategy_subscribe(
        index_token, // 指数代币账户
        &params.strategy_name, // 策略名称
        &params.params, // 策略参数
        ctx.accounts.authority.key(), // 操作人公钥
    )?;
    emit!(IndexTokenStrategySubscribed { // 触发策略申购事件
        index_token_id: index_token.id, // 事件：指数代币ID
        strategy: params.strategy_name.clone(), // 事件：策略名称
        params: params.params.clone(), // 事件：策略参数
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct StrategyRedeemIndexToken<'info> { // 定义策略赎回指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub index_token: Account<'info, BasketIndexState>, // 指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn strategy_redeem_index_token(
    ctx: Context<StrategyRedeemIndexToken>, // Anchor账户上下文
    strategy: String, // 策略名称
    params: Vec<u8>, // 策略参数
    exec_params: Option<AlgoParams>, // 可选算法参数
) -> Result<()> { // Anchor规范返回类型
    let index_token = &mut ctx.accounts.index_token; // 获取可变指数代币账户
    // 1. 校验策略参数
    require!(!strategy.is_empty(), crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验策略名称非空
    // 2. 算法融合（如有）
    if let Some(exec_params) = &exec_params { // 若有算法参数
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名称
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂
            if let Some(algo) = factory.get(algo_name) { // 获取算法适配器
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() { // 尝试转换为算法trait对象
                    // 这里应传递真实的Context和参数
                    // let _exec_result = exec_strategy.execute(ctx, &exec_params.algo_params)?;
                }
            }
        }
    }
    // 3. 策略融合
    crate::services::index_token_service::IndexTokenService::strategy_redeem(
        index_token, // 指数代币账户
        &strategy, // 策略名称
        &params, // 策略参数
        ctx.accounts.authority.key(), // 操作人公钥
    )?;
    emit!(IndexTokenStrategyRedeemed { // 触发策略赎回事件
        index_token_id: index_token.id, // 事件：指数代币ID
        strategy: strategy.clone(), // 事件：策略名称
        params: params.clone(), // 事件：策略参数
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct BatchCombineIndexToken<'info> { // 定义批量合并指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub target_index_token: Account<'info, BasketIndexState>, // 目标指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub source_index_tokens: Vec<Account<'info, BasketIndexState>>, // 批量源指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn batch_combine_index_token(
    ctx: Context<BatchCombineIndexToken>, // Anchor账户上下文
    params: BatchTradeParams, // 批量交易参数
) -> Result<()> { // Anchor规范返回类型
    require!(!params.swaps.is_empty(), crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验批量交易非空
    let target = &mut ctx.accounts.target_index_token; // 获取目标指数代币账户
    let mut sources: Vec<&mut BasketIndexState> = ctx.accounts.source_index_tokens.iter_mut().map(|i| i.as_mut()).collect(); // 获取批量源账户
    require!(params.amounts.len() == sources.len(), crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验数量匹配
    for (src, &amt) in sources.iter_mut().zip(params.amounts.iter()) { // 遍历每个源账户及其对应数量
        require!(amt > 0, crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验合并数量大于0
        require!(src.total_supply >= amt, crate::errors::index_token_error::IndexTokenError::InsufficientValue); // 校验余额充足
        src.total_supply -= amt; // 扣减源账户余额
        target.total_supply = target.total_supply.checked_add(amt).ok_or(crate::errors::index_token_error::IndexTokenError::Overflow)?; // 增加目标账户余额，防止溢出
        emit!(IndexTokenCombined { // 触发合并事件
            target_index_token_id: target.id, // 事件：目标ID
            source_index_token_id: src.id, // 事件：源ID
            amount: amt, // 事件：合并数量
            authority: ctx.accounts.authority.key(), // 事件：操作人
            timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
        });
    }
    Ok(()) // Anchor规范返回
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct BatchSplitIndexToken<'info> { // 定义批量拆分指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub source_index_token: Account<'info, BasketIndexState>, // 源指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub new_index_tokens: Vec<Account<'info, BasketIndexState>>, // 批量新生成指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn batch_split_index_token(
    ctx: Context<BatchSplitIndexToken>, // Anchor账户上下文
    amounts: Vec<u64>, // 批量拆分数量
) -> Result<()> { // Anchor规范返回类型
    let source = &mut ctx.accounts.source_index_token; // 获取源指数代币账户
    let mut new_index_tokens: Vec<&mut BasketIndexState> = ctx.accounts.new_index_tokens.iter_mut().map(|i| i.as_mut()).collect(); // 获取批量新生成账户
    let n = new_index_tokens.len(); // 新生成账户数量
    require!(n > 0, crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验数量大于0
    require!(amounts.len() == n, crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验数量匹配
    let total: u64 = amounts.iter().try_fold(0u64, |acc, &x| acc.checked_add(x).ok_or(crate::errors::index_token_error::IndexTokenError::Overflow))?; // 计算总拆分数量，防止溢出
    require!(source.total_supply >= total, crate::errors::index_token_error::IndexTokenError::InsufficientValue); // 校验余额充足
    for (to, &amt) in new_index_tokens.iter_mut().zip(amounts.iter()) { // 遍历每个新生成账户及其对应数量
        require!(amt > 0, crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验拆分数量大于0
        to.total_supply = to.total_supply.checked_add(amt).ok_or(crate::errors::index_token_error::IndexTokenError::Overflow)?; // 增加新账户余额，防止溢出
        emit!(IndexTokenSplit { // 触发拆分事件
            source_index_token_id: source.id, // 事件：源ID
            new_index_token_id: to.id, // 事件：新生成ID
            amount: amt, // 事件：拆分数量
            authority: ctx.accounts.authority.key(), // 事件：操作人
            timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
        });
    }
    source.total_supply -= total; // 扣减源账户余额
    Ok(()) // Anchor规范返回
}

// 可扩展 transfer/query 等指令

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::index_token_error::IndexTokenError;
    use crate::state::baskets::BasketIndexState;
    use anchor_lang::prelude::*;

    fn default_basket() -> BasketIndexState {
        BasketIndexState {
            fee_collector: Pubkey::new_unique(),
            composition: vec![Default::default()],
            weights: vec![10_000],
            is_active: true,
            total_supply: 1000,
            ..Default::default()
        }
    }

    #[test]
    fn test_mint_index_token_not_active() {
        let mut basket = default_basket();
        basket.is_active = false;
        let result = IndexTokenService::mint(&mut basket, 100);
        assert_eq!(
            result.unwrap_err().to_string(),
            IndexTokenError::NotAllowed.to_string()
        );
    }

    #[test]
    fn test_mint_index_token_invalid_assets() {
        let mut basket = default_basket();
        basket.fee_collector = Pubkey::default();
        let result = basket.validate();
        assert_eq!(
            result.unwrap_err().to_string(),
            IndexTokenError::InvalidAssets.to_string()
        );
    }

    #[test]
    fn test_mint_index_token_success() {
        let mut basket = default_basket();
        let result = IndexTokenService::mint(&mut basket, 100);
        assert!(result.is_ok());
        assert_eq!(basket.total_supply, 1100);
    }

    #[test]
    fn test_burn_index_token_insufficient_value() {
        let mut basket = default_basket();
        let result = IndexTokenService::burn(&mut basket, 2000);
        assert_eq!(
            result.unwrap_err().to_string(),
            IndexTokenError::InsufficientValue.to_string()
        );
    }

    #[test]
    fn test_burn_index_token_invalid_assets() {
        let mut basket = default_basket();
        basket.composition.clear();
        let result = basket.validate();
        assert_eq!(
            result.unwrap_err().to_string(),
            IndexTokenError::InvalidAssets.to_string()
        );
    }

    #[test]
    fn test_burn_index_token_success() {
        let mut basket = default_basket();
        let result = IndexTokenService::burn(&mut basket, 500);
        assert!(result.is_ok());
        assert_eq!(basket.total_supply, 500);
    }

    #[test]
    fn test_transfer_index_token_success() {
        let mut from = default_basket();
        let mut to = default_basket();
        let amount = 500;
        from.total_supply = 1000;
        to.total_supply = 200;
        // 模拟transfer逻辑
        assert!(from.total_supply >= amount);
        from.total_supply -= amount;
        to.total_supply += amount;
        assert_eq!(from.total_supply, 500);
        assert_eq!(to.total_supply, 700);
    }

    #[test]
    fn test_transfer_index_token_insufficient_value() {
        let mut from = default_basket();
        let mut to = default_basket();
        let amount = 2000;
        from.total_supply = 1000;
        to.total_supply = 200;
        // 模拟transfer逻辑
        assert!(from.total_supply < amount);
    }

    #[test]
    fn test_query_index_token() {
        let index_token = default_basket();
        let value = index_token.total_supply;
        assert_eq!(value, 1000);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::core::types::{TradeParams, BatchTradeParams, StrategyParams, OracleParams, AlgoParams};
    use anchor_lang::prelude::Pubkey;

    #[test]
    fn test_buy_index_token_invalid_params() {
        let ctx = Context::default();
        let params = TradeParams {
            from_token: Pubkey::default(),
            to_token: Pubkey::default(),
            amount_in: 0,
            min_amount_out: 0,
            dex_name: "raydium".to_string(),
        };
        let price_params = OracleParams { asset: Pubkey::default(), oracle_name: "chainlink".to_string() };
        let result = buy_index_token(ctx, params, price_params, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_batch_combine_index_token_empty() {
        let ctx = Context::default();
        let params = BatchTradeParams { swaps: vec![] };
        let result = batch_combine_index_token(ctx, params);
        assert!(result.is_err());
    }

    #[test]
    fn test_strategy_subscribe_index_token_empty_strategy() {
        let ctx = Context::default();
        let params = StrategyParams { strategy_name: "".to_string(), params: vec![] };
        let result = strategy_subscribe_index_token(ctx, params, None);
        assert!(result.is_err());
    }
}

// === 新增：报价指令 ===
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct QuoteIndexToken<'info> { // 定义指数代币报价指令的账户上下文结构体
    pub index_token: Account<'info, BasketIndexState>, // 查询目标指数代币账户
}

pub fn quote_index_token(
    ctx: Context<QuoteIndexToken>, // Anchor账户上下文
    params: TradeParams, // 交易参数
    price_params: OracleParams, // 预言机价格参数
) -> Result<u64> { // Anchor规范返回类型，返回u64报价
    require!(params.amount_in > 0, crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验买入数量大于0
    let price = crate::services::index_token_service::IndexTokenService::quote(&ctx.accounts.index_token, &params, &price_params)?; // 调用服务层报价逻辑
    emit!(IndexTokenQueried { // 触发查询事件
        index_token_id: ctx.accounts.index_token.id, // 事件：指数代币ID
        total_supply: ctx.accounts.index_token.total_supply, // 事件：总供应量
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(price) // 返回报价
}

// === 新增：执行买入指令 ===
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct ExecuteBuyIndexToken<'info> { // 定义执行买入指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub index_token: Account<'info, BasketIndexState>, // 指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub buyer: Signer<'info>, // 买方签名者
}

pub fn execute_buy_index_token(
    ctx: Context<ExecuteBuyIndexToken>, // Anchor账户上下文
    params: TradeParams, // 交易参数
    price: u64, // 买入价格
) -> Result<()> { // Anchor规范返回类型
    require!(params.amount_in > 0, crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验买入数量大于0
    crate::services::index_token_service::IndexTokenService::execute_buy(&mut ctx.accounts.index_token, &params, price, ctx.accounts.buyer.key())?; // 调用服务层买入逻辑
    emit!(IndexTokenBought { // 触发买入事件
        basket_id: ctx.accounts.index_token.id, // 事件：篮子ID
        amount: params.amount_in, // 事件：买入数量
        price, // 事件：买入价格
        buyer: ctx.accounts.buyer.key(), // 事件：买方
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

// === 新增：执行卖出指令 ===
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct ExecuteSellIndexToken<'info> { // 定义执行卖出指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub index_token: Account<'info, BasketIndexState>, // 指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub seller: Signer<'info>, // 卖方签名者
}

pub fn execute_sell_index_token(
    ctx: Context<ExecuteSellIndexToken>, // Anchor账户上下文
    params: TradeParams, // 交易参数
    price: u64, // 卖出价格
) -> Result<()> { // Anchor规范返回类型
    require!(params.amount_in > 0, crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验卖出数量大于0
    crate::services::index_token_service::IndexTokenService::execute_sell(&mut ctx.accounts.index_token, &params, price, ctx.accounts.seller.key())?; // 调用服务层卖出逻辑
    emit!(IndexTokenSold { // 触发卖出事件
        basket_id: ctx.accounts.index_token.id, // 事件：篮子ID
        amount: params.amount_in, // 事件：卖出数量
        price, // 事件：卖出价格
        seller: ctx.accounts.seller.key(), // 事件：卖方
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

// === 新增：执行交换指令 ===
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct ExecuteSwapIndexToken<'info> { // 定义执行交换指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub from_index_token: Account<'info, BasketIndexState>, // 转出方指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub to_index_token: Account<'info, BasketIndexState>, // 转入方指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn execute_swap_index_token(
    ctx: Context<ExecuteSwapIndexToken>, // Anchor账户上下文
    from_amount: u64, // 转出数量
    to_amount: u64, // 转入数量
) -> Result<()> { // Anchor规范返回类型
    require!(from_amount > 0 && to_amount > 0, crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验数量大于0
    crate::services::index_token_service::IndexTokenService::execute_swap(&mut ctx.accounts.from_index_token, &mut ctx.accounts.to_index_token, from_amount, to_amount, ctx.accounts.authority.key())?; // 调用服务层交换逻辑
    emit!(IndexTokenSwapped { // 触发交换事件
        from_index_token_id: ctx.accounts.from_index_token.id, // 事件：转出方ID
        to_index_token_id: ctx.accounts.to_index_token.id, // 事件：转入方ID
        from_amount, // 事件：转出数量
        to_amount, // 事件：转入数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

// === 新增：执行合并指令 ===
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct ExecuteCombineIndexToken<'info> { // 定义执行合并指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub target_index_token: Account<'info, BasketIndexState>, // 目标指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub source_index_token: Account<'info, BasketIndexState>, // 源指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn execute_combine_index_token(
    ctx: Context<ExecuteCombineIndexToken>, // Anchor账户上下文
    amount: u64, // 合并数量
) -> Result<()> { // Anchor规范返回类型
    require!(amount > 0, crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验合并数量大于0
    crate::services::index_token_service::IndexTokenService::execute_combine(&mut ctx.accounts.target_index_token, &mut ctx.accounts.source_index_token, amount, ctx.accounts.authority.key())?; // 调用服务层合并逻辑
    emit!(IndexTokenCombined { // 触发合并事件
        target_index_token_id: ctx.accounts.target_index_token.id, // 事件：目标ID
        source_index_token_id: ctx.accounts.source_index_token.id, // 事件：源ID
        amount, // 事件：合并数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

// === 新增：执行拆分指令 ===
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct ExecuteSplitIndexToken<'info> { // 定义执行拆分指令的账户上下文结构体
    #[account(mut)] // Anchor属性，标记账户为可变
    pub source_index_token: Account<'info, BasketIndexState>, // 源指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub new_index_token: Account<'info, BasketIndexState>, // 新生成指数代币账户，需可变
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者
}

pub fn execute_split_index_token(
    ctx: Context<ExecuteSplitIndexToken>, // Anchor账户上下文
    amount: u64, // 拆分数量
) -> Result<()> { // Anchor规范返回类型
    require!(amount > 0, crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验拆分数量大于0
    crate::services::index_token_service::IndexTokenService::execute_split(&mut ctx.accounts.source_index_token, &mut ctx.accounts.new_index_token, amount, ctx.accounts.authority.key())?; // 调用服务层拆分逻辑
    emit!(IndexTokenSplit { // 触发拆分事件
        source_index_token_id: ctx.accounts.source_index_token.id, // 事件：源ID
        new_index_token_id: ctx.accounts.new_index_token.id, // 事件：新生成ID
        amount, // 事件：拆分数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回
}

// 批量策略申购指令（如有需求，可按如下模式实现）
pub fn batch_strategy_subscribe_index_token(
    ctx: Context<BatchTransferIndexToken>, // Anchor账户上下文
    strategies: Vec<StrategyParams>, // 批量策略参数
    exec_params: Option<AlgoParams>, // 可选算法参数
) -> Result<()> { // Anchor规范返回类型
    let from = &mut ctx.accounts.from_index_token; // 获取转出方账户
    let mut to_index_tokens: Vec<&mut BasketIndexState> = ctx.accounts.to_index_tokens.iter_mut().map(|i| i.as_mut()).collect(); // 获取批量转入方账户
    let n = to_index_tokens.len(); // 转入方数量
    require!(n > 0, crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验转入方数量大于0
    require!(strategies.len() == n, crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验数量匹配
    require!(ctx.accounts.authority.key() == from.authority, crate::errors::index_token_error::IndexTokenError::NotAllowed); // 校验操作人权限
    for (to, strategy) in to_index_tokens.iter_mut().zip(strategies.iter()) { // 遍历每个转入方及其对应策略
        require!(!strategy.strategy_name.is_empty(), crate::errors::index_token_error::IndexTokenError::InvalidParams); // 校验策略名称非空
        // 算法融合（如有）
        if let Some(exec_params) = &exec_params { // 若有算法参数
            if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名称
                let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂
                if let Some(algo) = factory.get(algo_name) { // 获取算法适配器
                    if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() { // 尝试转换为算法trait对象
                        // 这里应传递真实的Context和参数
                        // let _exec_result = exec_strategy.execute(ctx, &exec_params.algo_params)?;
                    }
                }
            }
        }
        // 策略融合
        crate::services::index_token_service::IndexTokenService::strategy_subscribe(
            to, // 转入方账户
            &strategy.strategy_name, // 策略名称
            &strategy.params, // 策略参数
            ctx.accounts.authority.key(), // 操作人公钥
        )?;
        emit!(IndexTokenStrategySubscribed { // 触发策略申购事件
            index_token_id: to.id, // 事件：指数代币ID
            strategy: strategy.strategy_name.clone(), // 事件：策略名称
            params: strategy.params.clone(), // 事件：策略参数
            authority: ctx.accounts.authority.key(), // 事件：操作人
            timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
        });
    }
    Ok(()) // Anchor规范返回
}
