use crate::models::user;
use futures::StreamExt;
use mongodb::{
    bson,
    bson::bson,
    bson::doc,
    error::Error,
    options::{AggregateOptions, FindOneAndReplaceOptions, InsertOneOptions},
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

    /**
     * Here latitude and longitde are in decimal degrees on a WGS84 ellipsoid
     * (because Mongo do the job !).
     */
    pub async fn get_contacts_available_nearby(
        self: &DataBaseInterface,
        my_phone_hash: &String,
        my_latitude: f64,
        my_longitude: f64,
        max_distance_m: f32,
    ) -> Result<Vec<user::LocalizedUser>, Error> {
        let pipeline = vec![
            create_nearby_stage(my_phone_hash, my_latitude, my_longitude, max_distance_m),
            create_projection_stage()
        ];
        let mut cursor = self.available_collection.aggregate(pipeline, None).await?;
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

fn create_nearby_stage(
    phone_hash: &String,
    latitude: f64,
    longitude: f64,
    max_distance_m: f32,
) -> bson::Document {
    return doc! {
        "$geoNear": doc! {
            "near": doc! {
                "type": "Point",
                "coordinates": bson!([longitude, latitude]),
            },
            "distanceField": "distance",
            "distanceMax": max_distance_m,
            "query": doc! {"contacts_phone_number_hash": phone_hash.clone()},
            "spherical": true
        }
    };
}

fn create_projection_stage() -> bson::Document {
    return doc! {"$project": doc! {"phone_number_hash": 1, "distance": 1}};
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
    async fn test_we_can_get_available_contacts_nearby() {
        let database = prepare_test().await;
        let sylvester = user::User {
            phone_number_hash: String::from("Sylverster Staline"),
            latitude: 43.00001,
            longitude: 6.00001,
            available_until: DateTime::parse_from_rfc3339("2021-05-21T18:21:43+00:00")
                .expect("Can't parse date"),
            contacts_phone_number_hash: vec![
                "John Lenine".to_string(),
                "Didier CrouteChef".to_string(),
            ],
        };
        database
            .set_user_available(&sylvester)
            .await
            .expect("Can't add user");

        let didier = user::User {
            phone_number_hash: String::from("Didier CrouteChef"),
            latitude: 42.0000,
            longitude: 5.0000,
            available_until: DateTime::parse_from_rfc3339("2021-05-21T18:21:43+00:00")
                .expect("Can't parse date"),
            contacts_phone_number_hash: vec!["John Lenine".to_string()],
        };
        database
            .set_user_available(&didier)
            .await
            .expect("Can't add user");

        let unknown_man = user::User {
            phone_number_hash: String::from("Unknown Man"),
            latitude: 43.0000,
            longitude: 6.0000,
            available_until: DateTime::parse_from_rfc3339("2021-05-21T18:21:43+00:00")
                .expect("Can't parse date"),
            contacts_phone_number_hash: vec!["Unknwon Man's Friend".to_string()],
        };
        database
            .set_user_available(&unknown_man)
            .await
            .expect("Can't add user");

        let my_phone_hash = "John Lenine".to_string();
        let contact_availables = database
            .get_contacts_available_nearby(&my_phone_hash, 43.000_f64, 6.000_f64, 1000_f32)
            .await
            .expect("Can't get availables contacts");

        /*
         * Here, both Unknwo Men and Dider CrouteChef are close to John.
         * The aggregation must return only Didier, because sylvester is far from Joh !
         */
        assert_eq!(contact_availables.len(), 1);
        assert_eq!(
            contact_availables
                .get(0)
                .expect("Not enough returned values")
                .phone_number_hash,
            didier.phone_number_hash
        )
    }
}
