#[macro_use]
extern crate log;
extern crate dotenv;
extern crate otpauth;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use ez2paylib::{
    create_product, get_account, get_product, notify_auth_code, notify_info, verify_auth_code,
};
use harsh::Harsh;
use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
struct ImageMessage {
    #[serde(rename = "From")]
    from: String,
    #[serde(default, rename = "Body")]
    body: String,
    #[serde(default, rename = "MediaUrl0")]
    media_url0: String,
    #[serde(rename = "To")]
    to: String,
}
async fn auth_me(web::Path(code): web::Path<u32>) -> impl Responder {
    if verify_auth_code(code).await {
        HttpResponse::Ok()
    } else {
        HttpResponse::Unauthorized()
    }
}
async fn knock_knock(web::Path(id): web::Path<String>) -> impl Responder {
    notify_auth_code(&id).await;

    HttpResponse::Ok()
}
async fn manage_product(web::Path(id): web::Path<String>) -> HttpResponse {
    let harsh = Harsh::builder().salt("ez2pay").length(6).build().unwrap();
    let prod_id = harsh.decode_hex(&id).unwrap_or_default();
    if prod_id.len() > 0 {
        let prod = get_product(&prod_id).await;
        HttpResponse::Ok()
            .content_type("application/json")
            .body(prod)
    } else {
        warn!("Received bad product id: {}", &id);
        HttpResponse::Ok()
            .content_type("application/json")
            .body("{}".to_string())
    }
}

async fn ingest_image(form: web::Form<ImageMessage>) -> impl Responder {
    info!("");
    info!("From: {}", form.from);
    info!("To: {}", form.to);
    let (id, is_new_account) = get_account(&form.from).await;
    //probably not the greatest way to eliminate these characters
    //i should probs use regex and come up with some other cases
    //TODO: test thoroughly
    let price: i32 = match form
        .body
        .replace(",", "")
        .replace(".", "")
        .replace("$", "")
        .parse::<i32>()
    {
        Ok(i) => i,
        _ => 0,
    };
    if price > 0 {
        info!("Price: {}", price);
    }

    if form.media_url0.len() > 0 {
        info!("Image found...");
        let short_url = create_product(&id, &form.media_url0, price).await.unwrap();
        if is_new_account {
            notify_info(&form.from, &format!("Welcome!\nActivate your new product @ https://ez2pay.me/{}\nKeep this URL safe and don't share it!\nLearn more @ https://easy2pay.me",short_url)).await;
        //        info!("Activate your product at https://ez2pay.me/{}\nKeep this url safe and don't share it with anybody!",short_url);
        } else {
            notify_info(&form.from, &format!("Activate your new product @ https://ez2pay.me/{}\nKeep this URL safe and don't share it!\nLearn more @ https://easy2pay.me",short_url)).await;
        }
        HttpResponse::Ok()
    } else {
        warn!("No image found");
        notify_info(&form.from,  "Text a picture and price of what you want to sell.\nYou'll get a checkout page to share with your customers.\nSimple!\nLearn more @ https://easy2pay.me").await;
        HttpResponse::Ok()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //load the .env file
    dotenv().ok();
    //check for required vars
    let _: &str =
        &env::var("DB_AUTH_SECRET").unwrap_or_else(|_| panic!("DB_AUTH_SECRET must be set!"));

    //set env vars
    env::set_var(
        "RUST_LOG",
        format!(
            "ez2pay={}, ez2paylib={}",
            &env::var("RUST_LOG_EZ2PAY").unwrap_or_else(|_| "Info".into()),
            &env::var("RUST_LOG_EZ2PAYLIB").unwrap_or_else(|_| "Info".into())
        ),
    );
    let addr: &str = &env::var("ADDRESS").unwrap_or_else(|_| "127.0.0.1".into());
    let port: &str = &env::var("PORT").unwrap_or_else(|_| "8080".into());
    let address = format!("{}:{}", addr, port);

    //begin logging
    env_logger::init();
    //    let a = Account::get("+12063832022".into());
    info!("Server started and listening...");
    info!("{}", address);
    HttpServer::new(|| {
        App::new()
            .route("/input", web::post().to(ingest_image))
            .route("/{id}", web::get().to(manage_product))
            .route("/letmein/{id}", web::get().to(knock_knock))
            .route("/verifyme/{code}", web::get().to(auth_me))
    })
    .bind(address.clone())?
    .run()
    .await
}
