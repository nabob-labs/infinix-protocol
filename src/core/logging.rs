//! 日志记录模块

use anchor_lang::prelude::*;

/// 记录算法执行事件日志
///
/// # 用途
/// - 用于链上/链下观测算法执行结果
///
/// # 参数
/// - `algorithm`: 算法名称
/// - `result`: 执行结果描述
///
/// # Anchor 最佳实践
/// - 使用 msg! 宏输出日志，便于链上调试与观测
pub fn log_algorithm_event(algorithm: &str, result: &str) {
    // Anchor/Solana 推荐使用 msg! 宏输出日志，便于链上调试与观测
    msg!("[Algorithm] {} result: {}", algorithm, result);
}

/// 记录 DEX 交易日志
///
/// # 用途
/// - 用于链上/链下观测 DEX 交易明细
///
/// # 参数
/// - `dex`: DEX 名称
/// - `amount_in`: 输入数量
/// - `amount_out`: 输出数量
/// - `token_in`: 输入代币 mint
/// - `token_out`: 输出代币 mint
/// - `user`: 用户公钥
///
/// # Anchor 最佳实践
/// - 使用 msg! 宏输出日志，便于链上调试与观测
pub fn log_dex_swap(dex: &str, amount_in: u64, amount_out: u64, token_in: &Pubkey, token_out: &Pubkey, user: &Pubkey) {
    // Anchor/Solana 推荐使用 msg! 宏输出日志，便于链上调试与观测
    msg!("[DEX] {} swap: {} -> {} ({} -> {}), user: {}", dex, amount_in, amount_out, token_in, token_out, user);
}

/// 记录预言机查询日志
///
/// # 用途
/// - 用于链上/链下观测预言机价格查询
///
/// # 参数
/// - `oracle`: 预言机名称
/// - `base_mint`: 基础资产 mint
/// - `quote_mint`: 报价资产 mint
/// - `price`: 查询价格
///
/// # Anchor 最佳实践
/// - 使用 msg! 宏输出日志，便于链上调试与观测
pub fn log_oracle_query(oracle: &str, base_mint: &Pubkey, quote_mint: &Pubkey, price: u64) {
    // Anchor/Solana 推荐使用 msg! 宏输出日志，便于链上调试与观测
    msg!("[Oracle] {} price: {}/{} = {}", oracle, base_mint, quote_mint, price);
}

/// 记录指令分发日志
///
/// # 用途
/// - 用于链上/链下观测指令调用
///
/// # 参数
/// - `instruction`: 指令名称
/// - `accounts`: 相关账户列表
/// - `params`: 指令参数序列化数据
/// - `user`: 调用用户公钥
///
/// # Anchor 最佳实践
/// - 使用 msg! 宏输出日志，便于链上调试与观测
pub fn log_instruction_dispatch(instruction: &str, accounts: &[Pubkey], params: &[u8], user: &Pubkey) {
    // Anchor/Solana 推荐使用 msg! 宏输出日志，便于链上调试与观测
    msg!("[Instruction] {} dispatched by {}, accounts: {:?}, params: {:?}", instruction, user, accounts, params);
} 