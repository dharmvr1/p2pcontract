pub mod utils;
use borsh::{BorshDeserialize};
use {
    crate::utils::*,
    anchor_lang::{
        prelude::*,
        AnchorDeserialize,
        AnchorSerialize,
        Key,
        solana_program::{
            sysvar::{clock::Clock},
            msg
        }      
    },
};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod solana_anchor {
    use super::*;

    pub fn init_pool(
        ctx: Context<InitPool>,
        _bump : u8,
    ) -> ProgramResult {
        msg!("Init Pool");
        let pool = &mut ctx.accounts.pool;
        
        pool.owner = *ctx.accounts.owner.key;
        pool.rand = *ctx.accounts.rand.key;
        pool.admin = *ctx.accounts.admin.key;
        pool.fee = 200;
        pool.bump = _bump;

        Ok(())
    }
    pub fn update_pool(
        ctx: Context<UpdatePool>,
        _fee : u64,
    ) -> ProgramResult {
        msg!("Update Pool");
        let pool = &mut ctx.accounts.pool;
        
        if *ctx.accounts.owner.key != pool.owner {
            return Err(PoolError::NotAdmin.into());
        }
        pool.fee = _fee;

        Ok(())
    }

    pub fn create_offer(
        ctx: Context<CreateOffer>,
        _fiat: String,
        _token_amount: u64,
        _rate: String,
        _max_limit: u64,
        _min_limit: u64,
        _payment_options: String,
        _time_limit: u64,
        _public_key: String,
        _offer_terms: String,
        _sol: bool,
    ) -> ProgramResult {
        msg!("Create Offer");
        let offer_data = &mut ctx.accounts.offer_data;
        let clock = Clock::from_account_info(&ctx.accounts.clock)?;
        let pool = &mut ctx.accounts.pool;
        
        offer_data.owner = *ctx.accounts.owner.key;
        offer_data.pool = pool.key();
        offer_data.admin_account = *ctx.accounts.admin_account.key;
        offer_data.pool_account = *ctx.accounts.pool_account.key;
        offer_data.buyer_account = *ctx.accounts.buyer_account.key;
        offer_data.token = *ctx.accounts.token.key;
        offer_data.fiat = _fiat;
        offer_data.token_amount = _token_amount; 
        offer_data.rate = _rate; 
        offer_data.max_limit = _max_limit;
        offer_data.min_limit = _min_limit;
        offer_data.payment_options = _payment_options;
        offer_data.time_limit = _time_limit;
        offer_data.offer_terms = _offer_terms;
        offer_data.public_key = _public_key;
        offer_data.created_time = clock.unix_timestamp;
        offer_data.bought = 0;
        offer_data.sol = _sol;
        offer_data.status = true;

        Ok(())
    }

    pub fn update_offer(
        ctx: Context<UpdateOffer>,
        _fiat: String,
        _token_amount: u64,
        _max_limit: u64,
        _min_limit: u64,
        _payment_options: String,
        _time_limit: u64,
        _offer_terms: String,
    ) -> ProgramResult {
        msg!("Update Offer");
        let offer_data = &mut ctx.accounts.offer_data;

        offer_data.fiat = _fiat;
        offer_data.token_amount = _token_amount;
        offer_data.max_limit = _max_limit;
        offer_data.min_limit = _min_limit;
        offer_data.payment_options = _payment_options;
        offer_data.time_limit = _time_limit;
        offer_data.offer_terms = _offer_terms;

        Ok(())
    }

    pub fn cancel_offer(
        ctx: Context<CancelOffer>
    ) -> ProgramResult {
        msg!("Cancel Offer");
        let offer_data = &mut ctx.accounts.offer_data;

        offer_data.status = false;

        Ok(())
    }

    pub fn create_order(
        ctx: Context<CreateOrder>,
        _sell_amount: u64,
        _receive_amount: String,
        _payment_option: String,
        _account_name: String,
        _email_address: String,
    ) -> ProgramResult {
        msg!("Create Order");
        let order_data = &mut ctx.accounts.order_data;
        let clock = Clock::from_account_info(&ctx.accounts.clock)?;
        let offer = &mut ctx.accounts.offer;
        let pool = &mut ctx.accounts.pool;
        
        order_data.owner = *ctx.accounts.owner.key;
        order_data.pool = pool.key();
        order_data.offer = offer.key();
        order_data.buyer = *ctx.accounts.buyer.key;
        order_data.sell_amount = _sell_amount;
        if order_data.sell_amount > offer.token_amount {
            return Err(PoolError::InvalidBuyAmount.into());
        }
        order_data.receive_amount = _receive_amount;
        order_data.payment_option = _payment_option;
        order_data.account_name = _account_name;
        order_data.email_address = _email_address;

        if !offer.sol {
            spl_token_transfer_without_seed(
                TokenTransferParamsWithoutSeed{
                    source : ctx.accounts.seller_account.clone(),
                    destination : ctx.accounts.pool_account.clone(),
                    authority : ctx.accounts.owner.clone(),
                    token_program : ctx.accounts.token_program.clone(),
                    amount : _sell_amount,
                }
            )?;
        } else {
            sol_transfer_without_seed(
                SolTransferParamsWithoutSeed {
                    source: ctx.accounts.owner.clone(),
                    destination: pool.to_account_info().clone(),
                    system_program: ctx.accounts.system_program.to_account_info().clone(),
                    amount: _sell_amount,
                }
            )?;
        }
        order_data.created_time = clock.unix_timestamp;
        order_data.buyer_confirm = false;
        order_data.seller_confirm = false;
        order_data.status = 0;

        Ok(())
    }

    pub fn buyer_confirm(
        ctx: Context<BuyerConfirm>,
    ) -> ProgramResult {
        msg!("Buyer Confirm");
        let order_data = &mut ctx.accounts.order_data;
        let user = *ctx.accounts.owner.key;

        if user != order_data.buyer {
            return Err(PoolError::NotBuyer.into());
        }

        order_data.buyer_confirm = true;

        Ok(())
    }

    pub fn confirm_order(
        ctx: Context<ConfirmOrder>,
    ) -> ProgramResult {
        msg!("Confirm Order");
        let pool = &ctx.accounts.pool;
        let offer_data = &mut ctx.accounts.offer_data;
        let order_data = &mut ctx.accounts.order_data;
        let user = *ctx.accounts.owner.key;

        if order_data.status == 1 {
            return Err(PoolError::IsDestroyed.into());
        }

        if !order_data.buyer_confirm {
            return Err(PoolError::BuyerNotConfirm.into());
        }

        if user != pool.owner && user != order_data.owner {
            return Err(PoolError::NotAdmin.into());
        }
        
        let pool_seeds = &[
            pool.rand.as_ref(),
            &[pool.bump],
        ];

        if !offer_data.sol {
            spl_token_transfer(
                TokenTransferParams{
                    source: ctx.accounts.pool_account.clone(),
                    destination: ctx.accounts.buyer_account.clone(),
                    authority: pool.to_account_info().clone(),
                    authority_signer_seeds: pool_seeds,
                    token_program: ctx.accounts.token_program.clone(),
                    amount: order_data.sell_amount * (10000 - pool.fee) / 10000,
                }
            )?;
    
            spl_token_transfer(
                TokenTransferParams{
                    source: ctx.accounts.pool_account.clone(),
                    destination: ctx.accounts.admin_account.clone(),
                    authority: pool.to_account_info().clone(),
                    authority_signer_seeds: pool_seeds,
                    token_program: ctx.accounts.token_program.clone(),
                    amount: order_data.sell_amount * pool.fee / 10000,
                }
            )?;
        } else {
            sol_transfer(
                SolTransferParams {
                    source: pool.to_account_info().clone(),
                    destination: ctx.accounts.buyer.clone(),
                    amount: order_data.sell_amount * (10000 - pool.fee) / 10000,
                }
            )?;

            sol_transfer(
                SolTransferParams{
                    source: pool.to_account_info().clone(),
                    destination: ctx.accounts.admin.clone(),
                    amount: order_data.sell_amount * pool.fee / 10000,
                }
            )?;
        }
        

        offer_data.token_amount -= order_data.sell_amount;
        offer_data.bought += order_data.sell_amount;
        order_data.status = 1;

        if offer_data.token_amount == 0 {
            offer_data.status = true;
        }

        Ok(())
    }

    pub fn create_dispute(
        ctx: Context<Dispute>,
        _dispute_reason: u8,
        _dispute_explain: String,
        _dispute_img: String,
    ) -> ProgramResult {
        msg!("Create Dispute");
        let order_data = &mut ctx.accounts.order_data;

        if *ctx.accounts.owner.key != order_data.buyer && *ctx.accounts.owner.key != order_data.owner {
            return Err(PoolError::NotBuyer.into());
        }

        order_data.dispute_reason = _dispute_reason;
        order_data.dispute_explain = _dispute_explain;
        order_data.dispute_img = _dispute_img;

        Ok(())
    }

    pub fn cancel_order(
        ctx: Context<CancelOrder>
    ) -> ProgramResult {
        msg!("Cancel Order");
        let pool = &ctx.accounts.pool;
        let order_data = &mut ctx.accounts.order_data;
        let offer_data = &mut ctx.accounts.offer_data;

        if *ctx.accounts.owner.key != order_data.buyer && 
            *ctx.accounts.owner.key != order_data.owner && 
            *ctx.accounts.owner.key != pool.owner {
            return Err(PoolError::NotCreater.into());
        }

        if order_data.status == 1 {
            return Err(PoolError::IsCompleted.into());
        }

        let pool_seeds = &[
            pool.rand.as_ref(),
            &[pool.bump],
        ];

        if !offer_data.sol {
            spl_token_transfer(
                TokenTransferParams{
                    source : ctx.accounts.pool_account.clone(),
                    destination : ctx.accounts.seller_account.clone(),
                    authority : pool.to_account_info().clone(),
                    authority_signer_seeds : pool_seeds,
                    token_program : ctx.accounts.token_program.clone(),
                    amount : order_data.sell_amount,
                }
            )?;
        } else {
            sol_transfer(
                SolTransferParams {
                    source: pool.to_account_info().clone(),
                    destination: ctx.accounts.seller.clone(),
                    amount: order_data.sell_amount,
                }
            )?;
        }


        order_data.status = 2;

        Ok(())
    }
    
    pub fn create_user(
        ctx: Context<CreateUser>
    ) -> ProgramResult {
        msg!("Create User");
        let pool = &ctx.accounts.pool;
        let user_info = &mut ctx.accounts.user_info;

        user_info.user = *ctx.accounts.user.key;
        user_info.pool = pool.key();
        user_info.verified = false;
        user_info.thumbs_up = 0;
        user_info.thumbs_down = 0;

        Ok(())
    }
    
    pub fn verify_user(
        ctx: Context<VerifyUser>
    ) -> ProgramResult {
        msg!("Verify User");
        let pool = &ctx.accounts.pool;
        let user_info = &mut ctx.accounts.user_info;

        if *ctx.accounts.owner.key != pool.owner {
            return Err(PoolError::NotAdmin.into());
        }

        user_info.verified = true;

        Ok(())
    }

    pub fn update_user(
        ctx: Context<UpdateUser>,
        _nickname: String,
        _language: u8,
        _region: u8
    ) -> ProgramResult {
        msg!("Update User");
        let user_info = &mut ctx.accounts.user_info;

        if _nickname != "" {
            user_info.nickname = _nickname;
        }

        if _language != 0 {
            user_info.language = _language;
        }

        if _region != 0 {
            user_info.region = _region;
        }

        Ok(())
    }

    pub fn thumb_user (
        ctx: Context<ThumbUser>,
        _thumb_up: bool,
    ) -> ProgramResult {
        msg!("Feedback");
        let user_info = &mut ctx.accounts.user_info;
        let order_data = &mut ctx.accounts.order_data;

        if _thumb_up {
            user_info.thumbs_up += 1;
        } else {
            user_info.thumbs_down += 1;
        }

        order_data.feedback = true;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(_bump: u8)]
pub struct InitPool<'info> {
    /// CHECK:
    #[account(mut, signer)]
    owner: AccountInfo<'info>,
    /// CHECK:
    #[account(init, seeds=[(*rand.key).as_ref()], bump, payer = owner, space = 8 + POOL_SIZE)]
    pool: ProgramAccount<'info, Pool>,
    /// CHECK:
    admin: AccountInfo<'info>,
    /// CHECK:
    rand: AccountInfo<'info>,
    /// CHECK:
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePool<'info> {
    /// CHECK:
    #[account(mut, signer)]
    owner: AccountInfo<'info>, 
    /// CHECK:
    #[account(mut, seeds=[pool.rand.as_ref()], bump)]
    pool: ProgramAccount<'info, Pool>,
    /// CHECK:
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateOffer<'info> {
    /// CHECK:
    #[account(mut, signer)]
    owner: AccountInfo<'info>, 
    /// CHECK:
    #[account(mut, seeds=[pool.rand.as_ref()], bump)]
    pool: ProgramAccount<'info, Pool>,
    /// CHECK:
    #[account(mut, owner = spl_token::id())]
    admin_account: AccountInfo<'info>,
    /// CHECK:
    #[account(mut, owner = spl_token::id())]
    pool_account: AccountInfo<'info>,
    /// CHECK:
    #[account(mut, owner = spl_token::id())]
    buyer_account: AccountInfo<'info>,
    /// CHECK:
    #[account(mut, owner = spl_token::id())]
    token: AccountInfo<'info>,
    /// CHECK:
    #[account(init, payer = owner, space = 8 + OFFERDATA_SIZE)]
    offer_data: ProgramAccount<'info, OfferData>,
    /// CHECK:
    #[account(address = spl_token::id())]
    token_program: AccountInfo<'info>,
    /// CHECK:
    system_program: Program<'info, System>,
    /// CHECK:
    clock : AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct UpdateOffer<'info> {
    /// CHECK:
    #[account(mut, signer)]
    owner: AccountInfo<'info>, 
    /// CHECK:
    #[account(mut)]
    offer_data: ProgramAccount<'info, OfferData>,
    /// CHECK:
    #[account(address = spl_token::id())]
    token_program: AccountInfo<'info>,
    /// CHECK:
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelOffer<'info> {
    /// CHECK:
    #[account(mut, signer)]
    owner: AccountInfo<'info>, 
    /// CHECK:
    #[account(mut)]
    offer_data: ProgramAccount<'info, OfferData>,
    /// CHECK:
    #[account(address = spl_token::id())]
    token_program: AccountInfo<'info>,
    /// CHECK:
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateOrder<'info> {
    /// CHECK:
    #[account(mut, signer)]
    owner: AccountInfo<'info>, 
    /// CHECK:
    #[account(mut, seeds=[pool.rand.as_ref()], bump)]
    pool: ProgramAccount<'info, Pool>,
    /// CHECK:
    #[account(mut)]
    offer: ProgramAccount<'info, OfferData>,
    /// CHECK:
    #[account(mut)]
    buyer: AccountInfo<'info>,
    /// CHECK:
    #[account(mut, owner = spl_token::id())]
    seller_account: AccountInfo<'info>,
    /// CHECK:
    #[account(mut, owner = spl_token::id())]
    pool_account: AccountInfo<'info>,
    /// CHECK:
    #[account(init, payer = owner, space = 8 + ORDERDATA_SIZE)]
    order_data: ProgramAccount<'info, OrderData>,
    /// CHECK:
    #[account(address = spl_token::id())]
    token_program: AccountInfo<'info>,
    /// CHECK:
    system_program: Program<'info, System>,
    /// CHECK:
    clock : AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct BuyerConfirm<'info> {
    /// CHECK:
    #[account(mut, signer)]
    owner: AccountInfo<'info>, 
    /// CHECK:
    #[account(mut)]
    order_data: ProgramAccount<'info, OrderData>,
}

