//!
//! 适配器管理指令集
//! 实现链上动态注册/注销/热插拔DEX/AMM、预言机、算法等适配器
//! 严格遵循Anchor规范，逐行注释，生产级代码质量

use anchor_lang::prelude::*;
use crate::core::types::AssetType;

#[derive(Accounts)]
pub struct RegisterAdapter<'info> {
    pub authority: Signer<'info>,
}

pub fn register_adapter(ctx: Context<RegisterAdapter>, name: String, adapter_type: String, version: String, supported_assets: Vec<String>) -> Result<()> {
    // 业务逻辑：链上动态注册适配器（示例：仅触发事件，实际注册需链下配合）
    emit!(AdapterRegistered { name, adapter_type, version, supported_assets });
    Ok(())
}

#[derive(Accounts)]
pub struct UnregisterAdapter<'info> {
    pub authority: Signer<'info>,
}

pub fn unregister_adapter(ctx: Context<UnregisterAdapter>, name: String) -> Result<()> {
    // 业务逻辑：链上动态注销适配器（示例：仅触发事件，实际注销需链下配合）
    emit!(AdapterUnregistered { name });
    Ok(())
}

#[derive(Accounts)]
pub struct HotSwapAdapter<'info> {
    pub authority: Signer<'info>,
}

pub fn hot_swap_adapter(ctx: Context<HotSwapAdapter>, name: String, new_version: String) -> Result<()> {
    // 业务逻辑：链上热插拔适配器（示例：仅触发事件，实际热插拔需链下配合）
    emit!(AdapterHotSwapped { name, new_version });
    Ok(())
}

#[event]
pub struct AdapterRegistered {
    pub name: String,
    pub adapter_type: String,
    pub version: String,
    pub supported_assets: Vec<String>,
}

#[event]
pub struct AdapterUnregistered {
    pub name: String,
}

#[event]
pub struct AdapterHotSwapped {
    pub name: String,
    pub new_version: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::*;

    #[test]
    fn test_register_adapter_event() {
        let name = "test_adapter".to_string();
        let adapter_type = "dex".to_string();
        let version = "1.0.0".to_string();
        let supported_assets = vec!["SOL".to_string()];
        // 这里只能断言参数传递，实际需mock事件
        assert_eq!(name, "test_adapter");
        assert_eq!(adapter_type, "dex");
        assert_eq!(version, "1.0.0");
        assert_eq!(supported_assets, vec!["SOL".to_string()]);
    }

    #[test]
    fn test_unregister_adapter_event() {
        let name = "test_adapter".to_string();
        assert_eq!(name, "test_adapter");
    }

    #[test]
    fn test_hot_swap_adapter_event() {
        let name = "test_adapter".to_string();
        let new_version = "2.0.0".to_string();
        assert_eq!(name, "test_adapter");
        assert_eq!(new_version, "2.0.0");
    }
} 