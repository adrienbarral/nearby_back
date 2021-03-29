use crate::models::user;
use actix_web::{http::Error, web, HttpResponse, Result};
use crate::database::database_interface::DataBaseInterface;

pub async fn user_available(user: web::Json<user::User>) -> Result<HttpResponse, Error> {
    println!(
        "User Phone : {:0}, available until : {:1}",
        user.phone_number_hash, user.available_until
    );
    return Ok(HttpResponse::Ok().json(""));
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http, test, App};
    use chrono::DateTime;
    #[actix_rt::test]
    async fn test_index_ok() {
        let mut app = test::init_service(
            App::new()
                .service(web::resource("/user_available").route(web::post().to(user_available))),
        )
        .await;
        let user = user::User {
            phone_number_hash: String::from("15645612"),
            latitude: 43.2255228,
            longitude: 6.3516515645,
            available_until: DateTime::parse_from_rfc3339("2021-05-21T18:21:43+00:00")
                .expect("Can't parse date"),
            contacts_phone_number_hash: vec![]
        };

        let req = test::TestRequest::post()
            .header("content-type", "application/json")
            .uri("/user_available")
            .set_json(&user)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }
}
