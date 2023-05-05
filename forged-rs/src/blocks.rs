pub use queries::*;

#[cynic::schema_for_derives(file = "schema.graphql", module = "schema")]
pub mod queries {
    use super::schema;

    cynic::impl_scalar!(serde_json::Value, schema::JSON);

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "QueryRoot")]
    pub struct QueryBlocks {
        pub current_provisioner: Provisioner,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Provisioner")]
    pub struct Provisioner {
        pub current_run: Option<Run>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Run")]
    pub struct Run {
        pub blocks: Vec<Block>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Block")]
    pub struct Block {
        pub data_decoded: serde_json::Value,
        pub schema: BlockSchema,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "BlockSchema")]
    pub struct BlockSchema {
        pub name: String,
    }
}

mod schema {
    cynic::use_schema!("schema.graphql");
}
