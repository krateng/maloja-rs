use base64::Engine;
use chrono::{Days, Duration, TimeDelta, TimeZone, Utc};
use itertools::{Either, Itertools};
use crate::database::errors::MalojaError;
use crate::database::repository::MalojaDB;
use crate::entity::{Client, ClientActiveModel, ClientColumn, ClientModel, ClientType, Scrobble, ScrobbleColumn};
use sea_orm::{ColumnTrait, EntityTrait, IntoActiveModel, QuerySelect};
use sea_orm::{NotSet, QueryFilter, Set};
use base64::engine::general_purpose;
use sea_orm::prelude::DateTime;
use sea_query::{Condition, Iden, Query};
use sha2::{Digest, Sha256};


const WEB_SESSION_EXPIRY: Duration = Duration::days(30);
const EXPIRY_REFRESH_THRESHOLD: Duration = Duration::days(2);
const LAST_USED_REFRESH_THRESHOLD: Duration = Duration::seconds(30);


impl MalojaDB {

    pub async fn get_clients(&self) -> Result<SortedClientList, MalojaError> {
        let mut result = Client::find().all(&self.connection).await?;
        result.sort_by(|x, y| x.active.cmp(&y.active).then_with(|| x.last_used_at.cmp(&y.last_used_at)) );
        result.reverse();
        let (mut sessions, mut api_keys, mut imported) = (vec![], vec![], vec![]);
        result.into_iter().for_each(|x| {
            match x.client_type {
                ClientType::Session => sessions.push(x),
                ClientType::ApiKey => api_keys.push(x),
                ClientType::Imported => imported.push(x),
            }
        });
        Ok(SortedClientList {
            api_keys,
            sessions,
            imported,
        })
    }

    async fn create_client(&self, identifier: String, active: bool, created_at: Option<i64>, client_type: ClientType, custom_display: String) -> Result<(ClientModel, TokenResult), MalojaError> {
        let new = generate_token();
        let display = match client_type {
            ClientType::Session => custom_display,
            ClientType::ApiKey => new.token_display.to_owned(),
            ClientType::Imported => "".to_owned(),
        };
        let insert = ClientActiveModel {
            id: NotSet,
            identifier: Set(identifier),
            active: Set(active),
            created_at: Set(created_at),
            last_used_at: Set(None),
            valid_until: Set(None),
            token_hash: Set(new.hash.to_owned()),
            display: Set(display),
            client_type: Set(client_type),
        };
        let result = Client::insert(insert).exec_with_returning(&self.connection).await?;
        Ok((result, new))
    }

    pub async fn import_client(&self, identifier: String) -> Result<u32, MalojaError> {
        let existing = Client::find().filter(ClientColumn::Identifier.eq(&identifier)).one(&self.connection).await?;
        if let Some(client) = existing {
            Ok(client.id)
        }
        else {
            let (client, _) = self.create_client(
                identifier, false, Some(Utc::now().timestamp()), ClientType::Imported, "".to_owned()
            ).await?;
            Ok(client.id)
        }
    }

    pub async fn create_api_key(&self, identifier: String) -> Result<String, MalojaError> {
        let (_, token_result) = self.create_client(
            identifier, true, Some(Utc::now().timestamp()), ClientType::ApiKey, "".to_owned()
        ).await?;
        Ok(token_result.token_display)
    }

    // returns the session token
    pub async fn login(&self, session_id: Option<u32>, password: String, user_agent: Option<String>) -> Result<Option<String>, MalojaError> {
        let pw_correct = true;

        if pw_correct {
            if let Some(session_id) = session_id {
                let token = self.rotate_key(session_id, Some(true)).await?;
                Ok(Some(token))
            }
            else {
                let now = Utc::now().timestamp();
                let identifier: String = user_agent.clone().unwrap_or("UNKNOWN".to_string()).chars().take(7).collect();
                let identifier = format!("{}_{}", identifier, now);
                let (_, token_result) = self.create_client(
                    identifier,
                    true,
                    Some(Utc::now().timestamp()),
                    ClientType::Session,
                    user_agent.unwrap_or("".to_owned())
                ).await?;
                Ok(Some(token_result.token))
            }
        }
        else {
            Ok(None)
        }
    }

