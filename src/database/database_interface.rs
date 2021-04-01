use crate::models::user;
use futures::StreamExt;
use mongodb::{
    bson,
    bson::doc,
    error::Error,
    options::{FindOneAndReplaceOptions, FindOptions, InsertOneOptions},
    Client, Collection,
};

// TODO : Use that : https://developer.mongodb.com/article/serde-improvements/

pub struct DataBaseInterface {
    client: Client,
    available_collection: Collection,
}
pub enum ReplacedOrInserted {
    Replaced,
    Inserted,
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
        self: &DataBaseInterface,
        user: &user::User,
    ) -> Result<ReplacedOrInserted, mongodb::error::Error> {
        // We start by looking for this user in the available users :
        let filter = doc! {"phone_number_hash": user.phone_number_hash.clone()};
        let find_options = FindOneAndReplaceOptions::builder().build();
        let replaced = self
            .available_collection
            .find_one_and_replace(filter, user.to_bson_document(), find_options)
            .await?;

        if let Some(_) = replaced {
            // Ok we found the user and we replace its status, we can return
            return Ok(ReplacedOrInserted::Replaced);
        }

        // Else, we must insert a new user available into the collection :
        let _ = self
            .available_collection
            .insert_one(user.to_bson_document(), InsertOneOptions::builder().build())
            .await?;

        return Ok(ReplacedOrInserted::Inserted);
    }

    pub async fn get_contacts_available(
        self: &DataBaseInterface,
        my_phone_hash: &String,
    ) -> Result<Vec<user::LocalizedUser>, Error> {
        let filter = doc! {"contacts_phone_number_hash": my_phone_hash.clone()};
        let projection = doc! {"phone_number_hash": 1, "latitude": 1, "longitude": 1};
        let find_options = FindOptions::builder().projection(projection).build();
        let mut cursor = self.available_collection.find(filter, find_options).await?;
        let mut res: Vec<user::LocalizedUser> = Vec::new();
        while let Some(doc) = cursor.next().await {
            match doc {
                Ok(document) => {
                    let user: user::LocalizedUser = bson::from_document(document)?;
                    res.push(user);
                }
                Err(e) => return Err(e.into()),
            }
        }

        return Ok(res);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;
    use tokio;

    async fn prepare_test() -> DataBaseInterface {
        let database = DataBaseInterface::new().await.expect("Can't connect to DB");
        let delete_res = database
            .available_collection
            .delete_many(doc! {}, mongodb::options::DeleteOptions::builder().build())
            .await
            .expect("Can't clean DB");
        println!("{} document deleted", delete_res.deleted_count);
        return database;
    }
    #[tokio::test]
    async fn test_we_can_insert_new_user() {
        let database = prepare_test().await;

        let user = user::User {
            phone_number_hash: String::from("15645612"),
            latitude: 43.2255228,
            longitude: 6.3516515645,
            available_until: DateTime::parse_from_rfc3339("2021-05-21T18:21:43+00:00")
                .expect("Can't parse date"),
            contacts_phone_number_hash: vec![],
        };
        let res = database
            .set_user_available(&user)
            .await
            .expect("Can't add user");

        assert!(std::matches!(res, ReplacedOrInserted::Inserted));

        let res2 = database
            .set_user_available(&user)
            .await
            .expect("Can't add user");

        assert!(std::matches!(res2, ReplacedOrInserted::Replaced));
    }

    #[tokio::test]
    async fn test_we_can_get_available_contacts() {
        let database = prepare_test().await;
        let user1 = user::User {
            phone_number_hash: String::from("15645612"),
            latitude: 43.2255228,
            longitude: 6.3516515645,
            available_until: DateTime::parse_from_rfc3339("2021-05-21T18:21:43+00:00")
                .expect("Can't parse date"),
            contacts_phone_number_hash: vec!["123456".to_string(), "456789".to_string()],
        };
        database
            .set_user_available(&user1)
            .await
            .expect("Can't add user");

        let user2 = user::User {
            phone_number_hash: String::from("789456"),
            latitude: 43.2255228,
            longitude: 6.3516515645,
            available_until: DateTime::parse_from_rfc3339("2021-05-21T18:21:43+00:00")
                .expect("Can't parse date"),
            contacts_phone_number_hash: vec!["456789".to_string()],
        };
        database
            .set_user_available(&user2)
            .await
            .expect("Can't add user");

        let my_phone_hash = "123456".to_string();
        let contact_availables = database
            .get_contacts_available(&my_phone_hash)
            .await
            .expect("Can't get availables contacts");
        assert_eq!(contact_availables.len(), 1);
    }
}
