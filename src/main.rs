#[macro_use]
extern crate log;
extern crate dotenv;
extern crate otpauth;
use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use ez2paylib::mutations::products::update::UpdateProductArguments;
use ez2paylib::{
    create_product, get_account, get_product, notify_auth_code, notify_info, update_product,
    verify_auth_code,
};
use futures::future;
use harsh::Harsh;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use stripe::{
    CheckoutSessionMode, CreateCheckoutSession, CreateCheckoutSessionLineItems,
    CreateCheckoutSessionLineItemsPriceData, CreateCheckoutSessionLineItemsPriceDataProductData,
    CreateCheckoutSessionPaymentMethodTypes, Currency,
};

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
    let harsh = Harsh::builder().salt("ez2pay").length(6).build().unwrap();
    let prod_id = harsh.decode_hex(&id).unwrap_or_default();
    notify_auth_code(&prod_id).await;

    HttpResponse::Ok()
}
async fn stage_checkout(web::Path(id): web::Path<String>) -> impl Responder {
    let harsh = Harsh::builder()
        .salt("ez2pay_customer")
        .length(6)
        .build()
        .unwrap();
    let prod_id = harsh.decode_hex(&id).unwrap_or_default();
    if prod_id.len() > 0 {
        let prod = get_product(&prod_id).await.find_product_by_id.unwrap();

        let stripe_secret: &str =
            &env::var("STRIPE_STAGING").unwrap_or_else(|_| panic!("STRIPE_STAGING must be set!!!"));
        let mut checkout_session_params = CreateCheckoutSession::new(
            "https://localhost",
            vec![CreateCheckoutSessionPaymentMethodTypes::Card],
            "https://localhost/success",
        );

        //first let's fix the product url
        //having to make a extra http request to get the image isn't amazing
        //but it's required to get to the actual image to submit to stripe for the checkout page
        //the alternative is to set up a service that will copy the image from twilio to another
        //cdn for direct access.  this is a todo for the future since twilio only keeps media for
        //a year, but for now, nopes

        trace!("about to attempt to retrieve image from {}", prod.image);
        let img_response = surf::get(prod.image).await.unwrap();
        let img = img_response.header("location").unwrap().get(0).unwrap();

        //set up some price data for this checkout session using the product image we retrieved
        let price_data = CreateCheckoutSessionLineItemsPriceData {
            currency: Currency::USD,
            unit_amount: Some(prod.price.into()),
            product_data: Some(CreateCheckoutSessionLineItemsPriceDataProductData {
                description: Some(
                    prod.description
                        .unwrap_or_else(|| "Thanks for your support!".to_string()),
                ),
                images: Some(vec![img.to_string()]),
                name: prod.name.unwrap_or_else(|| "Buy me!".to_string()),
                metadata: HashMap::new(),
            }),
            unit_amount_decimal: Option::None,
            recurring: Option::None,
            product: Option::None,
        };

        /*
        let adjustable_quantity = CreateCheckoutSessionLineItemsAdjustableQuantity {
            enabled: true,
            maximum: Some(99),
            minimum: Some(1),
        };
        */

        let line_item = CreateCheckoutSessionLineItems {
            price_data: Some(price_data),
            quantity: Some(1),
            amount: Option::None,
            currency: Option::None,
            //adjustable_quantity: Some(adjustable_quantity),
            adjustable_quantity: Option::None,
            dynamic_tax_rates: Option::None,
            price: Option::None,
            tax_rates: Option::None,
            description: Option::None,
            name: Option::None,
            images: Option::None,
        };

        checkout_session_params.line_items = Some(vec![line_item]);
        checkout_session_params.mode = Some(CheckoutSessionMode::Payment);

        let client = stripe::Client::new(stripe_secret);
        let checkout_session = stripe::CheckoutSession::create(&client, checkout_session_params)
            .await
            .unwrap();
        info!("{}", checkout_session.id);

        HttpResponse::Ok().content_type("text/html").body(format!(
            "<html><head><title>loading your product...</title><script src='https://js.stripe.com/v3/'></script><script type='text/javascript'>var stripe = Stripe('pk_test_wR7xgNYdB8FGgjBLmrDdiWyZ');document.onload = stripe.redirectToCheckout({{ sessionId:'{}' }});</script></head><body></body></html>", checkout_session.id),
        )
    } else {
        warn!("Received bad product id: {}", &id);
        HttpResponse::Ok()
            .content_type("application/json")
            .body("{}".to_string())
    }
}

