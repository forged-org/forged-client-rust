pub use cynic;

mod blocks;
mod chips;

use std::{collections::HashMap, future::Future, pin::Pin};

use cynic::{http::CynicReqwestError, GraphQlError, GraphQlResponse, Operation, QueryBuilder};
use regex::Regex;
use reqwest::multipart;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// The default endpoint for the forged.dev API.
const DEFAULT_API_URL: &str = "https://api.forged.dev";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("At least one error was returned from the GraphQL server")]
    Graphql(#[from] GraphQlError<serde::de::IgnoredAny>),
    #[error("An error with cynic occured")]
    Cynic(#[from] CynicReqwestError),

    #[error("An HTTP Request error occured")]
    Reqwest(#[from] reqwest::Error),

    #[error("An API error occurred")]
    Api(#[from] anyhow::Error),
}

/// A client to interact with the forged.dev API.
pub struct Client {
    token: String,
    instance_url: String,
    cache_folder: Option<std::path::PathBuf>,
}

impl Default for Client {
    fn default() -> Self {
        // TODO: Handle non-existent home directory
        let mut cache_folder = home::home_dir().map(|mut dir| {
            dir.push(".forged");
            dir
        });

        if let Some(folder) = cache_folder.clone() {
            if let Err(e) = std::fs::create_dir_all(folder) {
                log::warn!("Failed to create cache folder. Caching disabled: {e}");
                cache_folder = None
            }
        }

        Self {
            token: std::env::var("FORGED_API_TOKEN").unwrap_or_else(|_| {
                log::info!("No Forged API token found");
                "".to_string()
            }),
            instance_url: std::env::var("FORGED_API_URL")
                .unwrap_or_else(|_| DEFAULT_API_URL.to_string()),

            cache_folder,
        }
    }
}

impl Client {
    /// Create a client to the forged.dev API.
    ///
    /// # Args
    /// * `token` - The provisioner token to use to authenticate with forged.dev
    pub fn new(token: String) -> Self {
        Self {
            token,
            instance_url: DEFAULT_API_URL.to_string(),
            ..Default::default()
        }
    }

    /// Create a client to the forged.dev API.
    pub fn token(self, token: String) -> Self {
        Self { token, ..self }
    }

    /// Specify a custom API endpoint for the client
    ///
    /// # Args
    /// * `instance_url` - The HTTP URL of the forged API endpoint.
    ///
    /// # Note
    /// This is intended only to be used for development or locally-hosted forged.dev
    pub fn api(self, instance_url: String) -> Self {
        Self {
            instance_url,
            ..self
        }
    }

    /// Execute a query against the forged API.
    ///
    /// # Args
    /// * `operation` - The operation to execute.
    ///
    /// # Returns
    /// A GraphQL object representing the result of the executed query.
    pub async fn run_query<T, V>(&self, operation: Operation<T, V>) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned + 'static,
        V: Serialize,
    {
        let r = make_graphql_request(
            reqwest::Client::new()
                .post(&self.instance_url)
                .bearer_auth(&self.token)
                .json(&operation),
        )
        .await?;

        if let Some(errors) = r.errors {
            Err(errors[0].clone().into())
        } else if let Some(data) = r.data {
            Ok(data)
        } else {
            unreachable!()
        }
    }

    /// Get all of the uploaded blocks for the current run.
    ///
    /// # Returns
    /// A map of the blocks that currently exist for the run.
    pub async fn blocks(&self) -> Result<HashMap<String, serde_json::Value>, Error> {
        let result = self.run_query(blocks::QueryBlocks::build(())).await?;

        let Some(run) = result.current_provisioner.current_run else {
            return Ok(HashMap::default());
        };

        Ok(HashMap::from_iter(run.blocks.into_iter().map(|block| {
            (
                block.schema.name,
                block
                    .data_decoded
                    .get("value")
                    .or(block.data_decoded.get("values"))
                    .unwrap()
                    .to_owned(),
            )
        })))
    }

    /// Execute a query with a file upload to the forged API.
    ///
    /// # Args
    /// * `operation` - The operation to execute.
    /// * `files` - A list of files to upload to the API alongside the query.
    ///
    /// # Returns
    /// A GraphQL object representing the result of the executed query.
    pub async fn run_query_with_file_upload<T, V>(
        &self,
        operation: Operation<T, V>,
        files: Vec<Upload>,
    ) -> Result<T, Error>
    where
        T: DeserializeOwned,
        V: Serialize,
    {
        let mut files_map = HashMap::new();

        let re = Regex::new(r"\$(\w+): Upload").unwrap();
        for cap in re.captures_iter(&operation.query) {
            files_map.insert(files_map.len(), vec![format!("variables.{}", &cap[1])]);
        }

        let mut form = multipart::Form::new()
            // Adding just a simple text field...
            .text(
                "operations",
                serde_json::to_string(&operation)
                    .expect("Serializing this should always work. Please report this as a bug."),
            );

        for (i, file) in files.into_iter().enumerate() {
            let name = i.to_string();
            let part = multipart::Part::bytes(file.content).file_name(file.name);
            form = form.part(name, part);
        }

        form = form.text(
            "map",
            serde_json::to_string(&files_map)
                .expect("Serializing this should always work. Please report this as a bug."),
        );

        let r = make_graphql_request(
            reqwest::Client::new()
                .post(&self.instance_url)
                .header("Authorization", format!("Bearer {}", &self.token))
                .multipart(form),
        )
        .await?;

        if let Some(errors) = r.errors {
            Err(errors[0].clone().into())
        } else if let Some(data) = r.data {
            Ok(data)
        } else {
            unreachable!()
        }
    }
}

fn make_graphql_request<T>(
    builder: reqwest::RequestBuilder,
) -> Pin<Box<dyn Future<Output = Result<GraphQlResponse<T>, CynicReqwestError>> + Send>>
where
    T: DeserializeOwned,
{
    Box::pin(async move {
        match builder.send().await {
            Ok(response) => {
                let status = response.status();
                if !status.is_success() {
                    let body_string = response.text().await?;
                    match serde_json::from_str::<GraphQlResponse<T>>(&body_string) {
                        Ok(response) => {
                            return Ok(response);
                        }
                        Err(_) => {
                            return Err(CynicReqwestError::ErrorResponse(status, body_string));
                        }
                    };
                }
                let body_string = response.text().await?;

                match serde_json::from_str::<GraphQlResponse<T>>(&body_string) {
                    Ok(response) => Ok(response),
                    Err(_) => Err(CynicReqwestError::ErrorResponse(status, body_string)),
                }
            }
            Err(e) => Err(CynicReqwestError::ReqwestError(e)),
        }
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
/// A file upload to the forged.dev API.
pub struct Upload {
    pub(crate) name: String,
    pub(crate) content: Vec<u8>,
}

impl Upload {
    /// Create a new upload for ingestion on the forged.dev server.
    ///
    /// # Args
    /// * `name` - The name of the file being uploaded.
    /// * `content` - The binary content of the uploaded file.
    pub fn new(name: String, content: Vec<u8>) -> Self {
        Self { name, content }
    }
}

impl Serialize for Upload {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_none()
    }
}
