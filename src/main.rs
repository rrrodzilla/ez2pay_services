#[macro_use]
extern crate log;
use ez2paylib::test;

//use actix_service::Service;
use actix_web::dev::Service;
use actix_web::http::header::CONTENT_TYPE;
use actix_web::http::HeaderValue;
use actix_web::{get, post, web, App, HttpMessage, HttpResponse, HttpServer, Responder, Result};
use clap::Clap;
//use futures::future::FutureExt;
use pickledb::{PickleDb, PickleDbDumpPolicy};
use serde::{Deserialize, Serialize};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/run")]
async fn run(account: web::Json<Account>) -> Result<String> {
    info!("executing run");
    Ok(format!("Welcome {}!", account.phone_number))
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "ez2pay=Info");
    env_logger::init();
    info!("Starting server...");
    test();
    HttpServer::new(|| {
        App::new()
            .wrap_fn(|req, srv| {
                info!("Hi from one. You requested: {}", req.path());
                let fut = srv.call(req);
                async {
                    let mut res = fut.await?;
                    res.headers_mut()
                        .insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));

                    info!("Updated response one");
                    Ok(res)
                }
            })
            .wrap_fn(|req, srv| {
                info!("Hi from two. You passed: {:#?}", req);
                let fut = srv.call(req);
                async {
                    let mut res = fut.await?;
                    res.headers_mut()
                        .insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));

                    info!("Updated response two");
                    Ok(res)
                }
            })
            .service(hello)
            .service(run)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

#[derive(Deserialize, Serialize, Debug)]
struct Account {
    phone_number: String,
}
