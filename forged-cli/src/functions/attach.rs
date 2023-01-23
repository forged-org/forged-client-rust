use crate::Result;
use cynic::MutationBuilder;

use crate::queries::{CreateAttachment, CreateAttachmentArguments};

pub async fn attach(client: &mut forged::Client, file_path: String) -> Result<()> {
    println!("📎  Attaching file: {file_path}");

    let data = std::fs::read(&file_path).unwrap();

    let upload = forged::Upload::new(file_path.clone(), data);
    client
        .run_query_with_file_upload(
            CreateAttachment::build(&CreateAttachmentArguments {
                data: upload.clone(),
            }),
            vec![upload],
        )
        .await?;

    Ok(())
}
