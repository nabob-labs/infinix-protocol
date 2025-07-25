//! Algorithm instruction set: register, query, switch algorithms (PDA持久化/权限校验/事件日志)
use anchor_lang::prelude::*; // 引入Anchor框架预导入模块，包含Solana程序开发常用类型与宏
use crate::accounts::algorithm_registry_account::{AlgorithmRegistryAccount, AlgorithmMeta}; // 引入算法注册表账户与元数据结构体
use crate::algorithms::algorithm_registry::AlgorithmRegistry; // 引入算法注册表全局管理器
use std::sync::Arc; // 引入Arc智能指针，用于多线程安全算法trait对象

#[event] // Anchor事件宏，自动生成链上事件日志结构体
pub struct AlgorithmRegistered { // 定义算法注册事件结构体
    pub name: String, // 注册算法名称，类型安全
    pub creator: Pubkey, // 创建人公钥，类型安全
    pub timestamp: i64, // 注册时间戳，链上可追溯
}

#[event] // Anchor事件宏，自动生成链上事件日志结构体
pub struct AlgorithmSwitched { // 定义算法切换事件结构体
    pub from: String, // 原算法名称，类型安全
    pub to: String, // 新算法名称，类型安全
    pub authority: Pubkey, // 操作人公钥，类型安全
    pub timestamp: i64, // 切换时间戳，链上可追溯
}

#[event] // Anchor事件宏，自动生成链上事件日志结构体
pub struct AlgorithmRegistryInitialized { // 定义算法注册表初始化事件结构体
    pub authority: Pubkey, // 初始化人公钥，类型安全
    pub timestamp: i64, // 初始化时间戳，链上可追溯
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct RegisterAlgorithm<'info> { // 定义注册算法指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut, has_one = authority)] // Anchor属性，标记账户为可变，校验PDA和权限
    pub registry: Account<'info, AlgorithmRegistryAccount>, // 算法注册表账户，需可变，类型安全
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)] // 派生Anchor序列化/反序列化、克隆、调试特性，便于跨链/链上数据传递
pub struct RegisterAlgorithmParams { // 定义注册算法参数结构体
    pub name: String, // 算法名称，类型安全
}

pub fn register_algorithm(
    ctx: Context<RegisterAlgorithm>, // Anchor账户上下文，自动校验权限与生命周期
    params: RegisterAlgorithmParams, // 注册参数，类型安全
) -> Result<()> { // Anchor规范返回类型
    let registry = &mut ctx.accounts.registry; // 获取可变算法注册表账户，生命周期由Anchor自动管理
    let authority = ctx.accounts.authority.key(); // 获取操作人公钥
    crate::services::algorithm_service::AlgorithmService::register(
        registry, // 算法注册表账户
        params.name.clone(), // 算法名称
        authority, // 操作人公钥
    )?; // 调用服务层注册逻辑
    emit!(AlgorithmRegistered { // 触发算法注册事件，链上可追溯
        name: params.name, // 事件：算法名称
        creator: authority, // 事件：创建人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct QueryAlgorithm<'info> { // 定义查询算法指令的账户上下文结构体，'info生命周期由Anchor自动推断
    pub registry: Account<'info, AlgorithmRegistryAccount>, // 只读算法注册表账户，类型安全
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)] // 派生Anchor序列化/反序列化、克隆、调试特性，便于跨链/链上数据传递
pub struct QueryAlgorithmParams { // 定义查询算法参数结构体
    pub name: String, // 算法名称，类型安全
}

pub fn query_algorithm(
    ctx: Context<QueryAlgorithm>, // Anchor账户上下文，自动校验权限与生命周期
    params: QueryAlgorithmParams, // 查询参数，类型安全
) -> Result<AlgorithmMeta> { // Anchor规范返回类型，返回算法元数据
    let registry = &ctx.accounts.registry; // 获取只读算法注册表账户，生命周期由Anchor自动管理
    let meta = crate::services::algorithm_service::AlgorithmService::query(
        registry, // 算法注册表账户
        &params.name, // 算法名称
    )?; // 调用服务层查询逻辑，返回元数据
    Ok(meta) // 返回算法元数据，Anchor自动处理生命周期
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct SwitchAlgorithm<'info> { // 定义切换算法指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut, has_one = authority)] // Anchor属性，标记账户为可变，校验PDA和权限
    pub registry: Account<'info, AlgorithmRegistryAccount>, // 算法注册表账户，需可变，类型安全
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)] // 派生Anchor序列化/反序列化、克隆、调试特性，便于跨链/链上数据传递
pub struct SwitchAlgorithmParams { // 定义切换算法参数结构体
    pub from: String, // 原算法名称，类型安全
    pub to: String, // 新算法名称，类型安全
}

