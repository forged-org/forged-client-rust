use std::{collections::HashMap, future::Future, pin::Pin};

use cynic::{http::CynicReqwestError, GraphQlError, GraphQlResponse, Operation};
use regex::Regex;
use reqwest::multipart;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("At least one error was returned from the GraphQL server")]
    Graphql(#[from] GraphQlError),
    #[error("An error with cynic occured")]
    Cynic(#[from] CynicReqwestError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Graphql(error) => {
                writeln!(
                    f,
                    "‣ [{}] {}",
                    error
                        .path
                        .as_ref()
                        .map(|p| p
                            .iter()
                            .map(|e| format!("{:?}", e))
                            .collect::<Vec<_>>()
                            .join("::"))
                        .unwrap_or_else(|| "unknown".into()),
                    error.message
                )
            }
            Error::Other(error) => writeln!(f, "{error}"),
        }
    }
}

pub struct Client {
    token: String,
    instance_url: String,
}

impl Client {
    pub fn new(token: String, instance_url: String) -> Self {
        Self {
            token,
            instance_url,
        }
    }

    pub async fn run_query<T: 'static>(
        &self,
        operation: Operation<'static, T>,
    ) -> Result<T, Error> {
        use cynic::http::ReqwestExt;

        let r = reqwest::Client::new()
            .post(&self.instance_url)
            .bearer_auth(&self.token)
            .run_graphql(operation)
            .await?;

        if let Some(errors) = r.errors {
            Err(errors[0].clone().into())
        } else if let Some(data) = r.data {
            Ok(data)
        } else {
            unreachable!()
        }
    }

    pub async fn run_query_with_file_upload<T: 'static>(
        &self,
        operation: Operation<'static, T>,
        files: Vec<Upload>,
    ) -> Result<T, Error> {
        let mut files_map = HashMap::new();

        let re = Regex::new(r"\$(_\d+): Upload").unwrap();
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
            operation,
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

fn make_graphql_request<'a, ResponseData: 'a>(
    builder: reqwest::RequestBuilder,
    operation: Operation<'a, ResponseData>,
) -> Pin<
    Box<dyn Future<Output = Result<GraphQlResponse<ResponseData>, CynicReqwestError>> + Send + 'a>,
> {
    Box::pin(async move {
        match builder.send().await {
            Ok(response) => {
                let status = response.status();
                if !status.is_success() {
                    let body_string = response.text().await?;

                    match serde_json::from_str::<GraphQlResponse<serde_json::Value>>(&body_string) {
                        Ok(response) => {
                            return operation
                                .decode_response(response)
                                .map_err(CynicReqwestError::DecodeError)
                        }
                        Err(_) => {
                            return Err(CynicReqwestError::ErrorResponse(status, body_string));
                        }
                    };
                }

                response
                    .json::<GraphQlResponse<serde_json::Value>>()
                    .await
                    .map_err(CynicReqwestError::ReqwestError)
                    .and_then(|gql_response| {
                        operation
                            .decode_response(gql_response)
                            .map_err(CynicReqwestError::DecodeError)
                    })
            }
            Err(e) => Err(CynicReqwestError::ReqwestError(e)),
        }
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Upload {
    pub(crate) name: String,
    pub(crate) content: Vec<u8>,
}

impl Upload {
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