async fn manage_product(web::Path(id): web::Path<String>) -> HttpResponse {
    let harsh = Harsh::builder().salt("ez2pay").length(6).build().unwrap();
    let prod_id = harsh.decode_hex(&id).unwrap_or_default();
    if prod_id.len() > 0 {
        //            serde_json::to_string(&prod).unwrap()
        let prod = get_product(&prod_id).await;
        let res = serde_json::to_string(&prod).unwrap();
        HttpResponse::Ok()
            .content_type("application/json")
            .body(res)
    } else {
        warn!("Received bad product id: {}", &id);
        HttpResponse::Ok()
            .content_type("application/json")
            .body("{}".to_string())
    }
}

async fn handle_update_product(
    web::Path(id): web::Path<String>,
    mut product: web::Form<UpdateProductArguments>,
) -> impl Responder {
    product.id = cynic::Id::from(id);
    match update_product(product.into_inner()).await {
        Ok(_) => HttpResponse::Ok(),
        Err(s) => {
            error!("Couldn't update product: {}", s);
            HttpResponse::BadRequest()
        }
    }
}
async fn ingest_image(form: web::Form<ImageMessage>) -> impl Responder {
    info!("");
    info!("From: {}", form.from);
    info!("To: {}", form.to);
    let customer_host: &str =
        &env::var("CUSTOMER_HOST").unwrap_or_else(|_| panic!("CUSTOMER_HOST NOT SET"));
    let mgmt_host: &str = &env::var("MGMT_HOST").unwrap_or_else(|_| panic!("MGMT_HOST NOT SET"));

    let (id, _) = get_account(&form.from).await;
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
        let (cust_url, short_url) = create_product(&id, &form.media_url0, price).await.unwrap();
        notify_info(&form.from, &format!("Visit your checkout page @ {}/{}\nManage your product @ {}/{}\nKeep this URL safe and don't share it!",customer_host, cust_url, mgmt_host, short_url)).await;
        HttpResponse::Ok()
    } else {
        warn!("No image found");
        notify_info(&form.from,  &format!("Text a picture and price of what you want to sell.\nYou'll get a checkout page to share with your customers.\nSimple!\nLearn more @ {}", mgmt_host)).await;
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
    let addr2: &str = &env::var("ADDRESS2").unwrap_or_else(|_| "127.0.0.1".into());
    let port2: &str = &env::var("PORT2").unwrap_or_else(|_| "8181".into());
    let address2 = format!("{}:{}", addr2, port2);

    //begin logging
    env_logger::init();
    //    let a = Account::get("+12063832022".into());
    info!("Servers started and listening...");
    info!("{}", address);
    info!("{}", address2);
    let api_server = HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("https://ez2payu.com")
            .allowed_methods(vec!["GET", "POST"]);
        App::new()
            .wrap(cors)
            .route("/input", web::post().to(ingest_image))
            .route("/{id}", web::get().to(manage_product))
            .route("/update/{id}", web::put().to(handle_update_product))
            .route("/letmein/{id}", web::get().to(knock_knock))
            .route("/verifyme/{code}", web::get().to(auth_me))
    })
    .bind(address.clone())?
    .run();
    let customer_server =
        HttpServer::new(|| App::new().route("/{id}", web::get().to(stage_checkout)))
            .bind(address2.clone())?
            .run();
    future::try_join(api_server, customer_server).await?;
    Ok(())
}
