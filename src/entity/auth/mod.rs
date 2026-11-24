use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use sea_orm::{ColIdx, DeriveActiveEnum, DeriveValueType, EnumIter};
use serde::Deserialize;

pub mod client;
pub mod login;


#[derive(EnumIter, DeriveActiveEnum, Hash, PartialEq, Eq, Debug, Clone, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ClientType {
    Session,
    ApiKey,
    Imported,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "u32", db_type = "Integer")]
pub enum Singleton {
    #[sea_orm(num_value = 1)]
    Only,
}

#[derive(Clone, PartialEq, Eq, Hash, DeriveValueType)]
pub struct StoredPasswordHash(String);
impl std::fmt::Debug for StoredPasswordHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("StoredPasswordHash(<redacted>)")
    }
}
impl StoredPasswordHash {
    pub fn from_pw(password: &str) -> StoredPasswordHash {
        let argon2 = Argon2::default();
        let hash = argon2.hash_password(password.as_bytes()).expect("Argon2 hash failed");
        Self(hash.to_string())
    }

    pub fn check_pw(&self, password: &str) -> bool {
        let argon2 = Argon2::default();
        let parsed = PasswordHash::new(&self.0).expect("Invalid password hash");
        match argon2.verify_password(password.as_bytes(), &parsed) {
            Ok(_) => true,
            Err(_) => false, // TODO: diff between wrong and error
        }
    }
}