    pub async fn authenticate_session(&self, session_token: String) -> Result<bool, MalojaError> {
        let hash = hash_token(session_token);
        let now = Utc::now();
        let c = Client::find()
            .filter(ClientColumn::ClientType.eq(ClientType::Session))
            .filter(Condition::any()
                .add(ClientColumn::ValidUntil.gt(Utc::now().timestamp()))
                .add(ClientColumn::ValidUntil.is_null())
            )
            .filter(ClientColumn::Active.eq(true))
            .filter(ClientColumn::TokenHash.eq(hash))
            .one(&self.connection).await?;

        if let Some(client) = c {
            let last_used_diff = now - Utc.timestamp_opt(client.last_used_at.unwrap_or(0), 0).unwrap();
            let expiry_diff = Utc.timestamp_opt(client.valid_until.unwrap_or(0), 0).unwrap() - now;
            if last_used_diff > LAST_USED_REFRESH_THRESHOLD || expiry_diff < (WEB_SESSION_EXPIRY - EXPIRY_REFRESH_THRESHOLD) {
                let mut client_active = client.into_active_model();
                client_active.last_used_at = Set(Some(now.timestamp()));
                client_active.valid_until = Set(Some((now + WEB_SESSION_EXPIRY).timestamp()));
                Client::update(client_active).exec_without_returning(&self.connection).await?;
            }
            Ok(true)
        }
        else {
            Ok(false)
        }
    }

    pub async fn change_client_type(&self, client_id: u32, target_type: ClientType) -> Result<ClientModel, MalojaError> {
        let client = Client::find_by_id(client_id).one(&self.connection).await?;
        if let Some(client) = client {
            let mut active = client.into_active_model();
            active.client_type = Set(target_type);
            let res = Client::update(active).exec(&self.connection).await?;
            Ok(res)
        }
        else {
            Err(MalojaError::EntityNotFound { id: client_id })
        }
    }

    pub async fn rotate_key(&self, client_id: u32, set_active: Option<bool>) -> Result<String, MalojaError> {
        let client = Client::find_by_id(client_id).one(&self.connection).await?;
        if let Some(client) = client {
            let token = generate_token();
            let mut active = client.into_active_model();
            active.token_hash = Set(token.hash);
            active.display = Set(token.token_display);
            if let Some(set_active) = set_active {
                active.active = Set(set_active);
                if !set_active {
                    active.valid_until = Set(None);
                }
            }
            Client::update(active).exec_without_returning(&self.connection).await?;
            Ok(token.token)
        }
        else {
            Err(MalojaError::EntityNotFound { id: client_id })
        }
    }

    pub async fn clear_unused(&self, client_type: ClientType) -> Result<(), MalojaError> {
        let used_clients = Query::select().column(ScrobbleColumn::OriginClientId).from(Scrobble).and_where(ScrobbleColumn::OriginClientId.is_not_null()).to_owned();

        Client::delete_many()
            .filter(ClientColumn::Id.not_in_subquery(used_clients))
            .filter(ClientColumn::ClientType.eq(client_type))
            .filter(ClientColumn::Active.eq(false))
            .exec(&self.connection).await?;

        Ok(())
    }

}

/// Sorts both client types first by active, then by last used
pub struct SortedClientList {
    pub api_keys: Vec<ClientModel>,
    pub sessions: Vec<ClientModel>,
    pub imported: Vec<ClientModel>,
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
    let hash_hex = hash_token(token.clone());
    let token_display = format!("{}...{}", &token[..4], &token[token.len()-4..]);
    TokenResult {
        token,
        token_display,
        hash: hash_hex,
    }
}

fn hash_token(token: String) -> String {
    let hash = Sha256::digest(token.as_bytes());
    let hash_hex = format!("{:x}", hash);
    hash_hex
}