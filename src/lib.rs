#![allow(dead_code)]
#[macro_use]
extern crate log;
extern crate dotenv;
pub mod mutations;
pub mod queries;
pub mod query_dsl;
pub mod types;
use crate::mutations::accounts::create::{CreateAccountByPhone, CreateAccountByPhoneArguments};
use crate::mutations::products::create::{
    CreateProductForAccount, CreateProductForAccountArguments,
};
use crate::mutations::products::update::{UpdateProduct, UpdateProductArguments};
use crate::queries::accounts::{FindAccountByPhone, FindAccountByPhoneArguments};
use crate::queries::products::{FindProductById, FindProductByIdArguments};
use harsh::Harsh;
use otpauth::TOTP;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use twilio::OutboundMessage;

pub async fn verify_auth_code(code: u32) -> bool {
    let auth_secret: &str =
        &env::var("AUTH_SECRET").unwrap_or_else(|_| panic!("AUTH_SECRET must be set!"));
    let auth = TOTP::new(auth_secret);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    auth.verify(code, 120, timestamp)
}

pub async fn notify_auth_code(id: &str) {
    let product = get_product(&id).await.find_product_by_id.unwrap().account;

    let auth_secret: &str =
        &env::var("AUTH_SECRET").unwrap_or_else(|_| panic!("AUTH_SECRET must be set!"));
    let auth = TOTP::new(auth_secret);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let code = auth.generate(120, timestamp);
    notify_info(
        &product.phone_number.unwrap(),
        &format!(
            "EZ2PAY.ME: {} is your Product Page Verification Code. Valid for 2 minutes",
            code
        ),
    )
    .await;
}

