use crate::database::database_interface::DataBaseInterface;
use crate::models::user;
use actix_web::{
    error::{Error, ErrorInternalServerError},
    web, HttpResponse, Result,
};

pub async fn user_available(
    database: web::Data<DataBaseInterface>,
    user: web::Json<user::User>,
) -> Result<HttpResponse, Error> {
    println!(
        "User Phone : {:0}, available until : {:1}",
        user.phone_number_hash, user.available_until
    );
    database
        .set_user_available(&user)
        .await
        .map_err(|err| ErrorInternalServerError(err.message))?;
    return Ok(HttpResponse::Ok().finish());
}

pub async fn get_nearby_friends(
    database: web::Data<DataBaseInterface>,
    user: web::Json<user::User>,
) -> Result<HttpResponse, Error> {
    println!(
        "User Phone : {:0}, available until : {:1}",
        user.phone_number_hash, user.available_until
    );
    let available_contacts = database
        .get_contacts_available_nearby(
            &user.phone_number_hash,
            user.latitude,
            user.longitude,
            10_000f32, // TODO : Expose that to the API !
        )
        .await
        .map_err(|err| ErrorInternalServerError(err.message))?;

    return Ok(HttpResponse::Ok().json(available_contacts));
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http, test, App};
    use chrono::DateTime;
    use std::string::String;
    use user::LocalizedUser;

    #[actix_rt::test]
    async fn test_full_scenario() {
        let database_interface = DataBaseInterface::new().await.unwrap();
        database_interface.clear_database().await.unwrap();
        /*
         * Given :
         *
         *         Available Until       Friend Of      Location
         *  Peppa   22h00          Rebecca, Suzy, Pedro   43.0,6.0
         *  Rebecca 21h00          Peppa                  43.0,6.0
         *  Suzy    21h00          Peppa                  44,5  // Too far !!
         *  Pedro   21h00          Suzy                   43.0,6.0
         *
         *  When : Peppa ask for friends nearby,
         *  Then, it should return only Rebecca (because Pedro don't want to see Peppa)
         *
         *  When : We remove user no longer available at 21h30
         *         And Peppa ask for frien nearby
         *
         *  Then it shoud return nothing.
         */
        let mut app = test::init_service(
            App::new()
                .data(database_interface.clone())
                .route("/user_available", web::post().to(user_available))
                .route(
                    "/contacts_availables_nearby",
                    web::get().to(get_nearby_friends),
                ),
        )
        .await;

        let nine_pm =
            DateTime::parse_from_rfc3339("2021-05-21T21:00:00+00:00").expect("Can't parse date");
        let rebecca = user::User {
            phone_number_hash: String::from("Rebecca"),
            latitude: 43.0000000,
            longitude: 6.000000,
            available_until: nine_pm,
            contacts_phone_number_hash: vec![String::from("Peppa")],
        };

        let req = test::TestRequest::post()
            .header("content-type", "application/json")
            .uri("/user_available")
            .set_json(&rebecca)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let suzy = user::User {
            phone_number_hash: String::from("Suzy"),
            latitude: 44.0,
            longitude: 5.0,
            available_until: nine_pm,
            contacts_phone_number_hash: vec![String::from("Peppa")],
        };

        let req = test::TestRequest::post()
            .header("content-type", "application/json")
            .uri("/user_available")
            .set_json(&suzy)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let pedro = user::User {
            phone_number_hash: String::from("Peppa"),
            latitude: 43.0,
            longitude: 6.0,
            available_until: nine_pm,
            contacts_phone_number_hash: vec![String::from("Suzy")],
        };

        let req = test::TestRequest::post()
            .header("content-type", "application/json")
            .uri("/user_available")
            .set_json(&pedro)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let peppa = user::User {
            phone_number_hash: String::from("Peppa"),
            latitude: 43.0000000,
            longitude: 6.000000,
            available_until: DateTime::parse_from_rfc3339("2021-05-21T22:00:00+00:00")
                .expect("Can't parse date"),
            contacts_phone_number_hash: vec![
                String::from("Suzy"),
                String::from("Rebecca"),
                String::from("Pedro"),
            ],
        };
        let req = test::TestRequest::get()
            .header("content-type", "application/json")
            .uri("/contacts_availables_nearby")
            .set_json(&peppa)
            .to_request();

        let resp: Vec<LocalizedUser> = test::read_response_json(&mut app, req).await;
        assert_eq!(resp.len(), 1);
        assert_eq!(
            resp.get(0)
                .expect("Not enough nearby friends")
                .phone_number_hash,
            "Rebecca"
        );

        let deleted = database_interface
            .remove_available_until(
                DateTime::parse_from_rfc3339("2021-05-21T21:30:00+00:00")
                    .expect("Can't parse date"),
            )
            .await
            .expect("Can't remove users");
        assert_eq!(deleted, 3);

        let req = test::TestRequest::get()
            .header("content-type", "application/json")
            .uri("/contacts_availables_nearby")
            .set_json(&peppa)
            .to_request();
        let resp: Vec<LocalizedUser> = test::read_response_json(&mut app, req).await;
        assert_eq!(resp.len(), 0);
    }
}
