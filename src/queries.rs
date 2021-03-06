#[cynic::query_module(
    schema_path = r#"src/graphql/schema.graphql"#,
    query_module = "query_dsl"
)]
pub mod accounts {

    use crate::query_dsl::*;

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

    #[derive(cynic::QueryFragment, Debug, Clone)]
    #[cynic(graphql_type = "Account")]
    pub struct Account {
        pub id: cynic::Id,
        pub phone_number: Option<String>,
        pub stripe_id: Option<String>,
    }
}
#[cynic::query_module(
    schema_path = r#"src/graphql/schema.graphql"#,
    query_module = "query_dsl"
)]
pub mod products {

    use crate::queries::accounts::*;
    use crate::query_dsl::*;

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

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Product")]
    pub struct Product {
        pub account: Account,
        pub description: Option<String>,
        pub image: Option<String>,
        pub name: Option<String>,
        pub price: Option<i32>,
        pub status: ProductStatus,
        pub tax: Option<i32>,
    }

    #[derive(cynic::Enum, Clone, Copy, Debug)]
    #[cynic(graphql_type = "ProductStatus")]
    pub enum ProductStatus {
        Published,
        Disabled,
        New,
    }
}
