use crate::models::blockchain::{
    AccountMetaResponse, ApiFailResponse, ApiResponse, ApiSuccessResponse, CreateTokenRequest,
    InstructionResponse, InstructionResponseArrayAccounts, InstructionResponseObjAccount,
    InstructionSimpleResponse, KeypairResponse, MintTokenRequest, SendSolRequest,
    SignMessageRequest, SignMessageResponse, VerifyMessageRequest, VerifyMessageResponse,
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
use solana_sdk::{
    signature::{Keypair, Signature},
    signer::Signer,
};
use spl_token::instruction::initialize_mint;
use spl_token::instruction::{self, mint_to};
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
use solana_program::{
    example_mocks::solana_sdk::system_instruction, instruction::Instruction as ProgramInstruction,
    pubkey::Pubkey,
};
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

//Create Token

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

// Mint Token

#[handler]
pub async fn mint_token(
    PoemJson(req): PoemJson<MintTokenRequest>,
) -> PoemJson<ApiSuccessResponse<InstructionResponseArrayAccounts>> {
    let mint = req.mint.parse().unwrap_or_else(|_| Pubkey::new_unique());
    let destination = req
        .destination
        .parse()
        .unwrap_or_else(|_| Pubkey::new_unique());
    let authority = req
        .authority
        .parse()
        .unwrap_or_else(|_| Pubkey::new_unique());

    let ix = mint_to(
        &spl_token::id(),
        &mint,
        &destination,
        &authority,
        &[],
        req.amount,
    )
    .map_err(InternalServerError)
    .unwrap();

    let accounts: Vec<AccountMetaResponse> = ix
        .accounts
        .iter()
        .map(|acc| AccountMetaResponse {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();

    let data = base64::encode(ix.data);

    PoemJson(ApiSuccessResponse {
        success: true,
        data: Some(InstructionResponseArrayAccounts {
            program_id: ix.program_id.to_string(),
            accounts,
            instruction_data: data,
        }),
    })
}

// Sign Message

#[handler]
pub async fn sign_message(
    PoemJson(req): PoemJson<SignMessageRequest>,
) -> PoemJson<ApiResponse<SignMessageResponse>> {
    if req.message.is_empty() || req.secret.is_empty() {
        return PoemJson(ApiResponse {
            success: false,
            data: None,
            error: Some("Missing required fields".to_string()),
        });
    }

    let secret_bytes = match bs58::decode(&req.secret).into_vec() {
        Ok(bytes) => bytes,
        Err(_) => {
            return PoemJson(ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid secret key format".to_string()),
            });
        }
    };

    let keypair = match Keypair::from_bytes(&secret_bytes) {
        Ok(kp) => kp,
        Err(_) => {
            return PoemJson(ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid secret key bytes".to_string()),
            });
        }
    };

    let signature = keypair.sign_message(req.message.as_bytes());

    PoemJson(ApiResponse {
        success: true,
        data: Some(SignMessageResponse {
            signature: base64::encode(signature.as_ref()),
            public_key: keypair.pubkey().to_string(),
            message: req.message,
        }),
        error: None,
    })
}

// Verfify message

#[handler]
pub async fn verify_message(
    PoemJson(req): PoemJson<VerifyMessageRequest>,
) -> PoemJson<ApiResponse<VerifyMessageResponse>> {
    let pubkey = match bs58::decode(&req.pubkey).into_vec() {
        Ok(bytes) => match Pubkey::try_from(bytes.as_slice()) {
            Ok(pk) => pk,
            Err(_) => {
                return PoemJson(ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Invalid public key format".to_string()),
                });
            }
        },
        Err(_) => {
            return PoemJson(ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid public key base58".to_string()),
            });
        }
    };

    let signature_bytes = match base64::decode(&req.signature) {
        Ok(bytes) => bytes,
        Err(_) => {
            return PoemJson(ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid signature base64".to_string()),
            });
        }
    };

    let sig = match Signature::try_from(signature_bytes.as_slice()) {
        Ok(s) => s,
        Err(_) => {
            return PoemJson(ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid signature format".to_string()),
            });
        }
    };

    let valid = sig.verify(pubkey.as_ref(), req.message.as_bytes());
    PoemJson(ApiResponse {
        success: true,
        data: Some(VerifyMessageResponse {
            valid,
            message: req.message,
            pubkey: req.pubkey,
        }),
        error: None,
    })
}

// Send Sol
// The actual handler function
#[handler]
pub async fn send_sol(
    PoemJson(req): PoemJson<SendSolRequest>,
) -> PoemJson<ApiResponse<InstructionSimpleResponse>> {
    // Validate sender pubkey
    let from = match req.from.parse::<Pubkey>() {
        Ok(pubkey) => pubkey,
        Err(_) => {
            return PoemJson(ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid sender address".into()),
            });
        }
    };

    // Validate receiver pubkey
    let to = match req.to.parse::<Pubkey>() {
        Ok(pubkey) => pubkey,
        Err(_) => {
            return PoemJson(ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid recipient address".into()),
            });
        }
    };

    // Create system transfer instruction
    let instruction = system_instruction::transfer(&from, &to, req.lamports);

    // Prepare response
    let response = InstructionSimpleResponse {
        program_id: instruction.program_id.to_string(),
        accounts: instruction
            .accounts
            .iter()
            .map(|acc| acc.pubkey.to_string())
            .collect(),
        instruction_data: base64::encode(instruction.data),
    };

    PoemJson(ApiResponse {
        success: true,
        data: Some(response),
        error: None,
    })
}
