use crate::{Client, Error};
use anyhow::anyhow;
use cynic::impl_scalar;
use cynic::QueryBuilder;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use futures_util::StreamExt;

impl Client {
    pub async fn binary_part(
        &self,
        binary_id: Uuid,
        part_id: Uuid,
        update_handler: Option<impl Fn(f64)>,
    ) -> Result<Vec<u8>, Error> {
        // Query the part hash
        let result = self
            .run_query(QueryBinaryPartHash::build(PartHashArguments { binary_id }))
            .await?;

        let Some(part) = result
            .current_provisioner
            .project
            .binary
            .parts
            .iter()
            .find(|part| part.id == part_id)
        else {
            return Err(anyhow!("Part not found").into());
        };

        self.fetch_url(result.current_provisioner.project.id, part, update_handler)
            .await
    }

    async fn fetch_url(
        &self,
        project_id: Uuid,
        part: &BinaryPart,
        update_handler: Option<impl Fn(f64)>,
    ) -> Result<Vec<u8>, Error> {
        let url = format!(
            "{api_url}/project/{project_id}/binary/{binary_id}/part/{part_id}",
            api_url = self.instance_url,
            binary_id = part.binary_id,
            part_id = part.id,
        );

        if let Some(cache_folder) = &self.cache_folder {
            let mut cache_file = cache_folder.clone();
            cache_file.push(format!("{}", part.id));

            log::info!("Reading the binary cache file at {cache_file:?}");
            match tokio::fs::read(&cache_file).await {
                Ok(file_content) => {
                    let part_hash = part.image_hash.iter().map(|v| *v as u8).collect();
                    let mut hasher = Sha256::new();
                    hasher.update(&file_content);
                    let image_hash = hasher.finalize();
                    if image_hash == part_hash {
                        log::info!("Read firmware from local cache");
                        return Ok(file_content);
                    } else {
                        log::warn!("Cached file at {cache_file:?} is corrupt");

                        if let Err(error) = tokio::fs::remove_file(&cache_file).await {
                            log::warn!(
                                "Removing the corrupt binary cache file at {cache_file:?} failed: {error}"
                            );
                            log::warn!("Please remove it manually!");
                        };
                    }
                }
                Err(error) => {
                    log::info!("Reading the binary cache file at {cache_file:?} failed: {error}");
                }
            };
        }

        log::info!("Downloading firmware from remote");

        let response = reqwest::Client::new()
            .get(url)
            .bearer_auth(self.token.clone())
            .send()
            .await?;

        let total_size = response
            .content_length()
            .ok_or_else(|| Error::Api(anyhow!("Could not get total size.")))?;

        let mut data: Vec<u8> = Vec::with_capacity(total_size as usize);
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item?;
            data.extend(&chunk);
            let new = core::cmp::min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            if let Some(update) = &update_handler {
                update(downloaded as f64 / total_size as f64)
            }
        }
        if let Some(update) = update_handler {
            update(1.0)
        }

        if let Some(cache_folder) = &self.cache_folder {
            let mut cache_file = cache_folder.clone();
            cache_file.push(format!("{}", part.id));
            if let Err(error) = tokio::fs::write(&cache_file, &data).await {
                log::warn!("Writing the binary cache file at {cache_file:?} failed: {error}");
            };
        }

        Ok(data)
    }
}

pub use queries::*;

#[cynic::schema_for_derives(file = "schema.graphql", module = "schema")]
pub mod queries {
    use super::schema;
    use uuid::Uuid;

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "QueryRoot", variables = "PartHashArguments")]
    pub struct QueryBinaryPartHash {
        pub current_provisioner: Provisioner,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(variables = "PartHashArguments")]
    pub struct Provisioner {
        pub project: Project,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(variables = "PartHashArguments")]
    pub struct Project {
        #[arguments(id: $binary_id)]
        pub binary: Binary,
        pub id: Uuid,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Binary {
        pub parts: Vec<BinaryPart>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct BinaryPart {
        pub id: Uuid,
        pub binary_id: Uuid,
        pub image_hash: Vec<i32>,
    }

    #[derive(cynic::QueryVariables, Debug)]
    pub struct PartHashArguments {
        pub binary_id: Uuid,
    }
}

mod schema {
    cynic::use_schema!("schema.graphql");
}

impl_scalar!(Uuid, schema::UUID);
