use base64::Engine;
use chrono::Utc;
use itertools::{Either, Itertools};
use crate::database::errors::MalojaError;
use crate::database::repository::MalojaDB;
use crate::entity::{Client, ClientActiveModel, ClientColumn, ClientModel};
use sea_orm::{ColumnTrait, EntityTrait};
use sea_orm::{NotSet, QueryFilter, Set};
use base64::engine::general_purpose;
use sha2::{Digest, Sha256};

impl MalojaDB {

    pub async fn get_or_create_client_by_identifier(&self, identifier: String, api_key: bool) -> Result<u32, MalojaError> {
        let result = Client::find().filter(ClientColumn::Identifier.eq(&identifier)).one(&self.connection).await?;
        if let Some(client) = result {
            return Ok(client.id);
        }

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

    pub async fn get_clients(&self) -> Result<SortedClientList, MalojaError> {
        let mut result = Client::find().all(&self.connection).await?;
        result.sort_by(|x, y| x.active.cmp(&y.active).then_with(|| x.last_used_at.cmp(&y.last_used_at)) );
        result.reverse();
        let (sessions, api_keys): (Vec<ClientModel>, Vec<ClientModel>) = result.into_iter().partition_map(|x| {
            match x.session {
                true => Either::Left(x),
                false => Either::Right(x)
            }
        });
        Ok(SortedClientList {
            api_keys,
            sessions,
        })
    }

}

/// Sorts both client types first by active, then by last used
pub struct SortedClientList {
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