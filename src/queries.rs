#[cynic::query_module(
    schema_path = r#"src/graphql/schema.graphql"#,
    query_module = "query_dsl"
)]
pub mod accounts {

    use crate::query_dsl::*;
    use serde::{Deserialize, Serialize};

    #[derive(cynic::FragmentArguments, Debug)]
    pub struct FindAccountByPhoneArguments {
        pub phone_number: String,
    }

    #[derive(cynic::FragmentArguments, Debug)]
    pub struct FindAccountByIdArguments {
        pub id: cynic::Id,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(
        graphql_type = "Query",
        argument_struct = "FindAccountByPhoneArguments"
    )]
    pub struct FindAccountByPhone {
        #[arguments(phone_number = &args.phone_number)]
        pub find_account_by_phone: Option<Account>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Query", argument_struct = "FindAccountByIdArguments")]
    pub struct FindAccountById {
        #[arguments(id = args.id.clone())]
        pub find_account_by_id: Option<Account>,
    }

    #[derive(cynic::QueryFragment, Debug, Clone, Serialize, Deserialize)]
    #[cynic(graphql_type = "Account")]
    pub struct Account {
        #[serde(skip, default = "default_id")]
        pub id: cynic::Id,
        pub phone_number: Option<String>,
        pub stripe_id: Option<String>,
    }
    fn default_id() -> cynic::Id {
        cynic::Id::from("")
    }
}
#[cynic::query_module(
    schema_path = r#"src/graphql/schema.graphql"#,
    query_module = "query_dsl"
)]
pub mod products {
    use crate::queries::accounts::*;
    use crate::query_dsl::*;
    use serde::{Deserialize, Serialize};

    #[derive(cynic::FragmentArguments, Debug)]
    pub struct FindProductByIdArguments {
        pub id: cynic::Id,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Query", argument_struct = "FindProductByIdArguments")]
    pub struct FindProductById {
        #[arguments(id = args.id.clone())]
        pub find_product_by_id: Option<Product>,
    }

    #[derive(cynic::QueryFragment, Debug, Deserialize, Serialize)]
    #[cynic(graphql_type = "Product")]
    pub struct Product {
        pub account: Account,
        pub description: Option<String>,
        pub image: String,
        pub name: Option<String>,
        pub price: i32,
        pub status: ProductStatus,
        pub tax: Option<i32>,
    }

    #[derive(cynic::Enum, Clone, Copy, Debug, Serialize, Deserialize)]
    #[cynic(graphql_type = "ProductStatus")]
    pub enum ProductStatus {
        Published,
        Disabled,
        New,
    }
}
