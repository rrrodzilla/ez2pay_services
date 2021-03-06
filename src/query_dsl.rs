pub mod query_dsl {
    use crate::types::types::*;
    cynic::query_dsl!(r#"src/graphql/schema.graphql"#);
}
