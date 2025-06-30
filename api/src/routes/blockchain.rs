use crate::models::blockchain::{
    AccountMetaResponse, ApiResponse, ApiSuccessResponse, CreateTokenRequest, InstructionResponse,
    InstructionResponseObjAccount, KeypairResponse,
};
use poem::{
    Result,
    error::InternalServerError,
    handler,
    http::StatusCode,
    web::{Json as PoemJson, Path},
};
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_client::GetConfirmedSignaturesForAddress2Config,
    rpc_response::RpcConfirmedTransactionStatusWithSignature,
};
use solana_sdk::{signature::Keypair, signer::Signer};
use spl_token::instruction;
use spl_token::instruction::initialize_mint;
use std::str::FromStr;
//

use base64;
use bs58;
// use poem::endpoint::Json;
// use poem::error::InternalServerError;
// use poem::handler;
// use poem::web::Json as PoemJson;
use poem::{Route, Server, listener::TcpListener};
use serde::{Deserialize, Serialize};
use solana_program::{instruction::Instruction as ProgramInstruction, pubkey::Pubkey};
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey;
// use spl_token::instruction::initialize_mint;
use std::sync::Arc;

// use solana_sdk::signature::{Keypair, Signer};
// use solana_program::pubkey::Pubkey;

// Generate keypair
#[handler]
pub async fn generate_keypair() -> PoemJson<ApiSuccessResponse<KeypairResponse>> {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey().to_string();
    let secret = bs58::encode(keypair.to_bytes()).into_string();

    PoemJson(ApiSuccessResponse {
        success: true,
        data: Some(KeypairResponse { pubkey, secret }),
    })
}

// //Create token
// #[handler]
// async fn create_token(
//     PoemJson(req): PoemJson<CreateTokenRequest>,
// ) -> PoemJson<ApiResponse<InstructionResponse>> {
//     let mint = req.mint.parse().unwrap_or_else(|_| Pubkey::new_unique());
//     let authority = req
//         .mint_authority
//         .parse()
//         .unwrap_or_else(|_| Pubkey::new_unique());

//     let ix = instruction::TokenInstruction::InitializeMint(
//         &spl_token::id(),
//         &mint,
//         &authority,
//         None,
//         req.decimals,
//     )
//     .map_err(|e| InternalServerError(e.to_string()))
//     .unwrap();

//     let accounts: Vec<AccountMetaResponse> = ix
//         .accounts
//         .iter()
//         .map(|acc| AccountMetaResponse {
//             pubkey: acc.pubkey.to_string(),
//             is_signer: acc.is_signer,
//             is_writable: acc.is_writable,
//         })
//         .collect();

//     let data = base64::encode(ix.data);

//     PoemJson(ApiResponse {
//         success: true,
//         data: Some(InstructionResponse {
//             program_id: ix.program_id.to_string(),
//             accounts,
//             instruction_data: data,
//         }),
//         error: None,
//     })
// }

#[handler]
pub async fn create_token(
    PoemJson(req): PoemJson<CreateTokenRequest>,
) -> PoemJson<ApiSuccessResponse<InstructionResponseObjAccount>> {
    let mint = req.mint.parse().unwrap_or_else(|_| Pubkey::new_unique());
    let authority = req
        .mint_authority
        .parse()
        .unwrap_or_else(|_| Pubkey::new_unique());

    let ix = initialize_mint(&spl_token::id(), &mint, &authority, None, req.decimals)
        .map_err(InternalServerError)
        .unwrap();

    let mut accounts = std::collections::HashMap::new();
    for (i, acc) in ix.accounts.iter().enumerate() {
        accounts.insert(
            format!("account_{}", i),
            AccountMetaResponse {
                pubkey: acc.pubkey.to_string(),
                is_signer: acc.is_signer,
                is_writable: acc.is_writable,
            },
        );
    }

    let data = base64::encode(ix.data);

    PoemJson(ApiSuccessResponse {
        success: true,
        data: Some(InstructionResponseObjAccount {
            program_id: ix.program_id.to_string(),
            accounts,
            instruction_data: data,
        }),
    })
}