#[derive(Accounts)]
pub struct ConfirmOrder<'info> {
    /// CHECK:
    #[account(mut, signer)]
    owner: AccountInfo<'info>, 
    /// CHECK:
    #[account(mut)]
    pool: ProgramAccount<'info, Pool>,
    /// CHECK:
    #[account(mut)]
    offer_data: ProgramAccount<'info, OfferData>,
    /// CHECK:
    #[account(mut)]
    order_data: ProgramAccount<'info, OrderData>,
    /// CHECK:
    #[account(mut, owner = spl_token::id())]
    pool_account: AccountInfo<'info>,
    /// CHECK:
    #[account(mut, owner = spl_token::id())]
    buyer_account: AccountInfo<'info>,
    /// CHECK:
    #[account(mut, owner = spl_token::id())]
    admin_account: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    buyer: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    admin: AccountInfo<'info>,
    /// CHECK:
    #[account(mut, owner = spl_token::id())]
    token: AccountInfo<'info>,
    /// CHECK:
    #[account(address = spl_token::id())]
    token_program: AccountInfo<'info>,
    /// CHECK:
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Dispute<'info> {
    /// CHECK:
    #[account(mut, signer)]
    owner: AccountInfo<'info>, 
    /// CHECK:
    #[account(mut)]
    order_data: ProgramAccount<'info, OrderData>,
}

