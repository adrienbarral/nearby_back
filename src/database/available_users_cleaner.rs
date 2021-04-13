use actix::prelude::*;
use crate::database::database_interface::DataBaseInterface;
use core::time::Duration;
use chrono::{DateTime, FixedOffset, Utc};

pub struct AvailableUserCleaner {
    database_interface: DataBaseInterface,
}
impl Actor for AvailableUserCleaner {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Context<Self>) {
        println!("Starting User Cleaner");
        ctx.run_interval(Duration::from_secs(300), move |this, _| {
            Arbiter::spawn(AvailableUserCleaner::clear_no_longuer_available_users(
                this.database_interface.clone(),
            ));
        });
    }
}

impl AvailableUserCleaner {
    pub fn new(database_interface: DataBaseInterface) -> Self {
        AvailableUserCleaner {
            database_interface: database_interface,
        }
    }
    pub async fn clear_no_longuer_available_users(database_interface: DataBaseInterface) {
        println!("Clearing database for availaible users");
        let now: DateTime<FixedOffset> = DateTime::from(Utc::now());
        let clear_res = database_interface.remove_available_until(now).await;
        match clear_res {
            Ok(n) => {
                println!("Removed {} users", n)
            }
            Err(err) => {
                println!("Error when clearing database : {}", err.message);
            }
        }
    }
}