pub fn switch_algorithm(
    ctx: Context<SwitchAlgorithm>, // Anchor账户上下文，自动校验权限与生命周期
    params: SwitchAlgorithmParams, // 切换参数，类型安全
) -> Result<()> { // Anchor规范返回类型
    let registry = &mut ctx.accounts.registry; // 获取可变算法注册表账户，生命周期由Anchor自动管理
    let authority = ctx.accounts.authority.key(); // 获取操作人公钥
    crate::services::algorithm_service::AlgorithmService::switch(
        registry, // 算法注册表账户
        &params.from, // 原算法名称
        &params.to, // 新算法名称
    )?; // 调用服务层切换逻辑
    emit!(AlgorithmSwitched { // 触发算法切换事件，链上可追溯
        from: params.from, // 事件：原算法名称
        to: params.to, // 事件：新算法名称
        authority, // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct InitAlgorithmRegistry<'info> { // 定义算法注册表初始化指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(
        init, // Anchor属性，指示账户初始化
        payer = authority, // 由authority账户支付租金
        space = AlgorithmRegistryAccount::INIT_SPACE, // 分配账户空间
        seeds = [b"algorithm_registry", authority.key().as_ref()], // PDA种子，确保唯一性
        bump // 自动推断bump种子
    )]
    pub registry: Account<'info, AlgorithmRegistryAccount>, // 新建算法注册表账户，类型安全
    #[account(mut)] // Anchor属性，标记账户为可变，自动校验签名
    pub authority: Signer<'info>, // 初始化人签名者，类型安全
    pub system_program: Program<'info, System>, // 系统程序，Anchor自动校验
}

