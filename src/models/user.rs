use chrono::{DateTime, FixedOffset};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct User {
    pub phone_number_hash: String,
    pub latitde: f64,
    pub longitude: f64,
    pub available_until: DateTime<FixedOffset>
}