#[derive(Accounts)]
pub struct CancelOrder<'info> {
    /// CHECK:
    #[account(mut, signer)]
    owner: AccountInfo<'info>, 
    /// CHECK:
    #[account(mut)]
    pool: ProgramAccount<'info, Pool>,
    /// CHECK:
    #[account(mut)]
    order_data: ProgramAccount<'info, OrderData>,
    /// CHECK:
    #[account(mut)]
    offer_data: ProgramAccount<'info, OfferData>,
    /// CHECK:
    #[account(mut, owner = spl_token::id())]
    pool_account: AccountInfo<'info>,
    /// CHECK:
    #[account(mut, owner = spl_token::id())]
    seller_account: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    seller: AccountInfo<'info>,
    /// CHECK:
    #[account(address = spl_token::id())]
    token_program: AccountInfo<'info>,
    /// CHECK:
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateUser<'info> {
    /// CHECK:
    #[account(mut, signer)]
    owner: AccountInfo<'info>, 
    /// CHECK:
    #[account(mut, seeds=[pool.rand.as_ref()], bump)]
    pool: ProgramAccount<'info, Pool>,
    /// CHECK:
    #[account(init, payer = owner, space = 8 + USERINFO_SIZE)]
    user_info: ProgramAccount<'info, UserInfo>,
    /// CHECK:
    user: AccountInfo<'info>,
    /// CHECK:
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VerifyUser<'info> {
    /// CHECK:
    #[account(mut, signer)]
    owner: AccountInfo<'info>, 
    /// CHECK:
    #[account(mut, seeds=[pool.rand.as_ref()], bump)]
    pool: ProgramAccount<'info, Pool>,
    /// CHECK:
    #[account(mut)]
    user_info: ProgramAccount<'info, UserInfo>,
    /// CHECK:
    user: AccountInfo<'info>,
    /// CHECK:
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateUser<'info> {
    /// CHECK:
    #[account(mut, signer)]
    owner: AccountInfo<'info>, 
    /// CHECK:
    #[account(mut, seeds=[pool.rand.as_ref()], bump)]
    pool: ProgramAccount<'info, Pool>,
    /// CHECK:
    #[account(mut)]
    user_info: ProgramAccount<'info, UserInfo>,
    /// CHECK:
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ThumbUser<'info> {
    /// CHECK:
    #[account(mut, signer)]
    owner: AccountInfo<'info>, 
    /// CHECK:
    #[account(mut, seeds=[pool.rand.as_ref()], bump)]
    pool: ProgramAccount<'info, Pool>,
    /// CHECK:
    #[account(mut)]
    user_info: ProgramAccount<'info, UserInfo>,
    /// CHECK:
    #[account(mut)]
    order_data: ProgramAccount<'info, OrderData>,
    /// CHECK:
    system_program: Program<'info, System>,
}

