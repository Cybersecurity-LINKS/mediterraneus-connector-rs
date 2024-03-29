// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::BTreeMap;

use actix_web::{get, patch, post};
use actix_web::{web, HttpResponse};
use deadpool_postgres::Pool;
use serde_json::json;

use crate::dtos::{IdentityRequest, CredentialRequest, SignDataRequest, PresentationRequest, QueryEthAddress};
use crate::errors::ConnectorError;
use crate::models::identity::Identity;
use crate::repository::identity_operations::IdentityExt;
use crate::utils::iota::IotaState;

#[post("/identities")] 
async fn create_identity(
    req_body: web::Json<IdentityRequest>, 
    db_pool: web::Data<Pool>,
    iota_state: web::Data<IotaState>
) -> Result<HttpResponse, ConnectorError> {
    log::info!("controller create_identity");
    let pg_client = db_pool.get().await.map_err(ConnectorError::PoolError)?;
    let (doc, fragment) = iota_state.create_did(Some(req_body.eth_address.clone())).await?;

    let new_identity = Identity {
        id: None,
        eth_address: req_body.eth_address.clone(),
        did: doc.id().to_string(),
        fragment: fragment,
        vcredential: None,
    };
    let created_identity = pg_client.insert_identity(&new_identity).await?;

    Ok(HttpResponse::Ok().json(created_identity))
}

#[get("/identities")]
async fn get_identity(
    query_params: web::Query<QueryEthAddress>, 
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, ConnectorError> {
    log::info!("controller get_identity");
    let pg_client = db_pool.get().await.map_err(ConnectorError::PoolError)?;
    let identity = pg_client.get_identity_with_eth_addr(&query_params.eth_address).await?;

    Ok(HttpResponse::Ok().json(identity))
}

#[patch("/identities")] // TODO: since we modify just the credential, is the patch correct? 
async fn patch_identity(
    query_params: web::Query<QueryEthAddress>, 
    req_body: web::Json<CredentialRequest>,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, ConnectorError> {
    let pg_client = db_pool.get().await.map_err(ConnectorError::PoolError)?;
    // TODO: add credential verification
    let identity = pg_client.set_credential(&query_params.eth_address, &req_body.credential_jwt).await?;

    Ok(HttpResponse::Ok().json(identity))
}

#[post("/identities/{identity_id}/sign-data")] 
async fn sign_data(
    path: web::Path<i64>,
    req_body: web::Json<SignDataRequest>, 
    db_pool: web::Data<Pool>,
    iota_state: web::Data<IotaState>
) -> Result<HttpResponse, ConnectorError> {
    log::info!("controller sign_data");
    let pg_client = db_pool.get().await.map_err(ConnectorError::PoolError)?;
    let identity_id = path.into_inner();    
    let identity = pg_client.get_identity(identity_id).await?;
    let jws = iota_state.sign_data(identity, req_body.payload.clone().into_bytes(), &req_body.nonce).await?;

    Ok(HttpResponse::Ok().json(json!({"ssiSignature": jws.as_str()})))
}

//TODO: pass also expiration time
#[post("/identities/{identity_id}/gen-presentation")] 
async fn gen_presentation(
    path: web::Path<i64>,
    req_body: web::Json<PresentationRequest>, 
    db_pool: web::Data<Pool>,
    iota_state: web::Data<IotaState>
) -> Result<HttpResponse, ConnectorError> {
    log::info!("controller gen_presentation");
    let pg_client = db_pool.get().await.map_err(ConnectorError::PoolError)?;
    let identity_id = path.into_inner();    
    let identity = pg_client.get_identity(identity_id).await?;

    let wallet_signature_claim = match req_body.eth_signature.clone() {
        Some(eth_signature) => {
            let mut wallet_signature_claim = BTreeMap::new();
            wallet_signature_claim.insert("walletSignature".to_string(), serde_json::Value::String(eth_signature)); 
            Some(wallet_signature_claim)
        }
        None => None,
    };
    let presentetion_jwt = iota_state.gen_presentation(identity, req_body.challenge.clone(), wallet_signature_claim).await?;

    Ok(HttpResponse::Ok().json(json!({"presentation": presentetion_jwt.as_str()})))
}

// this function could be located in a different module
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg
    .service(create_identity)
    .service(get_identity)     
    .service(patch_identity)       
    .service(sign_data)
    .service(gen_presentation);
}