//! NFT铸造指令模块
//! 
//! 本模块提供NFT资产的铸造功能，包括：
//! - 参数验证：验证铸造参数的有效性和边界条件
//! - 权限检查：验证铸造权限和授权状态
//! - 服务层调用：委托给NftService执行核心业务逻辑
//! - 事件发射：发射NFT铸造事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于NFT铸造功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给NftService
//! - 事件驱动：完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

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

/// NFT铸造参数结构体
/// 
/// 包含NFT铸造所需的所有参数：
/// - metadata_uri: NFT元数据URI
/// - name: NFT名称
/// - symbol: NFT符号
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct MintNftParams {
    /// NFT元数据URI
    pub metadata_uri: String,
    /// NFT名称
    pub name: String,
    /// NFT符号
    pub symbol: String,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// NFT铸造账户上下文
/// 
/// 定义NFT铸造指令所需的账户结构：
/// - nft: NFT账户（可变，NFT类型约束）
/// - authority: 铸造权限账户（mint_authority约束）
/// - system_program: 系统程序
/// - token_program: 代币程序
/// - associated_token_account: 关联代币账户
/// - recipient_token_account: 接收者代币账户
#[derive(Accounts)]
pub struct MintNft<'info> {
    /// NFT账户（可变，NFT类型约束）
    #[account(
        mut,
        constraint = nft.asset_type == AssetType::NFT @ AssetError::InvalidAssetType
    )]
    pub nft: Account<'info, Asset>,
    
    /// 铸造权限账户（mint_authority约束）
    #[account(
        constraint = authority.key() == nft.mint_authority @ AssetError::InvalidMintAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
    
    /// 关联代币账户
    /// CHECK: 由代币程序验证
    pub associated_token_account: UncheckedAccount<'info>,
    
    /// 接收者代币账户
    #[account(mut)]
    pub recipient_token_account: Account<'info, TokenAccount>,
}

/// 验证NFT铸造参数
/// 
/// 检查NFT铸造参数的有效性和边界条件：
/// - metadata_uri长度验证
/// - name长度验证
/// - symbol长度验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: NFT铸造参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_mint_nft_params(params: &MintNftParams) -> Result<()> {
    // 验证metadata_uri长度
    require!(
        params.metadata_uri.len() <= MAX_METADATA_URI_LENGTH,
        AssetError::MetadataUriTooLong
    );
    
    // 验证name长度
    require!(
        params.name.len() <= MAX_NAME_LENGTH,
        AssetError::NameTooLong
    );
    
    // 验证symbol长度
    require!(
        params.symbol.len() <= MAX_SYMBOL_LENGTH,
        AssetError::SymbolTooLong
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查NFT铸造权限
/// 
/// 验证NFT铸造权限和授权状态：
/// - 检查铸造权限
/// - 验证授权状态
/// 
/// # 参数
/// - authority: 权限账户
/// - nft: NFT账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_mint_authority_permission(
    authority: &Signer,
    nft: &Account<Asset>,
) -> Result<()> {
    // 检查铸造权限
    require!(
        authority.key() == nft.mint_authority,
        AssetError::InvalidMintAuthority
    );
    
    // 验证NFT状态
    require!(
        nft.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// NFT铸造指令
/// 
/// 执行NFT铸造操作，包括：
/// - 参数验证：验证铸造参数的有效性
/// - 权限检查：验证铸造权限
/// - 服务层调用：委托给NftService执行铸造逻辑
/// - 事件发射：发射NFT铸造事件
/// 
/// # 参数
/// - ctx: NFT铸造账户上下文
/// - params: NFT铸造参数
/// 
/// # 返回
/// - Result<()>: 铸造操作结果
pub fn mint_nft(
    ctx: Context<MintNft>,
    params: MintNftParams,
) -> Result<()> {
    // 参数验证
    validate_mint_nft_params(&params)?;
    
    // 权限检查
    check_mint_authority_permission(&ctx.accounts.authority, &ctx.accounts.nft)?;
    
    let nft = &mut ctx.accounts.nft;
    let authority = &ctx.accounts.authority;
    
    // 创建NFT服务实例
    let service = NftService::new();
    
    // 执行NFT铸造
    service.mint(
        nft,
        &params.metadata_uri,
        &params.name,
        &params.symbol,
        &params.exec_params,
    )?;
    
    // 发射NFT铸造事件
    emit!(AssetMinted {
        basket_id: nft.id,
        amount: 1, // NFT铸造数量为1
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::NFT,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 