#[cynic::query_module(
    schema_path = r#"src/graphql/schema.graphql"#,
    query_module = "query_dsl"
)]
pub mod products {
    #[cynic::query_module(
        schema_path = r#"src/graphql/schema.graphql"#,
        query_module = "query_dsl"
    )]
    pub mod create {

        use crate::query_dsl::*;

        #[derive(cynic::FragmentArguments, Debug)]
        pub struct CreateProductForAccountArguments {
            pub connect: cynic::Id,
            pub image: String,
        }

        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(
            graphql_type = "Mutation",
            argument_struct = "CreateProductForAccountArguments"
        )]
        pub struct CreateProductForAccount {
            #[arguments(data = ProductInput { account: Some(ProductAccountRelation { connect: Some(args.connect.clone()) }), image: Some(args.image.clone()), status: ProductStatus::New })]
            pub create_product: Product,
        }

        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(graphql_type = "Product")]
        pub struct Product {
            pub id: cynic::Id,
        }

        #[derive(cynic::Enum, Clone, Copy, Debug)]
        #[cynic(graphql_type = "ProductStatus")]
        pub enum ProductStatus {
            Published,
            Disabled,
            New,
        }

        #[derive(cynic::InputObject, Debug)]
        #[cynic(graphql_type = "ProductInput")]
        pub struct ProductInput {
            pub account: Option<ProductAccountRelation>,
            pub image: Option<String>,
            pub status: ProductStatus,
        }

        #[derive(cynic::InputObject, Debug)]
        #[cynic(graphql_type = "ProductAccountRelation")]
        pub struct ProductAccountRelation {
            pub connect: Option<cynic::Id>,
        }
    }
}
#[cynic::query_module(
    schema_path = r#"src/graphql/schema.graphql"#,
    query_module = "query_dsl"
)]
pub mod accounts {
    #[cynic::query_module(
        schema_path = r#"src/graphql/schema.graphql"#,
        query_module = "query_dsl"
    )]
    pub mod create {
        use crate::query_dsl::*;

        #[derive(cynic::FragmentArguments, Debug)]
        pub struct CreateAccountByPhoneArguments {
            pub phone_number: String,
        }

        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(
            graphql_type = "Mutation",
            argument_struct = "CreateAccountByPhoneArguments"
        )]
        pub struct CreateAccountByPhone {
            #[arguments(data = AccountInput { phone_number: Some(args.phone_number.clone()) })]
            pub create_account: Account,
        }

        #[derive(cynic::QueryFragment, Debug, Clone)]
        #[cynic(graphql_type = "Account")]
        pub struct Account {
            pub id: cynic::Id,
        }

        #[derive(cynic::InputObject, Debug)]
        #[cynic(graphql_type = "AccountInput")]
        pub struct AccountInput {
            #[cynic(rename = "phone_number")]
            pub phone_number: Option<String>,
        }
    }
    #[cynic::query_module(
        schema_path = r#"src/graphql/schema.graphql"#,
        query_module = "query_dsl"
    )]
    pub mod update {

        use crate::query_dsl::*;

        #[derive(cynic::FragmentArguments, Debug)]
        pub struct UpdateAccountByPhoneArguments {
            pub phone_number: String,
            pub stripe_id: String,
            pub id: cynic::Id,
        }

        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(
            graphql_type = "Mutation",
            argument_struct = "UpdateAccountByPhoneArguments"
        )]
        pub struct UpdateAccountByPhone {
            #[arguments(data = AccountInput { phone_number: Some(args.phone_number.clone()), stripe_id: Some(args.stripe_id.clone()) }, id = args.id.clone())]
            pub update_account: Option<Account>,
        }

        #[derive(cynic::QueryFragment, Debug, Clone)]
        #[cynic(graphql_type = "Account")]
        pub struct Account {
            pub stripe_id: Option<String>,
        }

        #[derive(cynic::InputObject, Debug)]
        #[cynic(graphql_type = "AccountInput")]
        pub struct AccountInput {
            #[cynic(rename = "phone_number")]
            pub phone_number: Option<String>,
            #[cynic(rename = "stripe_id")]
            pub stripe_id: Option<String>,
        }
    }
}
