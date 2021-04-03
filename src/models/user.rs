use chrono::{DateTime, FixedOffset, Utc};
use mongodb::bson::{bson, doc, Bson, Document};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct User {
    pub phone_number_hash: String,
    pub latitude: f64,
    pub longitude: f64,
    pub available_until: DateTime<FixedOffset>,
    pub contacts_phone_number_hash: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct LocalizedUser {
    pub phone_number_hash: String,
    pub distance: f32,
}

impl User {
    pub fn to_bson_document(&self) -> Document {
        let utc_available_datetime: DateTime<Utc> = DateTime::from(self.available_until);
        let mut contacts_phone =
            mongodb::bson::Array::with_capacity(self.contacts_phone_number_hash.len());

        for contact_phone_hash in self.contacts_phone_number_hash.iter() {
            contacts_phone.push(Bson::from(contact_phone_hash));
        }

        let res = doc! {
            "phone_number_hash": self.phone_number_hash.clone(),
            "location": doc! {
                "type": "Point",
                "coordinates": bson!([self.longitude, self.latitude])
            },
            "available_until": utc_available_datetime,
            "contacts_phone_number_hash": contacts_phone
        };
        return res;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn user_are_serializable_in_bson() {
        let user = User {
            phone_number_hash: String::from("01234"),
            latitude: 43.5,
            longitude: 5.8952895,
            available_until: DateTime::parse_from_rfc3339("2021-01-01T12:21:33+01:00")
                .expect("Can't parse date"),
            contacts_phone_number_hash: vec![String::from("56789"), String::from("0000000")],
        };

        let bson_user = user.to_bson_document();
        assert_eq!(
            bson_user
                .get_str("phone_number_hash")
                .expect("can't find phone number hash in bson doc."),
            user.phone_number_hash
        );

        let location = bson_user
            .get_document("location")
            .expect("can't find latitude in BSON Document");
        assert_eq!(
            location.get_str("type").expect("Can't find geoJSON Type"),
            "Point"
        );
        let coordinates = location
            .get_array("coordinates")
            .expect("Can't find coordinates");
        let longitude = coordinates
            .get(0)
            .expect("Coordinates array doesn't have the good size")
            .as_f64()
            .expect("Longitude is not f64 !!");

        let latitude = coordinates
            .get(1)
            .expect("Coordinates array doesn't have the good size")
            .as_f64()
            .expect("Longitude is not f64 !!");
        assert!((longitude - user.longitude).abs() < 0.0001);
        assert!((latitude - user.latitude).abs() < 0.0001);
    }
}
