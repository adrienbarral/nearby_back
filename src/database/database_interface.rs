use crate::models::user;
use actix_web::web::Data;
use chrono::{DateTime, FixedOffset, Utc};
use futures::StreamExt;
use mongodb::{bson, bson::bson, bson::doc, Client, Collection};

#[derive(Debug)]
pub struct DatabaseError {
    pub message: String,
}

impl From<mongodb::error::Error> for DatabaseError {
    fn from(db_error: mongodb::error::Error) -> Self {
        return DatabaseError {
            message: std::format!("Database Error : {}", db_error),
        };
    }
}

impl From<bson::de::Error> for DatabaseError {
    fn from(bson_error: bson::de::Error) -> Self {
        return DatabaseError {
            message: std::format!("BSON Error : {}", bson_error),
        };
    }
}
// TODO : Use that : https://developer.mongodb.com/article/serde-improvements/
#[derive(Clone)]
pub struct DataBaseInterface {
    client: Client,
    available_collection: Collection,
}

pub enum ReplacedOrInserted {
    Replaced,
    Inserted,
}

impl DataBaseInterface {
    pub async fn new() -> Result<DataBaseInterface, DatabaseError> {
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
    ) -> Result<ReplacedOrInserted, DatabaseError> {
        // We start by looking for this user in the available users :
        let filter = doc! {"phone_number_hash": user.phone_number_hash.clone()};
        let replaced = self
            .available_collection
            .find_one_and_replace(filter, user.to_bson_document(), None)
            .await?;

        if let Some(_) = replaced {
            // Ok we found the user and we replace its status, we can return
            return Ok(ReplacedOrInserted::Replaced);
        }

        // Else, we must insert a new user available into the collection :
        let _ = self
            .available_collection
            .insert_one(user.to_bson_document(), None)
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
    ) -> Result<Vec<user::LocalizedUser>, DatabaseError> {
        let pipeline = vec![
            create_nearby_stage(my_phone_hash, my_latitude, my_longitude, max_distance_m),
            create_projection_stage(),
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

    /**
     * Remove all user in database that are no longuer available.
     * Return the number of user deleted from the base.
     */
    pub async fn remove_available_until(
        self: &DataBaseInterface,
        date_time: DateTime<FixedOffset>,
    ) -> Result<i64, DatabaseError> {
        let date_time_utc: DateTime<Utc> = DateTime::from(date_time);
        let query = doc! {"available_until": doc! {"$lt": date_time_utc}};
        let delete_res = self.available_collection.delete_many(query, None).await?;

        return Ok(delete_res.deleted_count);
    }

    /**
     * Usefull for testing, will return the number of deleted items.
     */
    #[cfg(test)]
    pub async fn clear_database(self: &DataBaseInterface) -> Result<i64, DatabaseError> {
        let res = self
            .available_collection
            .delete_many(doc! {}, None)
            .await
            .map(|res| res.deleted_count)?;
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
            "maxDistance": max_distance_m,
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
        let deleted = database.clear_database().await.expect("Can't clean DB");
        println!("{} document deleted", deleted);
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
            sylvester.phone_number_hash
        )
    }

    #[tokio::test]
    async fn test_we_remove_user_no_longuer_available() {
        let database = prepare_test().await;

        let available = user::User {
            phone_number_hash: String::from("Available"),
            latitude: 43.2255228,
            longitude: 6.3516515645,
            available_until: DateTime::parse_from_rfc3339("2021-05-21T18:21:00+00:00")
                .expect("Can't parse date"),
            contacts_phone_number_hash: vec![],
        };

        let not_available = user::User {
            phone_number_hash: String::from("Not Available"),
            latitude: 43.2255228,
            longitude: 6.3516515645,
            available_until: DateTime::parse_from_rfc3339("2021-05-21T18:20:00+00:00")
                .expect("Can't parse date"),
            contacts_phone_number_hash: vec![],
        };

        let _ = database
            .set_user_available(&available)
            .await
            .expect("Can't add user");

        let _ = database
            .set_user_available(&not_available)
            .await
            .expect("Can't add user");

        let now =
            DateTime::parse_from_rfc3339("2021-05-21T18:20:30+00:00").expect("Can't parse date !");

        let count = database
            .remove_available_until(now)
            .await
            .expect("Can't remove users");
        assert_eq!(count, 1);

        let mut cursor = database
            .available_collection
            .find(doc! {}, None)
            .await
            .expect("Can't get users");
        // We check that the remaining user is the good one :
        let mut availables_users_hash: Vec<String> = Vec::new();
        while let Some(doc) = cursor.next().await {
            let user = doc.expect("Error when retrieving users");
            availables_users_hash.push(String::from(
                user.get_str("phone_number_hash")
                    .expect("Can't find key phone number hash"),
            ));
        }
        assert_eq!(availables_users_hash.len(), 1);
        assert_eq!(
            *availables_users_hash.get(0).expect("Incorrect length"),
            available.phone_number_hash
        );
    }
}
