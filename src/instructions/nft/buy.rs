//! NFT购买指令模块
//! 
//! 本模块提供NFT资产的购买功能，包括：
//! - 参数验证：验证购买参数的有效性和边界条件
// - 权限检查：验证购买权限和授权状态
//! - 服务层调用：委托给NftService执行核心业务逻辑
//! - 事件发射：发射NFT购买事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于NFT购买功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给NftService
//! - 事件驱动：完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::{
    core::{
        constants::*,
        events::*,
        types::*,
        validation::*,
    },
    errors::*,
    services::*,
    utils::*,
};

/// NFT购买参数结构体
/// 
/// 包含NFT购买所需的所有参数：
/// - price: 购买价格
/// - currency: 计价货币
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BuyNftParams {
    /// 购买价格
    pub price: u64,
    /// 计价货币
    pub currency: Pubkey,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// NFT购买账户上下文
/// 
/// 定义NFT购买指令所需的账户结构：
/// - nft: NFT账户（可变，NFT类型约束）
/// - buyer: 购买者账户（签名者）
/// - seller: 卖家账户
/// - buyer_token_account: 购买者代币账户
/// - seller_token_account: 卖家代币账户
/// - nft_token_account: NFT代币账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct BuyNft<'info> {
    /// NFT账户（可变，NFT类型约束）
    #[account(
        mut,
        constraint = nft.asset_type == AssetType::NFT @ AssetError::InvalidAssetType
    )]
    pub nft: Account<'info, Asset>,
    
    /// 购买者账户（签名者）
    pub buyer: Signer<'info>,
    
    /// 卖家账户
    /// CHECK: 由程序验证
    pub seller: UncheckedAccount<'info>,
    
    /// 购买者代币账户
    #[account(
        mut,
        constraint = buyer_token_account.owner == buyer.key() @ AssetError::InvalidTokenAccount
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,
    
    /// 卖家代币账户
    #[account(
        mut,
        constraint = seller_token_account.owner == seller.key() @ AssetError::InvalidTokenAccount
    )]
    pub seller_token_account: Account<'info, TokenAccount>,
    
    /// NFT代币账户
    #[account(
        mut,
        constraint = nft_token_account.owner == seller.key() @ AssetError::InvalidTokenAccount
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 验证NFT购买参数
/// 
/// 检查NFT购买参数的有效性和边界条件：
/// - 价格验证
/// - 计价货币验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: NFT购买参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_buy_nft_params(params: &BuyNftParams) -> Result<()> {
    // 验证价格
    require!(
        params.price > 0,
        AssetError::InvalidPrice
    );
    
    // 验证计价货币
    require!(
        params.currency != Pubkey::default(),
        AssetError::InvalidCurrency
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查NFT购买权限
/// 
/// 验证NFT购买权限和授权状态：
/// - 检查NFT状态
/// - 验证上架状态
/// - 检查购买者余额
/// 
/// # 参数
/// - buyer: 购买者账户
/// - nft: NFT账户
/// - buyer_token_account: 购买者代币账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_buy_authority_permission(
    buyer: &Signer,
    nft: &Account<Asset>,
    buyer_token_account: &Account<TokenAccount>,
) -> Result<()> {
    // 验证NFT状态
    require!(
        nft.is_active(),
        AssetError::AssetNotActive
    );
    
    // 验证上架状态
    require!(
        nft.is_listed(),
        AssetError::AssetNotListed
    );
    
    // 检查购买者余额
    require!(
        buyer_token_account.amount >= nft.listing_price,
        AssetError::InsufficientBalance
    );
    
    Ok(())
}

/// NFT购买指令
/// 
/// 执行NFT购买操作，包括：
/// - 参数验证：验证购买参数的有效性
/// - 权限检查：验证购买权限
/// - 服务层调用：委托给NftService执行购买逻辑
/// - 事件发射：发射NFT购买事件
/// 
/// # 参数
/// - ctx: NFT购买账户上下文
/// - params: NFT购买参数
/// 
/// # 返回
/// - Result<()>: 购买操作结果
pub fn buy_nft(
    ctx: Context<BuyNft>,
    params: BuyNftParams,
) -> Result<()> {
    // 参数验证
    validate_buy_nft_params(&params)?;
    
    // 权限检查
    check_buy_authority_permission(
        &ctx.accounts.buyer,
        &ctx.accounts.nft,
        &ctx.accounts.buyer_token_account,
    )?;
    
    let nft = &mut ctx.accounts.nft;
    let buyer = &ctx.accounts.buyer;
    
    // 创建NFT服务实例
    let service = NftService::new();
    
    // 执行NFT购买
    service.buy(
        nft,
        params.price,
        params.currency,
        &params.exec_params,
    )?;
    
    // 发射NFT购买事件
    emit!(AssetBought {
        basket_id: nft.id,
        amount: 1, // NFT购买数量为1
        authority: buyer.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::NFT,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 