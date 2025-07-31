//! DEX instruction set: register, query, switch DEX/AMM adapters (PDA持久化/权限校验/事件日志)
use anchor_lang::prelude::*; // 引入Anchor框架预导入模块，包含Solana程序开发常用类型与宏
use crate::account_models::dex_registry_account::{DexRegistryAccount, DexMeta}; // 引入DEX注册表账户与元数据结构体

#[event] // Anchor事件宏，自动生成链上事件日志结构体
pub struct DexRegistered { // 定义DEX注册事件结构体
    pub name: String, // 注册DEX名称，类型安全
    pub creator: Pubkey, // 创建人公钥，类型安全
    pub timestamp: i64, // 注册时间戳，链上可追溯
}

#[event] // Anchor事件宏，自动生成链上事件日志结构体
pub struct DexSwitched { // 定义DEX切换事件结构体
    pub from: String, // 原DEX名称，类型安全
    pub to: String, // 新DEX名称，类型安全
    pub authority: Pubkey, // 操作人公钥，类型安全
    pub timestamp: i64, // 切换时间戳，链上可追溯
}

#[event] // Anchor事件宏，自动生成链上事件日志结构体
pub struct DexRegistryInitialized { // 定义DEX注册表初始化事件结构体
    pub authority: Pubkey, // 初始化人公钥，类型安全
    pub timestamp: i64, // 初始化时间戳，链上可追溯
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct RegisterDex<'info> { // 定义注册DEX指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut, has_one = authority)] // Anchor属性，标记账户为可变，校验PDA和权限
    pub registry: Account<'info, DexRegistryAccount>, // DEX注册表账户，需可变，类型安全
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)] // 派生Anchor序列化/反序列化、克隆、调试特性，便于跨链/链上数据传递
pub struct RegisterDexParams { // 定义注册DEX参数结构体
    pub name: String, // DEX名称，类型安全
}

pub fn register_dex(
    ctx: Context<RegisterDex>, // Anchor账户上下文，自动校验权限与生命周期
    params: RegisterDexParams, // 注册参数，类型安全
) -> anchor_lang::Result<()> { // Anchor规范返回类型
    let registry = &mut ctx.accounts.registry; // 获取可变DEX注册表账户，生命周期由Anchor自动管理
    let authority = ctx.accounts.authority.key(); // 获取操作人公钥
    crate::services::dex_service::DexService::register(
        registry, // DEX注册表账户
        params.name.clone(), // DEX名称
        authority, // 操作人公钥
    )?; // 调用服务层注册逻辑
    emit!(DexRegistered { // 触发DEX注册事件，链上可追溯
        name: params.name, // 事件：DEX名称
        creator: authority, // 事件：创建人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct QueryDex<'info> { // 定义查询DEX指令的账户上下文结构体，'info生命周期由Anchor自动推断
    pub registry: Account<'info, DexRegistryAccount>, // 只读DEX注册表账户，类型安全
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)] // 派生Anchor序列化/反序列化、克隆、调试特性，便于跨链/链上数据传递
pub struct QueryDexParams { // 定义查询DEX参数结构体
    pub name: String, // DEX名称，类型安全
}

pub fn query_dex(
    ctx: Context<QueryDex>, // Anchor账户上下文，自动校验权限与生命周期
    params: QueryDexParams, // 查询参数，类型安全
) -> anchor_lang::Result<DexMeta> { // Anchor规范返回类型，返回DEX元数据
    let registry = &ctx.accounts.registry; // 获取只读DEX注册表账户，生命周期由Anchor自动管理
    let meta = crate::services::dex_service::DexService::query(
        registry, // DEX注册表账户
        &params.name, // DEX名称
    )?; // 调用服务层查询逻辑，返回元数据
    Ok(meta) // 返回DEX元数据，Anchor自动处理生命周期
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct SwitchDex<'info> { // 定义切换DEX指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut, has_one = authority)] // Anchor属性，标记账户为可变，校验PDA和权限
    pub registry: Account<'info, DexRegistryAccount>, // DEX注册表账户，需可变，类型安全
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)] // 派生Anchor序列化/反序列化、克隆、调试特性，便于跨链/链上数据传递
pub struct SwitchDexParams { // 定义切换DEX参数结构体
    pub from: String, // 原DEX名称，类型安全
    pub to: String, // 新DEX名称，类型安全
}

pub fn switch_dex(
    ctx: Context<SwitchDex>, // Anchor账户上下文，自动校验权限与生命周期
    params: SwitchDexParams, // 切换参数，类型安全
) -> anchor_lang::Result<()> { // Anchor规范返回类型
    let registry = &mut ctx.accounts.registry; // 获取可变DEX注册表账户，生命周期由Anchor自动管理
    let authority = ctx.accounts.authority.key(); // 获取操作人公钥
    crate::services::dex_service::DexService::switch(
        registry, // DEX注册表账户
        &params.from, // 原DEX名称
        &params.to, // 新DEX名称
    )?; // 调用服务层切换逻辑
    emit!(DexSwitched { // 触发DEX切换事件，链上可追溯
        from: params.from, // 事件：原DEX名称
        to: params.to, // 事件：新DEX名称
        authority, // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct InitDexRegistry<'info> { // 定义DEX注册表初始化指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(
        init, // Anchor属性，指示账户初始化
        payer = authority, // 由authority账户支付租金
        space = DexRegistryAccount::INIT_SPACE, // 分配账户空间
        seeds = [b"dex_registry", authority.key().as_ref()], // PDA种子，确保唯一性
        bump // 自动推断bump种子
    )]
    pub registry: Account<'info, DexRegistryAccount>, // 新建DEX注册表账户，类型安全
    #[account(mut)] // Anchor属性，标记账户为可变，自动校验签名
    pub authority: Signer<'info>, // 初始化人签名者，类型安全
    pub system_program: Program<'info, System>, // 系统程序，Anchor自动校验
}

pub fn init_dex_registry(
    ctx: Context<InitDexRegistry>, // Anchor账户上下文，自动校验权限与生命周期
) -> anchor_lang::Result<()> { // Anchor规范返回类型
    let registry = &mut ctx.accounts.registry; // 获取可变DEX注册表账户，生命周期由Anchor自动管理
    let authority = ctx.accounts.authority.key(); // 获取初始化人公钥
    let bump = *ctx.bumps.get("registry").unwrap(); // 获取PDA bump种子
    registry.base = crate::state::common::BaseAccount::new(authority, bump)?; // 初始化BaseAccount，权限与生命周期安全
    registry.dexes = Vec::new(); // 初始化DEX列表为空
    emit!(DexRegistryInitialized { // 触发DEX注册表初始化事件，链上可追溯
        authority, // 事件：初始化人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
} 