pub const POOL_SIZE: usize = 32 + 32 + 32 + 8 + 1;
pub const OFFERDATA_SIZE: usize = 32 + 32 + 32 + 32 + 32 + 32 + 4 + 10 + 8 + 8 + 4 + 10 + 8 + 8 + 4 + 100 + 8 + 4 + 200 + 8 + 4 + 200 + 1 + 1;
pub const ORDERDATA_SIZE: usize = 32 + 32 + 32 + 32 + 8 + 4 + 20 + 4 + 20 + 4 + 20 + 4 + 50 + 1 + 1 + 8 + 1 + 4 + 300 + 4 + 100 + 1 + 1;
pub const USERINFO_SIZE: usize = 32 + 32 + 1 + 4 + 20 + 1 + 1 + 8 + 8;

#[account]
pub struct Pool {
    pub owner: Pubkey,
    pub rand: Pubkey,
    pub admin: Pubkey,
    pub fee: u64,
    pub bump: u8,
}

#[account]
pub struct OfferData {
    pub owner: Pubkey,
    pub pool: Pubkey,
    pub admin_account: Pubkey,
    pub pool_account: Pubkey,
    pub buyer_account: Pubkey,
    pub token: Pubkey,
    pub fiat: String,
    pub token_amount: u64,
    pub bought: u64,
    pub rate: String,
    pub max_limit: u64,
    pub min_limit: u64,
    pub payment_options: String,
    pub time_limit: u64,
    pub offer_terms: String,
    pub created_time: i64,
    pub public_key: String,
    pub sol: bool,
    pub status: bool,
}

