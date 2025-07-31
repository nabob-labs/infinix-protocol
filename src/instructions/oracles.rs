//! Oracle instruction set: register, query, switch oracle adapters (PDA持久化/权限校验/事件日志)
use anchor_lang::prelude::*; // 引入Anchor框架预导入模块，包含Solana程序开发常用类型与宏
use crate::account_models::oracle_registry_account::{OracleRegistryAccount, OracleMeta}; // 引入Oracle注册表账户与元数据结构体

#[event] // Anchor事件宏，自动生成链上事件日志结构体
pub struct OracleRegistered { // 定义Oracle注册事件结构体
    pub name: String, // 注册Oracle名称，类型安全
    pub creator: Pubkey, // 创建人公钥，类型安全
    pub timestamp: i64, // 注册时间戳，链上可追溯
}

#[event] // Anchor事件宏，自动生成链上事件日志结构体
pub struct OracleSwitched { // 定义Oracle切换事件结构体
    pub from: String, // 原Oracle名称，类型安全
    pub to: String, // 新Oracle名称，类型安全
    pub authority: Pubkey, // 操作人公钥，类型安全
    pub timestamp: i64, // 切换时间戳，链上可追溯
}

#[event] // Anchor事件宏，自动生成链上事件日志结构体
pub struct OracleRegistryInitialized { // 定义Oracle注册表初始化事件结构体
    pub authority: Pubkey, // 初始化人公钥，类型安全
    pub timestamp: i64, // 初始化时间戳，链上可追溯
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct RegisterOracle<'info> { // 定义注册Oracle指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut, has_one = authority)] // Anchor属性，标记账户为可变，校验PDA和权限
    pub registry: Account<'info, OracleRegistryAccount>, // Oracle注册表账户，需可变，类型安全
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)] // 派生Anchor序列化/反序列化、克隆、调试特性，便于跨链/链上数据传递
pub struct RegisterOracleParams { // 定义注册Oracle参数结构体
    pub name: String, // Oracle名称，类型安全
}

pub fn register_oracle(
    ctx: Context<RegisterOracle>, // Anchor账户上下文，自动校验权限与生命周期
    params: RegisterOracleParams, // 注册参数，类型安全
) -> anchor_lang::Result<()> { // Anchor规范返回类型
    let registry = &mut ctx.accounts.registry; // 获取可变Oracle注册表账户，生命周期由Anchor自动管理
    let authority = ctx.accounts.authority.key(); // 获取操作人公钥
    crate::services::oracle_service::OracleService::register(
        registry, // Oracle注册表账户
        params.name.clone(), // Oracle名称
        authority, // 操作人公钥
    )?; // 调用服务层注册逻辑
    emit!(OracleRegistered { // 触发Oracle注册事件，链上可追溯
        name: params.name, // 事件：Oracle名称
        creator: authority, // 事件：创建人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct QueryOracle<'info> { // 定义查询Oracle指令的账户上下文结构体，'info生命周期由Anchor自动推断
    pub registry: Account<'info, OracleRegistryAccount>, // 只读Oracle注册表账户，类型安全
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)] // 派生Anchor序列化/反序列化、克隆、调试特性，便于跨链/链上数据传递
pub struct QueryOracleParams { // 定义查询Oracle参数结构体
    pub name: String, // Oracle名称，类型安全
}

pub fn query_oracle(
    ctx: Context<QueryOracle>, // Anchor账户上下文，自动校验权限与生命周期
    params: QueryOracleParams, // 查询参数，类型安全
) -> anchor_lang::Result<OracleMeta> { // Anchor规范返回类型，返回Oracle元数据
    let registry = &ctx.accounts.registry; // 获取只读Oracle注册表账户，生命周期由Anchor自动管理
    let meta = crate::services::oracle_service::OracleService::query(
        registry, // Oracle注册表账户
        &params.name, // Oracle名称
    )?; // 调用服务层查询逻辑，返回元数据
    Ok(meta) // 返回Oracle元数据，Anchor自动处理生命周期
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct SwitchOracle<'info> { // 定义切换Oracle指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut, has_one = authority)] // Anchor属性，标记账户为可变，校验PDA和权限
    pub registry: Account<'info, OracleRegistryAccount>, // Oracle注册表账户，需可变，类型安全
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)] // 派生Anchor序列化/反序列化、克隆、调试特性，便于跨链/链上数据传递
pub struct SwitchOracleParams { // 定义切换Oracle参数结构体
    pub from: String, // 原Oracle名称，类型安全
    pub to: String, // 新Oracle名称，类型安全
}

pub fn switch_oracle(
    ctx: Context<SwitchOracle>, // Anchor账户上下文，自动校验权限与生命周期
    params: SwitchOracleParams, // 切换参数，类型安全
) -> anchor_lang::Result<()> { // Anchor规范返回类型
    let registry = &mut ctx.accounts.registry; // 获取可变Oracle注册表账户，生命周期由Anchor自动管理
    let authority = ctx.accounts.authority.key(); // 获取操作人公钥
    crate::services::oracle_service::OracleService::switch(
        registry, // Oracle注册表账户
        &params.from, // 原Oracle名称
        &params.to, // 新Oracle名称
    )?; // 调用服务层切换逻辑
    emit!(OracleSwitched { // 触发Oracle切换事件，链上可追溯
        from: params.from, // 事件：原Oracle名称
        to: params.to, // 事件：新Oracle名称
        authority, // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct InitOracleRegistry<'info> { // 定义Oracle注册表初始化指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(
        init, // Anchor属性，指示账户初始化
        payer = authority, // 由authority账户支付租金
        space = OracleRegistryAccount::INIT_SPACE, // 分配账户空间
        seeds = [b"oracle_registry", authority.key().as_ref()], // PDA种子，确保唯一性
        bump // 自动推断bump种子
    )]
    pub registry: Account<'info, OracleRegistryAccount>, // 新建Oracle注册表账户，类型安全
    #[account(mut)] // Anchor属性，标记账户为可变，自动校验签名
    pub authority: Signer<'info>, // 初始化人签名者，类型安全
    pub system_program: Program<'info, System>, // 系统程序，Anchor自动校验
}

pub fn init_oracle_registry(
    ctx: Context<InitOracleRegistry>, // Anchor账户上下文，自动校验权限与生命周期
) -> anchor_lang::Result<()> { // Anchor规范返回类型
    let registry = &mut ctx.accounts.registry; // 获取可变Oracle注册表账户，生命周期由Anchor自动管理
    let authority = ctx.accounts.authority.key(); // 获取初始化人公钥
    let bump = *ctx.bumps.get("registry").unwrap(); // 获取PDA bump种子
    registry.base = crate::state::common::BaseAccount::new(authority, bump)?; // 初始化BaseAccount，权限与生命周期安全
    registry.oracles = Vec::new(); // 初始化Oracle列表为空
    emit!(OracleRegistryInitialized { // 触发Oracle注册表初始化事件，链上可追溯
        authority, // 事件：初始化人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
} 