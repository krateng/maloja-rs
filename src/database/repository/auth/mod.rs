use sea_orm::{NotSet, QueryFilter, Set};
use sea_orm::{ColumnTrait, EntityTrait};
use crate::database::errors::MalojaError;
use crate::database::repository::MalojaDB;
use crate::entity::{Client, ClientActiveModel, ClientColumn, ClientModel};

impl MalojaDB {

    pub async fn get_or_create_client_by_identifier(&self, identifier: String) -> Result<u32, MalojaError> {
        let result = Client::find().filter(ClientColumn::Identifier.eq(&identifier)).one(&self.connection).await?;
        if let Some(client) = result {
            println!("Found {}", identifier);
            return Ok(client.id);
        }
        println!("Not found: {}", identifier);

        let insert = ClientActiveModel {
            id: NotSet,
            identifier: Set(identifier),
            permissions: Set(None),
            active: Set(false),
        };
        let result = Client::insert(insert).exec(&self.connection).await?;
        Ok(result.last_insert_id)
    }

}