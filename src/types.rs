#[cynic::query_module(
    schema_path = r#"src/graphql/schema.graphql"#,
    query_module = "query_dsl"
)]
pub mod types {
    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct Date(pub String);

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct Time(pub String);

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct Long(pub String);
}
