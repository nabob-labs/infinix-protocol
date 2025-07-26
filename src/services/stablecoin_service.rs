//! StablecoinService - 稳定币资产服务层
//! 生产级最小功能单元，SOLID原则、合规注释

use crate::state::baskets::BasketIndexState;
use anchor_lang::prelude::*;

/// StablecoinService结构体，稳定币资产服务实现
pub struct StablecoinService;

impl StablecoinService {
    /// 构造函数，返回StablecoinService实例
    pub fn new() -> Self {
        Self
    }

    /// 稳定币资产mint最小功能单元
    /// - basket: 稳定币资产账户，需可变
    /// - amount: 增发数量，类型安全
    pub fn mint(&self, basket: &mut BasketIndexState, amount: u64) -> Result<()> {
        // 校验资产类型
        require!(basket.asset_type == crate::core::types::AssetType::Stablecoin, crate::error::ProgramError::InvalidAssetType);
        // 校验账户激活状态
        require!(basket.is_active, crate::errors::asset_error::AssetError::NotAllowed);
        // 增发操作，防止溢出
        basket.total_value = basket.total_value.checked_add(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        Ok(())
    }

    /// 稳定币资产buy最小功能单元
    /// - basket: 稳定币资产账户，需可变
    /// - amount: 买入数量，类型安全
    pub fn buy(&self, basket: &mut BasketIndexState, amount: u64) -> Result<()> {
        // 校验资产类型
        require!(basket.asset_type == crate::core::types::AssetType::Stablecoin, crate::error::ProgramError::InvalidAssetType);
        // 校验账户激活状态
        require!(basket.is_active, crate::errors::asset_error::AssetError::NotAllowed);
        // 买入操作，防止溢出
        basket.total_value = basket.total_value.checked_add(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        Ok(())
    }

    /// 稳定币资产sell最小功能单元
    /// - basket: 稳定币资产账户，需可变
    /// - amount: 卖出数量，类型安全
    pub fn sell(&self, basket: &mut BasketIndexState, amount: u64) -> Result<()> {
        // 校验资产类型
        require!(basket.asset_type == crate::core::types::AssetType::Stablecoin, crate::error::ProgramError::InvalidAssetType);
        // 校验账户激活状态
        require!(basket.is_active, crate::errors::asset_error::AssetError::NotAllowed);
        // 校验余额充足
        require!(basket.total_value >= amount, crate::errors::asset_error::AssetError::InsufficientValue);
        // 卖出操作，防止溢出
        basket.total_value = basket.total_value.checked_sub(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        Ok(())
    }

    /// 稳定币资产transfer最小功能单元
    /// - from: 转出稳定币资产账户，需可变
    /// - to: 转入稳定币资产账户，需可变
    /// - amount: 转账数量，类型安全
    pub fn transfer(&self, from: &mut BasketIndexState, to: &mut BasketIndexState, amount: u64) -> Result<()> {
        // 校验资产类型
        require!(from.asset_type == crate::core::types::AssetType::Stablecoin && to.asset_type == crate::core::types::AssetType::Stablecoin, crate::error::ProgramError::InvalidAssetType);
        // 校验账户激活状态
        require!(from.is_active && to.is_active, crate::errors::asset_error::AssetError::NotAllowed);
        // 校验余额充足
        require!(from.total_value >= amount, crate::errors::asset_error::AssetError::InsufficientValue);
        // 转账操作，防止溢出
        from.total_value = from.total_value.checked_sub(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        to.total_value = to.total_value.checked_add(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        Ok(())
    }

    /// 稳定币资产swap最小功能单元
    /// - from: 转出稳定币资产账户，需可变
    /// - to: 转入稳定币资产账户，需可变
    /// - from_amount: 转出数量，类型安全
    pub fn swap(&self, from: &mut BasketIndexState, to: &mut BasketIndexState, from_amount: u64) -> Result<()> {
        // 校验资产类型
        require!(from.asset_type == crate::core::types::AssetType::Stablecoin && to.asset_type == crate::core::types::AssetType::Stablecoin, crate::error::ProgramError::InvalidAssetType);
        // 校验账户激活状态
        require!(from.is_active && to.is_active, crate::errors::asset_error::AssetError::NotAllowed);
        // 校验余额充足
        require!(from.total_value >= from_amount, crate::errors::asset_error::AssetError::InsufficientValue);
        // swap操作，防止溢出
        from.total_value = from.total_value.checked_sub(from_amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        to.total_value = to.total_value.checked_add(from_amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        Ok(())
    }

    /// 稳定币资产combine最小功能单元
    /// - target: 目标稳定币资产账户，需可变
    /// - source: 源稳定币资产账户，需可变
    /// - amount: 合并数量，类型安全
    pub fn combine(&self, target: &mut BasketIndexState, source: &mut BasketIndexState, amount: u64) -> Result<()> {
        // 校验资产类型
        require!(target.asset_type == crate::core::types::AssetType::Stablecoin && source.asset_type == crate::core::types::AssetType::Stablecoin, crate::error::ProgramError::InvalidAssetType);
        // 校验账户激活状态
        require!(target.is_active && source.is_active, crate::errors::asset_error::AssetError::NotAllowed);
        // 校验源账户余额充足
        require!(source.total_value >= amount, crate::errors::asset_error::AssetError::InsufficientValue);
        // 合并操作，防止溢出
        source.total_value = source.total_value.checked_sub(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        target.total_value = target.total_value.checked_add(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        Ok(())
    }

    /// 稳定币资产split最小功能单元
    /// - source: 源稳定币资产账户，需可变
    /// - new: 新稳定币资产账户，需可变
    /// - amount: 拆分数量，类型安全
    pub fn split(&self, source: &mut BasketIndexState, new: &mut BasketIndexState, amount: u64) -> Result<()> {
        // 校验资产类型
        require!(source.asset_type == crate::core::types::AssetType::Stablecoin && new.asset_type == crate::core::types::AssetType::Stablecoin, crate::error::ProgramError::InvalidAssetType);
        // 校验账户激活状态
        require!(source.is_active && new.is_active, crate::errors::asset_error::AssetError::NotAllowed);
        // 校验源账户余额充足
        require!(source.total_value >= amount, crate::errors::asset_error::AssetError::InsufficientValue);
        // 拆分操作，防止溢出
        source.total_value = source.total_value.checked_sub(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        new.total_value = new.total_value.checked_add(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        Ok(())
    }

    /// 稳定币资产freeze最小功能单元
    /// - basket: 稳定币资产账户，需可变
    pub fn freeze(&self, basket: &mut BasketIndexState) -> Result<()> {
        // 校验资产类型
        require!(basket.asset_type == crate::core::types::AssetType::Stablecoin, crate::error::ProgramError::InvalidAssetType);
        // 校验账户激活状态
        require!(basket.is_active, crate::errors::asset_error::AssetError::NotAllowed);
        // 冻结操作
        basket.is_active = false;
        Ok(())
    }

    /// 稳定币资产unfreeze最小功能单元
    /// - basket: 稳定币资产账户，需可变
    pub fn unfreeze(&self, basket: &mut BasketIndexState) -> Result<()> {
        // 校验资产类型
        require!(basket.asset_type == crate::core::types::AssetType::Stablecoin, crate::error::ProgramError::InvalidAssetType);
        // 校验账户冻结状态
        require!(!basket.is_active, crate::errors::asset_error::AssetError::NotAllowed);
        // 解冻操作
        basket.is_active = true;
        Ok(())
    }

    /// 稳定币资产authorize最小功能单元
    /// - basket: 稳定币资产账户，需可变
    /// - new_authority: 新授权人公钥
    pub fn authorize(&self, basket: &mut BasketIndexState, new_authority: Pubkey) -> Result<()> {
        // 校验资产类型
        require!(basket.asset_type == crate::core::types::AssetType::Stablecoin, crate::error::ProgramError::InvalidAssetType);
        // 授权操作，更新authority字段
        basket.authority = new_authority;
        Ok(())
    }

    /// 稳定币资产query最小功能单元
    /// - basket: 稳定币资产账户
    /// 返回: 资产信息结构体
    pub fn query(&self, basket: &BasketIndexState) -> Result<StablecoinInfo> {
        // 校验资产类型
        require!(basket.asset_type == crate::core::types::AssetType::Stablecoin, crate::error::ProgramError::InvalidAssetType);
        Ok(StablecoinInfo {
            total_value: basket.total_value,
            is_active: basket.is_active,
            authority: basket.authority,
            asset_type: basket.asset_type as u8,
        })
    }

    /// 稳定币资产quote最小功能单元
    /// - basket: 稳定币资产账户
    /// - amount: 询价数量
    /// 返回: 报价信息结构体
    pub fn quote(&self, basket: &BasketIndexState, amount: u64) -> Result<StablecoinQuote> {
        // 校验资产类型
        require!(basket.asset_type == crate::core::types::AssetType::Stablecoin, crate::error::ProgramError::InvalidAssetType);
        // 校验数量
        require!(amount > 0, crate::errors::asset_error::AssetError::InvalidAmount);
        // 这里可集成oracle价格，示例直接返回1:1
        Ok(StablecoinQuote {
            quote_value: amount, // 假设1:1报价
            price_oracle: Pubkey::default(), // 实际应集成oracle
        })
    }

    /// 稳定币资产batch最小功能单元
    /// - basket: 稳定币资产账户，需可变
    /// - actions: 批量操作类型与参数数组
    pub fn batch(&self, basket: &mut BasketIndexState, actions: &[StablecoinBatchAction]) -> Result<()> {
        // 校验资产类型
        require!(basket.asset_type == crate::core::types::AssetType::Stablecoin, crate::error::ProgramError::InvalidAssetType);
        // 遍历批量操作
        for action in actions {
            match action.action_type {
                0 => { self.mint(basket, action.amount)?; },
                1 => { self.burn(basket, action.amount)?; },
                2 => {
                    if let Some(target) = action.target {
                        self.transfer(basket, action.amount, target)?;
                    }
                },
                3 => { self.freeze(basket)?; },
                4 => { self.unfreeze(basket)?; },
                _ => return Err(crate::errors::asset_error::AssetError::InvalidAction.into()),
            }
        }
        Ok(())
    }
}

/// 稳定币资产信息结构体
#[derive(Clone, Debug)]
pub struct StablecoinInfo {
    pub total_value: u64,      // 总价值
    pub is_active: bool,       // 是否激活
    pub authority: Pubkey,     // 授权人
    pub asset_type: u8,        // 资产类型
}

/// 稳定币资产报价信息结构体
#[derive(Clone, Debug)]
pub struct StablecoinQuote {
    pub quote_value: u64,      // 报价数值
    pub price_oracle: Pubkey,  // 价格预言机地址
}

/// 批量操作类型定义
#[derive(Clone, Debug)]
pub struct StablecoinBatchAction {
    pub action_type: u8, // 0:mint, 1:burn, 2:transfer, 3:freeze, 4:unfreeze, ...
    pub amount: u64,
    pub target: Option<Pubkey>, // 目标账户（如转账、授权等）
} 