pub fn init_algorithm_registry(
    ctx: Context<InitAlgorithmRegistry>, // Anchor账户上下文，自动校验权限与生命周期
) -> Result<()> { // Anchor规范返回类型
    let registry = &mut ctx.accounts.registry; // 获取可变算法注册表账户，生命周期由Anchor自动管理
    let authority = ctx.accounts.authority.key(); // 获取初始化人公钥
    let bump = *ctx.bumps.get("registry").unwrap(); // 获取PDA bump种子
    registry.base = crate::state::common::BaseAccount::new(authority, bump)?; // 初始化BaseAccount，权限与生命周期安全
    registry.algorithms = Vec::new(); // 初始化算法列表为空
    emit!(AlgorithmRegistryInitialized { // 触发算法注册表初始化事件，链上可追溯
        authority, // 事件：初始化人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)]
pub struct RegisterExecutionAlgorithm<'info> {
    // 可扩展权限校验等
}

pub fn register_execution_algorithm(_ctx: Context<RegisterExecutionAlgorithm>, name: String, algo: Arc<dyn crate::algorithms::traits::ExecutionStrategy>) -> Result<()> {
    crate::algorithms::algorithm_registry::ALGORITHM_REGISTRY.register_execution(&name, algo);
    Ok(())
}

#[derive(Accounts)]
pub struct RemoveExecutionAlgorithm<'info> {}

pub fn remove_execution_algorithm(_ctx: Context<RemoveExecutionAlgorithm>, name: String) -> Result<()> {
    crate::algorithms::algorithm_registry::ALGORITHM_REGISTRY.remove_execution(&name);
    Ok(())
}

#[derive(Accounts)]
pub struct ListExecutionAlgorithms<'info> {}

pub fn list_execution_algorithms(_ctx: Context<ListExecutionAlgorithms>) -> Result<Vec<String>> {
    Ok(crate::algorithms::algorithm_registry::ALGORITHM_REGISTRY.list_executions())
}

#[derive(Accounts)]
pub struct RegisterRoutingAlgorithm<'info> {
    // 可扩展权限校验等
}

pub fn register_routing_algorithm(_ctx: Context<RegisterRoutingAlgorithm>, name: String, algo: Arc<dyn crate::algorithms::traits::RoutingStrategy>) -> Result<()> {
    crate::algorithms::algorithm_registry::ALGORITHM_REGISTRY.register_routing(&name, algo);
    Ok(())
}

#[derive(Accounts)]
pub struct RemoveRoutingAlgorithm<'info> {}

pub fn remove_routing_algorithm(_ctx: Context<RemoveRoutingAlgorithm>, name: String) -> Result<()> {
    crate::algorithms::algorithm_registry::ALGORITHM_REGISTRY.remove_routing(&name);
    Ok(())
}

#[derive(Accounts)]
pub struct ListRoutingAlgorithms<'info> {}

pub fn list_routing_algorithms(_ctx: Context<ListRoutingAlgorithms>) -> Result<Vec<String>> {
    Ok(crate::algorithms::algorithm_registry::ALGORITHM_REGISTRY.list_routings())
}

#[derive(Accounts)]
pub struct RegisterRiskAlgorithm<'info> {
    // 可扩展权限校验等
}

pub fn register_risk_algorithm(_ctx: Context<RegisterRiskAlgorithm>, name: String, algo: Arc<dyn crate::algorithms::traits::RiskStrategy>) -> Result<()> {
    crate::algorithms::algorithm_registry::ALGORITHM_REGISTRY.register_risk(&name, algo);
    Ok(())
}

#[derive(Accounts)]
pub struct RemoveRiskAlgorithm<'info> {}

pub fn remove_risk_algorithm(_ctx: Context<RemoveRiskAlgorithm>, name: String) -> Result<()> {
    crate::algorithms::algorithm_registry::ALGORITHM_REGISTRY.remove_risk(&name);
    Ok(())
}

#[derive(Accounts)]
pub struct ListRiskAlgorithms<'info> {}

pub fn list_risk_algorithms(_ctx: Context<ListRiskAlgorithms>) -> Result<Vec<String>> {
    Ok(crate::algorithms::algorithm_registry::ALGORITHM_REGISTRY.list_risks())
}

#[derive(Accounts)]
pub struct RegisterOptimizerAlgorithm<'info> {
    // 可扩展权限校验等
}

pub fn register_optimizer_algorithm(_ctx: Context<RegisterOptimizerAlgorithm>, name: String, algo: Arc<dyn crate::algorithms::traits::OptimizerStrategy>) -> Result<()> {
    crate::algorithms::algorithm_registry::ALGORITHM_REGISTRY.register_optimizer(&name, algo);
    Ok(())
}

#[derive(Accounts)]
pub struct RemoveOptimizerAlgorithm<'info> {}

pub fn remove_optimizer_algorithm(_ctx: Context<RemoveOptimizerAlgorithm>, name: String) -> Result<()> {
    crate::algorithms::algorithm_registry::ALGORITHM_REGISTRY.remove_optimizer(&name);
    Ok(())
}

#[derive(Accounts)]
pub struct ListOptimizerAlgorithms<'info> {}

pub fn list_optimizer_algorithms(_ctx: Context<ListOptimizerAlgorithms>) -> Result<Vec<String>> {
    Ok(crate::algorithms::algorithm_registry::ALGORITHM_REGISTRY.list_optimizers())
} 