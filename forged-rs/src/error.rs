use std::fmt::Display;

use cynic::GraphQlError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    Graphql(Vec<GraphQlError>),
    Other(#[from] anyhow::Error),
}

impl From<Vec<GraphQlError>> for Error {
    fn from(err: Vec<GraphQlError>) -> Self {
        Error::Graphql(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Graphql(errors) => {
                for error in errors {
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
                    )?
                }
                Ok(())
            }
            Error::Other(error) => writeln!(f, "{error}"),
        }
    }
}
