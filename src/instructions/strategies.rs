//! Strategy instruction set: register, query, switch strategies (PDA持久化/权限校验/事件日志)
use anchor_lang::prelude::*; // 引入Anchor框架预导入模块，包含Solana程序开发常用类型与宏
use crate::accounts::strategy_registry_account::{StrategyRegistryAccount, StrategyMeta}; // 引入策略注册表账户与元数据结构体
use crate::strategies::strategy_registry::StrategyConfig; // 引入策略配置结构体

#[event] // Anchor事件宏，自动生成链上事件日志结构体
pub struct StrategyRegistered { // 定义策略注册事件结构体
    pub id: u64, // 策略ID，类型安全
    pub creator: Pubkey, // 创建人公钥，类型安全
    pub timestamp: i64, // 注册时间戳，链上可追溯
}

#[event] // Anchor事件宏，自动生成链上事件日志结构体
pub struct StrategySwitched { // 定义策略切换事件结构体
    pub from: u64, // 原策略ID，类型安全
    pub to: u64, // 新策略ID，类型安全
    pub authority: Pubkey, // 操作人公钥，类型安全
    pub timestamp: i64, // 切换时间戳，链上可追溯
}

#[event] // Anchor事件宏，自动生成链上事件日志结构体
pub struct StrategyRegistryInitialized { // 定义策略注册表初始化事件结构体
    pub authority: Pubkey, // 初始化人公钥，类型安全
    pub timestamp: i64, // 初始化时间戳，链上可追溯
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct RegisterStrategy<'info> { // 定义注册策略指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut, has_one = authority)] // Anchor属性，标记账户为可变，校验PDA和权限
    pub registry: Account<'info, StrategyRegistryAccount>, // 策略注册表账户，需可变，类型安全
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)] // 派生Anchor序列化/反序列化、克隆、调试特性，便于跨链/链上数据传递
pub struct RegisterStrategyParams { // 定义注册策略参数结构体
    pub id: u64, // 策略ID，类型安全
    pub config: StrategyConfig, // 策略配置，类型安全
}

pub fn register_strategy(
    ctx: Context<RegisterStrategy>, // Anchor账户上下文，自动校验权限与生命周期
    params: RegisterStrategyParams, // 注册参数，类型安全
) -> Result<u64> { // Anchor规范返回类型，返回策略ID
    let registry = &mut ctx.accounts.registry; // 获取可变策略注册表账户，生命周期由Anchor自动管理
    let authority = ctx.accounts.authority.key(); // 获取操作人公钥
    let id = crate::services::strategy_service::StrategyService::register(
        registry, // 策略注册表账户
        params.id, // 策略ID
        params.config.clone(), // 策略配置
        authority, // 操作人公钥
    )?; // 调用服务层注册逻辑，返回策略ID
    emit!(StrategyRegistered { // 触发策略注册事件，链上可追溯
        id: params.id, // 事件：策略ID
        creator: authority, // 事件：创建人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(id) // 返回策略ID，Anchor自动处理生命周期
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct QueryStrategy<'info> { // 定义查询策略指令的账户上下文结构体，'info生命周期由Anchor自动推断
    pub registry: Account<'info, StrategyRegistryAccount>, // 只读策略注册表账户，类型安全
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)] // 派生Anchor序列化/反序列化、克隆、调试特性，便于跨链/链上数据传递
pub struct QueryStrategyParams { // 定义查询策略参数结构体
    pub strategy_id: u64, // 策略ID，类型安全
}

pub fn query_strategy(
    ctx: Context<QueryStrategy>, // Anchor账户上下文，自动校验权限与生命周期
    params: QueryStrategyParams, // 查询参数，类型安全
) -> Result<StrategyMeta> { // Anchor规范返回类型，返回策略元数据
    let registry = &ctx.accounts.registry; // 获取只读策略注册表账户，生命周期由Anchor自动管理
    let meta = crate::services::strategy_service::StrategyService::query(
        registry, // 策略注册表账户
        params.strategy_id, // 策略ID
    )?; // 调用服务层查询逻辑，返回元数据
    Ok(meta) // 返回策略元数据，Anchor自动处理生命周期
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct SwitchStrategy<'info> { // 定义切换策略指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut, has_one = authority)] // Anchor属性，标记账户为可变，校验PDA和权限
    pub registry: Account<'info, StrategyRegistryAccount>, // 策略注册表账户，需可变，类型安全
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)] // 派生Anchor序列化/反序列化、克隆、调试特性，便于跨链/链上数据传递
pub struct SwitchStrategyParams { // 定义切换策略参数结构体
    pub from: u64, // 原策略ID，类型安全
    pub to: u64, // 新策略ID，类型安全
}

pub fn switch_strategy(
    ctx: Context<SwitchStrategy>, // Anchor账户上下文，自动校验权限与生命周期
    params: SwitchStrategyParams, // 切换参数，类型安全
) -> Result<()> { // Anchor规范返回类型
    let registry = &mut ctx.accounts.registry; // 获取可变策略注册表账户，生命周期由Anchor自动管理
    let authority = ctx.accounts.authority.key(); // 获取操作人公钥
    crate::services::strategy_service::StrategyService::switch(
        registry, // 策略注册表账户
        params.from, // 原策略ID
        params.to, // 新策略ID
    )?; // 调用服务层切换逻辑
    emit!(StrategySwitched { // 触发策略切换事件，链上可追溯
        from: params.from, // 事件：原策略ID
        to: params.to, // 事件：新策略ID
        authority, // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct InitStrategyRegistry<'info> { // 定义初始化策略注册表指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(init, payer = authority, space = 8 + StrategyRegistryAccount::LEN)] // Anchor属性，初始化账户，指定付费者和空间
    pub registry: Account<'info, StrategyRegistryAccount>, // 策略注册表账户，需初始化，类型安全
    #[account(mut)] // Anchor属性，标记账户为可变
    pub authority: Signer<'info>, // 操作人签名者，类型安全
    pub system_program: Program<'info, System>, // 系统程序账户，Anchor自动校验
}

pub fn init_strategy_registry(
    ctx: Context<InitStrategyRegistry>, // Anchor账户上下文，自动校验权限与生命周期
) -> Result<()> { // Anchor规范返回类型
    let registry = &mut ctx.accounts.registry; // 获取可变策略注册表账户，生命周期由Anchor自动管理
    let authority = ctx.accounts.authority.key(); // 获取操作人公钥
    crate::services::strategy_service::StrategyService::init_registry(
        registry, // 策略注册表账户
        authority, // 操作人公钥
    )?; // 调用服务层初始化逻辑
    emit!(StrategyRegistryInitialized { // 触发策略注册表初始化事件，链上可追溯
        authority, // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
} 