#[account]
pub struct OrderData {
    pub owner: Pubkey,
    pub pool: Pubkey,
    pub offer: Pubkey,
    pub buyer: Pubkey,
    pub sell_amount: u64,
    pub receive_amount: String,
    pub payment_option: String,
    pub account_name: String,
    pub email_address: String,
    pub buyer_confirm: bool,
    pub seller_confirm: bool,
    pub created_time : i64,
    pub dispute_reason: u8,
    pub dispute_explain: String,
    pub dispute_img: String,
    pub feedback: bool,
    pub status: u8,
}

#[account]
pub struct UserInfo {
    pub user: Pubkey,
    pub pool: Pubkey,
    pub verified: bool,
    pub nickname: String,
    pub language: u8,
    pub region: u8,
    pub thumbs_up: u64,
    pub thumbs_down: u64
}

#[error]
pub enum PoolError {
    #[msg("Token mint to failed")]
    TokenMintToFailed,

    #[msg("Token set authority failed")]
    TokenSetAuthorityFailed,

    #[msg("Token transfer failed")]
    TokenTransferFailed,

    #[msg("SOL transfer failed")]
    SOLTransferFailed,

    #[msg("Don't have enough SOL")]
    InsufficentFunds,
    
    #[msg("Invalid User")]
    InvalidUser,
    
    #[msg("Not Admin")]
    NotAdmin,

    #[msg("Not Buyer")]
    NotBuyer,

    #[msg("Token is invalid")]
    InvalidToken,

    #[msg("Not Creater")]
    NotCreater,

    #[msg("Trade is completed")]
    IsCompleted,

    #[msg("Trade is destroyed")]
    IsDestroyed,

    #[msg("Buy amount is not Correct")]
    InvalidBuyAmount,

    #[msg("Buy not confirm order")]
    BuyerNotConfirm,
}