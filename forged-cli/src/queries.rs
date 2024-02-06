use cynic::impl_scalar;
use uuid::Uuid;

pub use queries::*;

#[cynic::schema_for_derives(file = "schema.graphql", module = "schema")]
pub mod queries {
    use super::schema;
    use forged::cynic;
    use uuid::Uuid;

    cynic::impl_scalar!(serde_json::Value, schema::JSON);

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
    #[cynic(graphql_type = "MutationRoot")]
    pub struct FinishRun {
        pub run_finish: Uuid,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "MutationRoot", variables = "CreateLogArguments")]
    pub struct CreateLog {
        #[arguments(level: $level, message: $message)]
        pub log_create: Log,
    }

    #[derive(cynic::QueryVariables, Debug)]
    pub struct CreateLogArguments {
        pub level: String,
        pub message: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Log {
        pub id: Uuid,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Chip")]
    pub struct Chip {
        pub id: Uuid,
        pub name: String,
        pub part_number: String,
        pub binaries: Vec<Binary>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "QueryRoot")]
    pub struct Chips {
        pub current_provisioner: ProvisionerProjectChips,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Project")]
    pub struct ProjectChips {
        pub chips: Vec<Chip>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Provisioner")]
    pub struct ProvisionerProjectChips {
        pub project: ProjectChips,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Binary {
        pub id: Uuid,
        pub version_major: i32,
        pub version_minor: i32,
        pub version_patch: i32,
        pub parts: Vec<BinaryPart>,
    }

    impl Binary {
        pub fn version(&self) -> semver::Version {
            semver::Version {
                major: self.version_major as u64,
                minor: self.version_minor as u64,
                patch: self.version_patch as u64,
                pre: Default::default(),
                build: Default::default(),
            }
        }
    }

    #[derive(cynic::QueryFragment, Clone, Debug)]
    pub struct BinaryPart {
        pub id: Uuid,
        pub kind: BinaryKind,
        pub memory_offset: Option<i32>,
        pub analysis: Option<BinaryPartAnalysis>,
    }

    #[derive(cynic::QueryFragment, Debug, Clone)]
    pub struct BinaryAnalysis {
        pub nvm_size: i32,
    }

    #[derive(cynic::QueryFragment, Debug, Clone)]
    pub struct BinaryPartAnalysis {
        pub nvm_size: i32,
    }

    #[derive(cynic::Enum, Debug, Clone)]
    pub enum BinaryKind {
        Elf,
        Bin,
        Hex,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "MutationRoot", variables = "CreateAttachmentArguments")]
    pub struct CreateAttachment {
        #[arguments(data: $data)]
        pub attachment_create: Attachment,
    }

    #[derive(cynic::QueryVariables, Debug)]
    pub struct CreateAttachmentArguments {
        pub data: forged::Upload,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Attachment {
        pub id: Uuid,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "MutationRoot", variables = "CreateBlockArguments")]
    pub struct CreateBlock {
        #[arguments(schemaName: $schema_name, data: $data)]
        pub block_create: Block,
    }

    #[derive(cynic::QueryVariables, Debug)]
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
impl_scalar!(Uuid, schema::UUID);
