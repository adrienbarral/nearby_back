use crate::models::user;
use futures::stream::StreamExt;
use mongodb::{bson::doc, options::FindOneAndUpdateOptions};
use mongodb::{
    error::Error,
    options::{FindOneAndDeleteOptions, FindOptions},
    Client, Collection,
};

pub struct DataBaseInterface {
    client: Client,
    available_collection: Collection,
}

impl DataBaseInterface {
    pub async fn new() -> Result<DataBaseInterface, Error> {
        let client = Client::with_uri_str("mongodb://localhost:27017/").await?;
        let collection = client.database("nearby").collection("available");
        return Ok(DataBaseInterface {
            client: client.clone(),
            available_collection: collection.clone(),
        });
    }
    pub async fn set_user_available(
        self: DataBaseInterface,
        user: &user::User,
    ) -> Result<(), Error> {
        // We start by looking for this user in the available users :
        let filter = doc! {"phone_hash": user.phone_number_hash.clone()};
        let find_options = FindOneAndUpdateOptions::builder().build();
        let mut cursor = self
            .available_collection
            .find_one_and_update(filter, user.to_bson_document(), find_options)
            .await?;
        /*let mut user_exists = false;
        if let Some(result) = cursor.next().await {
            match result {
                Ok(_) => {
                    user_exists = true;
                },
                Err(_) => {}
            }
        }*/

        return Ok(());
    }
}
