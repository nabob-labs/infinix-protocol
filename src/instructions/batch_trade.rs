//! 批量交易指令集实现文件
//! 该文件实现了BatchTradeInstructionTrait及其最小功能单元实现，所有方法均为生产级实现，逐行专业注释，无伪代码、无省略。

use anchor_lang::prelude::*;
use crate::core::types::{AssetOperationTrait, AssetBatchOperation, AssetBatchOpType, TokenInfo};

/// 批量交易指令集trait，定义所有批量交易相关操作接口
pub trait BatchTradeInstructionTrait {
    /// 执行一组批量资产操作
    fn execute_batch(ctx: &Context<BatchTrade>, ops: Vec<AssetBatchOperation>) -> Result<()>;
    /// 查询批量操作的预期结果
    fn preview_batch(ctx: &Context<BatchTrade>, ops: Vec<AssetBatchOperation>) -> Result<Vec<u64>>;
    /// 验证批量操作的有效性
    fn validate_batch(ctx: &Context<BatchTrade>, ops: &Vec<AssetBatchOperation>) -> Result<()>;
}

/// Anchor上下文结构体，定义批量交易涉及的账户
#[derive(Accounts)]
pub struct BatchTrade<'info> {
    /// 操作发起人
    #[account(mut)]
    pub authority: Signer<'info>,
    /// 资产账户列表（可扩展为多资产）
    #[account(mut)]
    pub asset_accounts: Vec<AccountInfo<'info>>,
}

/// 批量交易指令集实现体
pub struct BatchTradeInstruction;

impl BatchTradeInstructionTrait for BatchTradeInstruction {
    /// 执行一组批量资产操作
    fn execute_batch(ctx: &Context<BatchTrade>, ops: Vec<AssetBatchOperation>) -> Result<()> {
        // 遍历所有批量操作
        for op in ops.iter() {
            // 针对每个操作类型，调用对应的资产操作trait方法
            match op.op_type {
                AssetBatchOpType::Buy => {
                    // 调用资产买入操作
                    TokenInfo::buy(ctx, &op.asset, op.amount)?;
                },
                AssetBatchOpType::Sell => {
                    // 调用资产卖出操作
                    TokenInfo::sell(ctx, &op.asset, op.amount)?;
                },
                AssetBatchOpType::Swap => {
                    // 调用资产兑换操作
                    TokenInfo::swap(ctx, &op.asset, &op.target_asset, op.amount)?;
                },
                AssetBatchOpType::Combine => {
                    // 调用资产组合操作
                    TokenInfo::combine(ctx, &op.asset, &op.target_asset, op.amount)?;
                },
                AssetBatchOpType::Split => {
                    // 调用资产分割操作
                    TokenInfo::split(ctx, &op.asset, op.amount)?;
                },
                AssetBatchOpType::Authorize => {
                    // 调用资产授权操作
                    TokenInfo::authorize(ctx, &op.asset, &op.authority)?;
                },
                AssetBatchOpType::Freeze => {
                    // 调用资产冻结操作
                    TokenInfo::freeze(ctx, &op.asset)?;
                },
                AssetBatchOpType::Unfreeze => {
                    // 调用资产解冻操作
                    TokenInfo::unfreeze(ctx, &op.asset)?;
                },
            }
        }
        Ok(())
    }

    /// 查询批量操作的预期结果
    fn preview_batch(ctx: &Context<BatchTrade>, ops: Vec<AssetBatchOperation>) -> Result<Vec<u64>> {
        let mut results = Vec::with_capacity(ops.len());
        // 遍历所有批量操作，调用各自的预览逻辑
        for op in ops.iter() {
            let res = match op.op_type {
                AssetBatchOpType::Buy => TokenInfo::preview_buy(ctx, &op.asset, op.amount)?,
                AssetBatchOpType::Sell => TokenInfo::preview_sell(ctx, &op.asset, op.amount)?,
                AssetBatchOpType::Swap => TokenInfo::preview_swap(ctx, &op.asset, &op.target_asset, op.amount)?,
                AssetBatchOpType::Combine => TokenInfo::preview_combine(ctx, &op.asset, &op.target_asset, op.amount)?,
                AssetBatchOpType::Split => TokenInfo::preview_split(ctx, &op.asset, op.amount)?,
                AssetBatchOpType::Authorize => TokenInfo::preview_authorize(ctx, &op.asset, &op.authority)?,
                AssetBatchOpType::Freeze => TokenInfo::preview_freeze(ctx, &op.asset)?,
                AssetBatchOpType::Unfreeze => TokenInfo::preview_unfreeze(ctx, &op.asset)?,
            };
            results.push(res);
        }
        Ok(results)
    }

    /// 验证批量操作的有效性
    fn validate_batch(ctx: &Context<BatchTrade>, ops: &Vec<AssetBatchOperation>) -> Result<()> {
        // 遍历所有批量操作，调用各自的验证逻辑
        for op in ops.iter() {
            match op.op_type {
                AssetBatchOpType::Buy => TokenInfo::validate_buy(ctx, &op.asset, op.amount)?,
                AssetBatchOpType::Sell => TokenInfo::validate_sell(ctx, &op.asset, op.amount)?,
                AssetBatchOpType::Swap => TokenInfo::validate_swap(ctx, &op.asset, &op.target_asset, op.amount)?,
                AssetBatchOpType::Combine => TokenInfo::validate_combine(ctx, &op.asset, &op.target_asset, op.amount)?,
                AssetBatchOpType::Split => TokenInfo::validate_split(ctx, &op.asset, op.amount)?,
                AssetBatchOpType::Authorize => TokenInfo::validate_authorize(ctx, &op.asset, &op.authority)?,
                AssetBatchOpType::Freeze => TokenInfo::validate_freeze(ctx, &op.asset)?,
                AssetBatchOpType::Unfreeze => TokenInfo::validate_unfreeze(ctx, &op.asset)?,
            }
        }
        Ok(())
    }
} 