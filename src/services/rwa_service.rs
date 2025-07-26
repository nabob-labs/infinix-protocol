//! RwaService - RWA资产服务层
//! 生产级最小功能单元，SOLID原则、合规注释

use crate::state::baskets::BasketIndexState;
use anchor_lang::prelude::*;

/// RwaService结构体，RWA资产服务实现
pub struct RwaService;

impl RwaService {
    /// 构造函数，返回RwaService实例
    pub fn new() -> Self {
        Self
    }

    /// RWA资产mint最小功能单元
    /// - basket: RWA资产账户，需可变
    /// - amount: 增发数量，类型安全
    pub fn mint(&self, basket: &mut BasketIndexState, amount: u64) -> Result<()> {
        // 校验资产类型
        require!(basket.asset_type == crate::core::types::AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
        // 校验账户激活状态
        require!(basket.is_active, crate::errors::asset_error::AssetError::NotAllowed);
        // 增发操作，防止溢出
        basket.total_value = basket.total_value.checked_add(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        Ok(())
    }

    /// RWA资产burn最小功能单元
    /// - basket: RWA资产账户，需可变
    /// - amount: 销毁数量，类型安全
    pub fn burn(&self, basket: &mut BasketIndexState, amount: u64) -> Result<()> {
        // 校验资产类型
        require!(basket.asset_type == crate::core::types::AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
        // 校验账户激活状态
        require!(basket.is_active, crate::errors::asset_error::AssetError::NotAllowed);
        // 校验余额充足
        require!(basket.total_value >= amount, crate::errors::asset_error::AssetError::InsufficientValue);
        // 销毁操作，防止溢出
        basket.total_value = basket.total_value.checked_sub(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        Ok(())
    }

    /// RWA资产buy最小功能单元
    /// - basket: RWA资产账户，需可变
    /// - amount: 买入数量，类型安全
    pub fn buy(&self, basket: &mut BasketIndexState, amount: u64) -> Result<()> {
        // 校验资产类型
        require!(basket.asset_type == crate::core::types::AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
        // 校验账户激活状态
        require!(basket.is_active, crate::errors::asset_error::AssetError::NotAllowed);
        // 买入操作，防止溢出
        basket.total_value = basket.total_value.checked_add(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        Ok(())
    }

    /// RWA资产sell最小功能单元
    /// - basket: RWA资产账户，需可变
    /// - amount: 卖出数量，类型安全
    pub fn sell(&self, basket: &mut BasketIndexState, amount: u64) -> Result<()> {
        // 校验资产类型
        require!(basket.asset_type == crate::core::types::AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
        // 校验账户激活状态
        require!(basket.is_active, crate::errors::asset_error::AssetError::NotAllowed);
        // 校验余额充足
        require!(basket.total_value >= amount, crate::errors::asset_error::AssetError::InsufficientValue);
        // 卖出操作，防止溢出
        basket.total_value = basket.total_value.checked_sub(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        Ok(())
    }

    /// RWA资产transfer最小功能单元
    /// - from: 转出RWA资产账户，需可变
    /// - to: 转入RWA资产账户，需可变
    /// - amount: 转账数量，类型安全
    pub fn transfer(&self, from: &mut BasketIndexState, to: &mut BasketIndexState, amount: u64) -> Result<()> {
        // 校验资产类型
        require!(from.asset_type == crate::core::types::AssetType::RWA && to.asset_type == crate::core::types::AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
        // 校验账户激活状态
        require!(from.is_active && to.is_active, crate::errors::asset_error::AssetError::NotAllowed);
        // 校验余额充足
        require!(from.total_value >= amount, crate::errors::asset_error::AssetError::InsufficientValue);
        // 转账操作，防止溢出
        from.total_value = from.total_value.checked_sub(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        to.total_value = to.total_value.checked_add(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        Ok(())
    }

    /// RWA资产swap最小功能单元
    /// - from: 转出RWA资产账户，需可变
    /// - to: 转入RWA资产账户，需可变
    /// - from_amount: 转出数量，类型安全
    pub fn swap(&self, from: &mut BasketIndexState, to: &mut BasketIndexState, from_amount: u64) -> Result<()> {
        // 校验资产类型
        require!(from.asset_type == crate::core::types::AssetType::RWA && to.asset_type == crate::core::types::AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
        // 校验账户激活状态
        require!(from.is_active && to.is_active, crate::errors::asset_error::AssetError::NotAllowed);
        // 校验余额充足
        require!(from.total_value >= from_amount, crate::errors::asset_error::AssetError::InsufficientValue);
        // swap操作，防止溢出
        from.total_value = from.total_value.checked_sub(from_amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        to.total_value = to.total_value.checked_add(from_amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        Ok(())
    }

    /// RWA资产combine最小功能单元
    /// - target: 目标RWA资产账户，需可变
    /// - source: 源RWA资产账户，需可变
    /// - amount: 合并数量，类型安全
    pub fn combine(&self, target: &mut BasketIndexState, source: &mut BasketIndexState, amount: u64) -> Result<()> {
        // 校验资产类型
        require!(target.asset_type == crate::core::types::AssetType::RWA && source.asset_type == crate::core::types::AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
        // 校验账户激活状态
        require!(target.is_active && source.is_active, crate::errors::asset_error::AssetError::NotAllowed);
        // 校验源账户余额充足
        require!(source.total_value >= amount, crate::errors::asset_error::AssetError::InsufficientValue);
        // 合并操作，防止溢出
        source.total_value = source.total_value.checked_sub(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        target.total_value = target.total_value.checked_add(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        Ok(())
    }

    /// RWA资产split最小功能单元
    /// - source: 源RWA资产账户，需可变
    /// - new: 新RWA资产账户，需可变
    /// - amount: 拆分数量，类型安全
    pub fn split(&self, source: &mut BasketIndexState, new: &mut BasketIndexState, amount: u64) -> Result<()> {
        // 校验资产类型
        require!(source.asset_type == crate::core::types::AssetType::RWA && new.asset_type == crate::core::types::AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
        // 校验账户激活状态
        require!(source.is_active && new.is_active, crate::errors::asset_error::AssetError::NotAllowed);
        // 校验源账户余额充足
        require!(source.total_value >= amount, crate::errors::asset_error::AssetError::InsufficientValue);
        // 拆分操作，防止溢出
        source.total_value = source.total_value.checked_sub(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        new.total_value = new.total_value.checked_add(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        Ok(())
    }

    /// RWA资产freeze最小功能单元
    /// - basket: RWA资产账户，需可变
    pub fn freeze(&self, basket: &mut BasketIndexState) -> Result<()> {
        // 校验资产类型
        require!(basket.asset_type == crate::core::types::AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
        // 校验账户激活状态
        require!(basket.is_active, crate::errors::asset_error::AssetError::NotAllowed);
        // 冻结操作
        basket.is_active = false;
        Ok(())
    }

    /// RWA资产unfreeze最小功能单元
    /// - basket: RWA资产账户，需可变
    pub fn unfreeze(&self, basket: &mut BasketIndexState) -> Result<()> {
        // 校验资产类型
        require!(basket.asset_type == crate::core::types::AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
        // 校验账户冻结状态
        require!(!basket.is_active, crate::errors::asset_error::AssetError::NotAllowed);
        // 解冻操作
        basket.is_active = true;
        Ok(())
    }

    /// RWA资产authorize最小功能单元
    /// - basket: RWA资产账户，需可变
    /// - new_authority: 新授权人公钥
    pub fn authorize(&self, basket: &mut BasketIndexState, new_authority: Pubkey) -> Result<()> {
        // 校验资产类型
        require!(basket.asset_type == crate::core::types::AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
        // 授权操作
        basket.authority = new_authority;
        Ok(())
    }

    /// RWA资产query最小功能单元
    /// - basket: RWA资产账户，只读
    pub fn query(&self, basket: &BasketIndexState) -> Result<u64> {
        // 校验资产类型
        require!(basket.asset_type == crate::core::types::AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
        Ok(basket.total_value)
    }

    /// RWA资产quote最小功能单元
    /// - basket: RWA资产账户，只读
    /// - price_params: 价格参数
    pub fn quote(&self, basket: &BasketIndexState, price_params: crate::core::types::PriceParams) -> Result<u64> {
        // 校验资产类型
        require!(basket.asset_type == crate::core::types::AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
        // 这里可集成预言机/外部价格逻辑，当前直接返回price_params.price
        Ok(price_params.price)
    }

    /// RWA资产batch_transfer最小功能单元
    /// - from: 批量转出RWA资产账户，需可变
    /// - to_rwas: 批量转入RWA资产账户，需可变
    /// - params: 批量参数
    /// - authority: 操作人公钥
    pub fn batch_transfer(&self, from: &mut BasketIndexState, to_rwas: &mut [BasketIndexState], params: &crate::core::types::BatchSwapParams, authority: Pubkey) -> Result<()> {
        // 校验资产类型
        require!(from.asset_type == crate::core::types::AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
        for to in to_rwas.iter() {
            require!(to.asset_type == crate::core::types::AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
        }
        // 校验批量参数长度
        require!(params.amounts.len() == to_rwas.len(), crate::error::ProgramError::InvalidArgument);
        // 校验余额充足
        let total: u64 = params.amounts.iter().sum();
        require!(from.total_value >= total, crate::errors::asset_error::AssetError::InsufficientValue);
        // 批量转账操作
        for (to, &amount) in to_rwas.iter_mut().zip(params.amounts.iter()) {
            from.total_value = from.total_value.checked_sub(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
            to.total_value = to.total_value.checked_add(amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        }
        Ok(())
    }
} 