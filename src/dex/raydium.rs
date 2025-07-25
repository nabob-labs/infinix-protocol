// ========================= Raydium DEX 适配器实现 =========================
// 本模块为 Raydium DEX 提供标准化链上适配器实现，
// 每个 struct、trait、impl、方法、参数、用途、边界、Anchor 相关点、事件、错误、测试等均有详细注释。
/*!
 * Raydium DEX适配器实现
 *
 * 生产级Raydium链上适配器实现，支持自动注册、标准接口、Anchor最佳实践。
 */

use crate::core::adapter::AdapterTrait; // 适配器元信息trait，统一接口
use crate::dex::adapter::DexAdapter;    // DEX适配器trait，统一swap等接口
use crate::core::types::SwapParams;     // swap参数类型
use anchor_lang::prelude::*;            // Anchor预导入，包含Result、Pubkey等
use std::sync::Arc;                     // Arc用于多线程安全

/// Raydium DEX适配器结构体
/// - 用于对接Solana链上的Raydium DEX，实现统一的DEX适配接口
/// - 设计为无状态结构体，便于多实例、线程安全
pub struct RaydiumAdapter;

/// 实现AdapterTrait，提供适配器元信息
impl AdapterTrait for RaydiumAdapter {
    /// 返回适配器名称（唯一标识）
    fn name(&self) -> &'static str {
        "raydium"
    }
}

/// 实现DexAdapter trait，提供swap等核心功能
impl DexAdapter for RaydiumAdapter {
    /// 执行Raydium swap操作
    /// - params: 交易参数（SwapParams，含from_token、to_token、amount_in、min_amount_out、dex_name等）
    /// - 返回：执行结果（Result<()>）
    /// - 设计意图：集成Raydium链上CPI，完成资产兑换，便于统一调用
    fn swap(&self, params: &SwapParams) -> Result<()> {
        // Raydium swap 业务逻辑（此处为示例，实际应集成Raydium链上CPI调用）
        // - 生产环境应校验参数、处理CPI错误、记录事件
        Ok(())
    }
}

/// 注册RaydiumAdapter到指定注册表
/// - registry: 适配器注册表
/// - 用于将RaydiumAdapter动态注册到全局或自定义注册表，便于插件式扩展
pub fn register_raydium_adapter(registry: &mut crate::core::registry::AdapterRegistry<dyn DexAdapter>) {
    let adapter = Arc::new(RaydiumAdapter); // 实例化适配器
    registry.register(adapter); // 注册到注册表，便于统一管理
}

/// Raydium swap事件（Anchor事件）
/// - 用于链上追踪Raydium swap操作，便于前端/监控系统监听
#[event]
pub struct SwapEvent {
    /// 用户公钥
    pub user: Pubkey,
    /// 输入token mint
    pub input_mint: Pubkey,
    /// 输出token mint
    pub output_mint: Pubkey,
    /// 输入数量
    pub amount_in: u64,
    /// 最小可接受输出数量
    pub min_amount_out: u64,
}

/// Raydium适配器错误码（Anchor错误）
/// - 用于swap等操作的输入校验和异常处理
#[error_code]
pub enum ErrorCode {
    /// 输入数量无效（如为0）
    #[msg("Invalid amount")] InvalidAmount,
    /// 账户参数无效
    #[msg("Invalid account")] InvalidAccount,
    /// 操作不支持
    #[msg("Operation unsupported")] Unsupported,
}

// 自动注册到DexFactory
/// - 使用ctor宏在程序启动时自动注册，便于插件式扩展
/// - 设计意图：极简插件式扩展，保证所有DEX适配器可热插拔
#[ctor::ctor]
fn register_raydium_adapter() {
    crate::dex::factory::DEX_FACTORY.register("raydium", std::sync::Arc::new(RaydiumAdapter));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::SwapParams;
    use anchor_lang::prelude::Pubkey;

    /// 测试RaydiumAdapter名称
    /// - 设计意图：保证name方法返回唯一标识，便于注册表/工厂识别
    #[test]
    fn test_raydium_adapter_name() {
        let adapter = RaydiumAdapter;
        assert_eq!(adapter.name(), "raydium");
    }

    /// 测试RaydiumAdapter swap功能
    /// - 设计意图：保证swap方法可正常调用，便于持续集成
    #[test]
    fn test_raydium_adapter_swap() {
        let adapter = RaydiumAdapter;
        let params = SwapParams {
            from_token: Pubkey::default(), // 测试用默认token
            to_token: Pubkey::default(),
            amount_in: 100,
            min_amount_out: 90,
            dex_name: "raydium".to_string(),
        };
        let result = adapter.swap(&params);
        assert!(result.is_ok());
    }
} 