pub async fn notify_info(phone_number: &str, message: &str) {
    //using test values if real values aren't set
    //these vars should all be moved to their own struct
    let sid: &str =
        &env::var("TWILIO_SID").unwrap_or_else(|_| "AC68e25593ac8571dc6b654cec468f67e7".into());
    let secret: &str =
        &env::var("TWILIO_SECRET").unwrap_or_else(|_| "ae037c08815fe4c48d83de8fb71af72b".into());
    let service_phone_number: &str =
        &env::var("EZPAY_PHONE_NUMBER").unwrap_or_else(|_| "+15005550006".into());
    let client = twilio::Client::new(sid, secret);
    match client
        .send_message(OutboundMessage::new(
            service_phone_number,
            phone_number,
            message,
        ))
        .await
    {
        Ok(_) => info!(
            "\n*****MESSAGE SENT*****\nSent to: {}\nMessage: {}",
            phone_number, message
        ),
        Err(_) => error!("Couldn't send info message"),
    };
}
pub async fn update_product(product: UpdateProductArguments) -> Result<(), &'static str> {
    use cynic::http::SurfExt;
    use cynic::MutationBuilder;

    let db_secret_key: &str =
        &env::var("DB_AUTH_SECRET").unwrap_or_else(|_| panic!("DB_AUTH_SECRET must be set!"));
    let graphql_endpoint: &str = &env::var("GRAPHQL_ENDPOINT")
        .unwrap_or_else(|_| "https://graphql.fauna.com/graphql".into());
    let operation = UpdateProduct::build(&product);
    let response = surf::post(graphql_endpoint)
        .header("authorization", format!("Basic {}", db_secret_key))
        .header("Accept-Encoding", "gzip, deflate, br")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Connection", "keep-alive")
        .header("DNT", "1")
        .run_graphql(operation)
        .await
        .unwrap()
        .data;
    match response {
        Some(_) => {
            Ok(())

            // here we generate a has for the id and send the management url to the user
        }
        None => {
            error!("Product couldn't be updated for some reason...");
            Err("Product couldn't be created for some reason...")
        }
    }
}
pub async fn create_product(
    id: &str,
    image: &str,
    price: i32,
) -> Result<(String, String), &'static str> {
    use cynic::http::SurfExt;
    use cynic::MutationBuilder;

    let mut default_price = 500;
    if price > 0 {
        default_price = price;
    }
    let db_secret_key: &str =
        &env::var("DB_AUTH_SECRET").unwrap_or_else(|_| panic!("DB_AUTH_SECRET must be set!"));
    let graphql_endpoint: &str = &env::var("GRAPHQL_ENDPOINT")
        .unwrap_or_else(|_| "https://graphql.fauna.com/graphql".into());
    let operation = CreateProductForAccount::build(&CreateProductForAccountArguments {
        connect: cynic::Id::from(id),
        image: image.to_string(),
        price: default_price,
    });
    let response = surf::post(graphql_endpoint)
        .header("authorization", format!("Basic {}", db_secret_key))
        .header("Accept-Encoding", "gzip, deflate, br")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Connection", "keep-alive")
        .header("DNT", "1")
        .run_graphql(operation)
        .await
        .unwrap()
        .data;
    match response {
        Some(a) => {
            let new_product_id = a.create_product.id.clone().into_inner();
            let harsh = Harsh::builder().salt("ez2pay").length(6).build().unwrap();
            let short_url = harsh.encode_hex(&new_product_id).unwrap();
            let cust_harsh = Harsh::builder()
                .salt("ez2pay_customer")
                .length(6)
                .build()
                .unwrap();
            let cust_url = cust_harsh.encode_hex(&new_product_id).unwrap();
            Ok((cust_url, short_url))

            // here we generate a has for the id and send the management url to the user
        }
        None => {
            error!("Product couldn't be created for some reason...");
            Err("Product couldn't be created for some reason...")
        }
    }
}
pub async fn get_product(product_id: &str) -> FindProductById {
    use cynic::http::SurfExt;
    use cynic::QueryBuilder;

    let db_secret_key: &str =
        &env::var("DB_AUTH_SECRET").unwrap_or_else(|_| panic!("DB_AUTH_SECRET must be set!"));
    let graphql_endpoint: &str = &env::var("GRAPHQL_ENDPOINT")
        .unwrap_or_else(|_| "https://graphql.fauna.com/graphql".into());
    let operation = FindProductById::build(&FindProductByIdArguments {
        id: cynic::Id::from(product_id),
    });
    let response = surf::post(graphql_endpoint)
        .header("authorization", format!("Basic {}", db_secret_key))
        .header("Accept-Encoding", "gzip, deflate, br")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Connection", "keep-alive")
        .header("DNT", "1")
        .run_graphql(operation)
        .await
        .unwrap()
        .data;
    response.unwrap()
    /*
    match response {
        Some(a) => {
            a.find_product_by_id.unwrap();
            //            serde_json::to_string(&prod).unwrap()
        }
        None => {
            warn!("No product was found for id: {}", product_id);
            "{}".to_string()
        }
    }
    */
}
//we'll want this to return an account id no matter what
//return an existing account or create one from the incoming phone number if it's not found and
//return the new id
pub async fn get_account(phone_number: &str) -> (String, bool) {
    use cynic::http::SurfExt;
    use cynic::QueryBuilder;

    let db_secret_key: &str =
        &env::var("DB_AUTH_SECRET").unwrap_or_else(|_| panic!("DB_AUTH_SECRET must be set!"));
    let graphql_endpoint: &str = &env::var("GRAPHQL_ENDPOINT")
        .unwrap_or_else(|_| "https://graphql.fauna.com/graphql".into());
    let operation = FindAccountByPhone::build(&FindAccountByPhoneArguments {
        phone_number: phone_number.to_string(),
    });
    let response = surf::post(graphql_endpoint)
        .header("authorization", format!("Basic {}", db_secret_key))
        .header("Accept-Encoding", "gzip, deflate, br")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Connection", "keep-alive")
        .header("DNT", "1")
        .run_graphql(operation)
        .await
        .unwrap()
        .data
        .unwrap()
        .find_account_by_phone;

    match response {
        Some(a) => {
            let account = a.clone();
            let existing_id = account.id.clone().inner().to_string();
            info!("Retrieved Existing Account: {:?}", existing_id);
            (existing_id, false)
        }
        None => {
            //if no account was found, we need to create one
            use cynic::MutationBuilder;
            let operation = CreateAccountByPhone::build(&CreateAccountByPhoneArguments {
                phone_number: phone_number.to_string(),
            });
            let response = surf::post(graphql_endpoint)
                .header("authorization", format!("Basic {}", db_secret_key))
                .header("Accept-Encoding", "gzip, deflate, br")
                .header("Content-Type", "application/json")
                .header("Accept", "application/json")
                .header("Connection", "keep-alive")
                .header("DNT", "1")
                .run_graphql(operation)
                .await
                .unwrap()
                .data
                .unwrap()
                .create_account;
            let new_id = response.id.inner().to_string();
            info!("Created New Account: {:?}", new_id);
            (new_id, true)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn new_account_by_phone_test() {
        use crate::queries::queries::{NewAccountByPhone, NewAccountByPhoneArguments};
        use cynic::MutationBuilder;

        let operation = NewAccountByPhone::build(NewAccountByPhoneArguments {
            phone_number: "+12063832022".to_string(),
        });
        insta::assert_snapshot!(operation.query);
    }
    #[test]
    fn find_account_by_phone_test() {
        use crate::queries::queries::{FindAccountByPhone, FindAccountByPhoneArguments};
        use cynic::QueryBuilder;

        let operation = FindAccountByPhone::build(FindAccountByPhoneArguments {
            phone_number: "+12063832022".to_string(),
        });
        insta::assert_snapshot!(operation.query);
    }
}
