use cynic::impl_scalar;
use uuid::Uuid;

pub use queries::*;

#[cynic::schema_for_derives(file = "schema.graphql", module = "schema")]
pub mod queries {
    use forged::cynic;
    use super::schema;
    use uuid::Uuid;

    cynic::impl_scalar!(serde_json::Value, schema::Json);

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "MutationRoot")]
    pub struct CreateDevice {
        pub device_create: Device,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Device {
        pub id: Uuid,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(
        graphql_type = "MutationRoot",
        argument_struct = "FinishDeviceArguments"
    )]
    pub struct FinishRun {
        #[arguments()]
        pub run_finish: Uuid,
    }

    #[derive(cynic::FragmentArguments, Debug)]
    pub struct FinishDeviceArguments {}

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "MutationRoot", argument_struct = "CreateLogArguments")]
    pub struct CreateLog {
        #[arguments(level=&args.level, message=&args.message)]
        pub log_create: Log,
    }

    #[derive(cynic::FragmentArguments, Debug)]
    pub struct CreateLogArguments {
        pub level: String,
        pub message: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Log {
        pub id: Uuid,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Provisioner")]
    pub struct ProvisionerBinary {
        pub project: ProjectBinary,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Chip")]
    pub struct Chip {
        pub name: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Project")]
    pub struct ProjectBinary {
        pub binary_newest: Option<Binary>,
        pub chip: Chip,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "QueryRoot")]
    pub struct BinaryNewest {
        pub current_provisioner: ProvisionerBinary,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Binary {
        pub id: Uuid,
        pub parts: Vec<BinaryPart>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct BinaryPart {
        pub id: Uuid,
        pub kind: BinaryKind,
        pub memory_offset: Option<i32>,
        pub image: Vec<i32>,
    }

    #[derive(cynic::Enum, Debug, Clone)]
    pub enum BinaryKind {
        Elf,
        Bin,
        Hex,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(
        graphql_type = "MutationRoot",
        argument_struct = "CreateAttachmentArguments"
    )]
    pub struct CreateAttachment {
        #[arguments(data=&args.data)]
        pub attachment_create: Attachment,
    }

    #[derive(cynic::FragmentArguments, Debug)]
    pub struct CreateAttachmentArguments {
        pub data: forged::Upload,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Attachment {
        pub id: Uuid,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(
        graphql_type = "MutationRoot",
        argument_struct = "CreateBlockArguments"
    )]
    pub struct CreateBlock {
        #[arguments(schema_name=&args.schema_name, data=&args.data)]
        pub block_create: Block,
    }

    #[derive(cynic::FragmentArguments, Debug)]
    pub struct CreateBlockArguments {
        pub schema_name: String,
        pub data: serde_json::Value,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Block {
        pub id: Uuid,
    }
}

mod schema {
    cynic::use_schema!("schema.graphql");
}

impl_scalar!(forged::Upload, schema::Upload);
impl_scalar!(Uuid, schema::Uuid);
