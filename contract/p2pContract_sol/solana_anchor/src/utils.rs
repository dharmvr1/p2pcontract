use {
    crate::PoolError,
    anchor_lang::{
        prelude::{AccountInfo,},
        solana_program::{
            program::{invoke_signed, invoke},
            system_instruction:: {transfer},
            entrypoint:: {ProgramResult},
        },
    },
};

//##Ans
//There are some function related to token control. These functions invoke spl-token library endpoints.
//You can see info about extra questions.
// https://doc.rust-lang.org/rust-by-example/fn/closures.html
// https://doc.rust-lang.org/std/result/enum.Result.html#method.map_err
// https://stackoverflow.com/questions/37639276/when-should-inline-be-used-in-rust

//## Why do we need to create this struct? 
//## Are not we using regular transfer of the token that works with solana built-in methods?
///TokenTransferParams
pub struct TokenTransferParams<'a: 'b, 'b> {
    /// CHECK:
    pub source: AccountInfo<'a>,
    /// CHECK:
    pub destination: AccountInfo<'a>,
    /// CHECK:
    pub amount: u64,
    /// CHECK:
    pub authority: AccountInfo<'a>,
    /// CHECK:
    pub authority_signer_seeds: &'b [&'b [u8]],
    /// CHECK:
    pub token_program: AccountInfo<'a>,
}

//## Why do we need to create this function? 
//## Are not we using regular transfer of the token that works with solana built-in methods?
#[inline(always)]  //## Why do we need to use inline(always) macro here?
pub fn spl_token_transfer(params: TokenTransferParams<'_, '_>) -> ProgramResult {
    let TokenTransferParams {
        source,
        destination,
        authority,
        token_program,
        amount,
        authority_signer_seeds,
    } = params;

    let result = invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            source.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?,
        &[source, destination, authority, token_program],
        &[authority_signer_seeds],
    );

    result.map_err(|_| PoolError::TokenTransferFailed.into())
}

pub struct SolTransferParams<'a> {
    /// CHECK:
    pub source: AccountInfo<'a>,
    /// CHECK:
    pub destination: AccountInfo<'a>,
    /// CHECK:
    pub amount: u64,
}

//## Why do we need to create this function? 
//## Are not we using regular transfer of the token that works with solana built-in methods?
#[inline(always)]  //## Why do we need to use inline(always) macro here?
pub fn sol_transfer(params: SolTransferParams<'_>) -> ProgramResult {
    let SolTransferParams {
        source,
        destination,
        amount,
    } = params;

    // Does the from account have enough lamports to transfer?
    if **source.try_borrow_lamports()? < amount {
        return Err(PoolError::InsufficentFunds.into());
    }
    // Debit from_account and credit to_account
    **source.try_borrow_mut_lamports()? -= amount;
    **destination.try_borrow_mut_lamports()? += amount;
    Ok(())
}

//## Why do we need to create this struct? 
//## Are not we using regular transfer of the token that works with solana built-in methods?
///TokenTransferParams
pub struct TokenTransferParamsWithoutSeed<'a> {
    /// CHECK:
    pub source: AccountInfo<'a>,
    /// CHECK:
    pub destination: AccountInfo<'a>,
    /// CHECK:
    pub amount: u64,
    /// CHECK:
    pub authority: AccountInfo<'a>,
    /// CHECK:
    pub token_program: AccountInfo<'a>,
}

//## Why do we need to create this function? 
//## Are not we using regular transfer of the token that works with solana built-in methods?
#[inline(always)] //## Why do we need to use inline(always) macro here?
pub fn spl_token_transfer_without_seed(params: TokenTransferParamsWithoutSeed<'_>) -> ProgramResult {
    let TokenTransferParamsWithoutSeed {
        source,
        destination,
        authority,
        token_program,
        amount,
    } = params;

    let result = invoke(
        &spl_token::instruction::transfer(
            token_program.key,
            source.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?,
        &[source, destination ,authority, token_program],
    );

    result.map_err(|_| PoolError::TokenTransferFailed.into())
}

pub struct SolTransferParamsWithoutSeed<'a> {
    /// CHECK:
    pub source: AccountInfo<'a>,
    /// CHECK:
    pub destination: AccountInfo<'a>,
    /// CHECK:
    pub amount: u64,
    /// CHECK:
    pub system_program: AccountInfo<'a>,
}

#[inline(always)] //## Why do we need to use inline(always) macro here?
pub fn sol_transfer_without_seed(params: SolTransferParamsWithoutSeed<'_>) -> ProgramResult {
    let SolTransferParamsWithoutSeed {
        source,
        destination,
        system_program,
        amount,
    } = params;

    let result = invoke(
        &transfer(
            source.key,
            destination.key,
            amount,
        ),
        &[source, destination, system_program],
    );

    result.map_err(|_| PoolError::SOLTransferFailed.into())
}

//## Why do we need to create this struct? 
//## Are not we using regular transfer of the token that works with solana built-in methods?
pub struct TokenSetAuthorityParams<'a>{
    /// CHECK:
    pub authority : AccountInfo<'a>,
    /// CHECK:
    pub new_authority : AccountInfo<'a>,
    /// CHECK:
    pub account : AccountInfo<'a>,
    /// CHECK:
    pub token_program : AccountInfo<'a>,
}

//## Why do we need to create this function? 
//## Are not we using regular transfer of the token that works with solana built-in methods?
#[inline(always)]
pub fn spl_token_set_authority(params : TokenSetAuthorityParams<'_>) -> ProgramResult {
    let TokenSetAuthorityParams {
        authority,
        new_authority,
        account,
        token_program,
    } = params;

    let result = invoke(
        &spl_token::instruction::set_authority(
            token_program.key,
            account.key,
            Some(new_authority.key),
            spl_token::instruction::AuthorityType::AccountOwner,
            authority.key,
            &[],
        )?,
        &[authority,new_authority,account,token_program],
    );
    //## Can you please explain this map_err and |_| ? This is just a question about Rust&Solana
    result.map_err(|_| PoolError::TokenSetAuthorityFailed.into())
}

//## Why do we need to create this struct? 
//## Are not we using regular transfer of the token that works with solana built-in methods?
pub struct TokenMintToParams<'a> {
    /// CHECK:
    pub mint : AccountInfo<'a>,
    /// CHECK:
    pub account : AccountInfo<'a>,
    /// CHECK:
    pub owner : AccountInfo<'a>,
    /// CHECK:
    pub token_program : AccountInfo<'a>,
    /// CHECK:
    pub amount : u64,
}

//## Why do we need to create this function? 
//## Are not we using regular transfer of the token that works with solana built-in methods?
#[inline(always)]
pub fn spl_token_mint_to(params : TokenMintToParams<'_>) -> ProgramResult {
    let TokenMintToParams {
        mint,
        account,
        owner,
        token_program,
        amount,
    } = params;
    let result = invoke(
        &spl_token::instruction::mint_to(
            token_program.key,
            mint.key,
            account.key,
            owner.key,
            &[],
            amount,
        )?,
        &[mint,account,owner,token_program],
    );
    result.map_err(|_| PoolError::TokenMintToFailed.into())
}
