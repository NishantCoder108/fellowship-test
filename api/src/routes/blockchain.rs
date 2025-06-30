use crate::models::blockchain::{ApiResponse, KeypairResponse};
use poem::{
    Result, handler,
    http::StatusCode,
    web::{Json as PoemJson, Path},
};
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_client::GetConfirmedSignaturesForAddress2Config,
    rpc_response::RpcConfirmedTransactionStatusWithSignature,
};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use std::str::FromStr;

#[handler]
pub async fn generate_keypair() -> PoemJson<ApiResponse<KeypairResponse>> {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey().to_string();
    let secret = bs58::encode(keypair.to_bytes()).into_string();

    PoemJson(ApiResponse {
        success: true,
        data: Some(KeypairResponse { pubkey, secret }),
        error: None,
    })
}
