#![allow(dead_code, unused_imports)]
#[macro_use]
extern crate log;
extern crate dotenv;
mod mutations;
mod queries;
mod query_dsl;
mod types;
use crate::mutations::accounts::create::{CreateAccountByPhone, CreateAccountByPhoneArguments};
use crate::queries::accounts::{FindAccountByPhone, FindAccountByPhoneArguments};
use std::env;
use twilio::{Client, OutboundMessage};

pub async fn notify_info(phone_number: &str) {
    //using test values if real values aren't set
    //these vars should all be moved to their own struct
    let sid: &str =
        &env::var("TWILIO_SID").unwrap_or_else(|_| "AC68e25593ac8571dc6b654cec468f67e7".into());
    let secret: &str =
        &env::var("TWILIO_SECRET").unwrap_or_else(|_| "ae037c08815fe4c48d83de8fb71af72b".into());
    let service_phone_number: &str =
        &env::var("EZPAY_PHONE_NUMBER").unwrap_or_else(|_| "+15005550006".into());
    let client = twilio::Client::new(sid, secret);
    match client.send_message(OutboundMessage::new(service_phone_number,phone_number , "To sell a product, text a picture of what you're selling.  Learn more at: https://ez2pay.me")).await {
        Ok(_) => info!("Sent info message to {}", phone_number),
        Err(_) => error!("Couldn't send info message")
    };
}

//we'll want this to return an account id no matter what
//return an existing account or create one from the incoming phone number if it's not found and
//return the new id
pub async fn get_account(phone_number: &str) -> String {
    use cynic::http::SurfExt;
    use cynic::QueryBuilder;

    let db_secret_key: &str =
        &env::var("DB_AUTH_SECRET").unwrap_or_else(|_| panic!("DB_AUTH_SECRET must be set!"));
    let graphql_endpoint: &str = &env::var("GRAPHQL_ENDPOINT")
        .unwrap_or_else(|_| "https://graphql.fauna.com/graphql".into());
    let operation = FindAccountByPhone::build(&FindAccountByPhoneArguments {
        phone_number: phone_number.to_string(),
    });
    //Zm5BRURXRTdoQ0FDQWtRS1cyLXItLTU5MlBnc1hqTDRkTkNfdTJ6VzplenBheTpzZXJ2ZXI=
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
            existing_id
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
            new_id
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
