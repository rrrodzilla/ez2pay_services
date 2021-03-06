#[macro_use]
extern crate log;
extern crate dotenv;
use actix_web::dev::Service;
use actix_web::http::header::CONTENT_TYPE;
use actix_web::http::HeaderValue;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use ez2paylib::{create_product, get_account, notify_info};
use serde::Deserialize;
use std::env;
#[derive(Deserialize)]
struct ImageMessage {
    #[serde(rename = "From")]
    from: String,
    #[serde(default, rename = "MediaUrl0")]
    media_url0: String,
    #[serde(rename = "To")]
    to: String,
}

async fn ingest_image(form: web::Form<ImageMessage>) -> impl Responder {
    info!("");
    info!("From: {}", form.from);
    info!("To: {}", form.to);
    let id = get_account(&form.from).await;

    if form.media_url0.len() > 0 {
        info!("Image: {}", &form.media_url0);
        create_product(&id, &form.media_url0).await;
        HttpResponse::Ok()
    } else {
        warn!("No image found");
        notify_info(&form.from).await;
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
            .wrap_fn(|req, srv| {
                //                info!("Hi from one. You requested: {}", req.path());
                let fut = srv.call(req);
                async {
                    let mut res = fut.await?;
                    res.headers_mut()
                        .insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));

                    //                   info!("Updated response one");
                    Ok(res)
                }
            })
            .wrap_fn(|req, srv| {
                //             info!("Hi from two. You passed: {:#?}", req);
                let fut = srv.call(req);
                async {
                    let mut res = fut.await?;
                    res.headers_mut()
                        .insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));

                    //                  info!("Updated response two");
                    Ok(res)
                }
            })
            .route("/input", web::post().to(ingest_image))
    })
    .bind(address.clone())?
    .run()
    .await
}
