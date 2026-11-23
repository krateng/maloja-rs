use base64::Engine;
use chrono::{DateTime, Utc};
use itertools::{Either, Itertools};
use crate::database::errors::MalojaError;
use crate::database::repository::MalojaDB;
use crate::entity::{Client, ClientActiveModel, ClientColumn, ClientModel};
use sea_orm::{ColumnTrait, EntityTrait};
use sea_orm::{NotSet, QueryFilter, Set};
use base64::engine::general_purpose;
use sea_query::Token;
use sha2::{Digest, Sha256, Sha512};

impl MalojaDB {

    pub async fn get_or_create_client_by_identifier(&self, identifier: String, api_key: bool) -> Result<u32, MalojaError> {
        let result = Client::find().filter(ClientColumn::Identifier.eq(&identifier)).one(&self.connection).await?;
        if let Some(client) = result {
            println!("Found {}", identifier);
            return Ok(client.id);
        }
        println!("Not found: {}", identifier);

        let new = generate_token();
        let insert = ClientActiveModel {
            id: NotSet,
            identifier: Set(identifier),
            active: Set(false),
            created_at: Set(Utc::now().timestamp()),
            last_used_at: Set(None),
            valid_until: Set(None),
            session: Set(!api_key),
            token_hash: Set(new.hash.to_owned()),
            display: Set(new.token_display.to_owned()),
        };
        let result = Client::insert(insert).exec(&self.connection).await?;
        Ok(result.last_insert_id)
    }

    pub async fn get_clients(&self) -> Result<ClientList, MalojaError> {
        let result = Client::find().all(&self.connection).await?;
        let (sessions, api_keys) = result.into_iter().partition_map(|x| {
            match x.session {
                true => Either::Left(x),
                false => Either::Right(x)
            }
        });
        Ok(ClientList {
            api_keys,
            sessions,
        })
    }

}

pub struct ClientList {
    pub api_keys: Vec<ClientModel>,
    pub sessions: Vec<ClientModel>,
}

struct TokenResult {
    token: String,
    token_display: String,
    hash: String,
}
fn generate_token() -> TokenResult {
    let mut buf = [0u8; 48];
    getrandom::fill(&mut buf).expect("OS RNG failed");
    let token = format!("mlj_{}", general_purpose::URL_SAFE_NO_PAD.encode(buf));
    let hash = Sha256::digest(token.as_bytes());
    let hash_hex = format!("{:x}", hash);
    let token_display = format!("{}...{}", &token[..4], &token[token.len()-4..]);
    TokenResult {
        token,
        token_display,
        hash: hash_hex,
    }

}