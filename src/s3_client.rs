use std::time::Duration;

use anyhow::Result;
use rusoto_core::credential::StaticProvider;
use rusoto_core::Region;
use rusoto_credential::AwsCredentials;
use rusoto_s3::util::{PreSignedRequest, PreSignedRequestOption};
use rusoto_s3::S3;
use rusoto_s3::{ListObjectsV2Request, Object, PutObjectRequest, S3Client};

use crate::config;

#[derive(Debug)]
pub struct UploadObject {
    pub md5: String,
    pub file_id: String,
}

fn region() -> Region {
    Region::Custom {
        name: "us-west-1".to_owned(),
        endpoint: "s3.us-west-1.wasabisys.com".to_owned(),
    }
}

fn credentials() -> AwsCredentials {
    AwsCredentials::new(
        config::get_aws_access_key_id(),
        config::get_aws_secret_access_key(),
        None,
        None,
    )
}

fn key_to_file_id(key: String) -> String {
    key.split('.')
        .next()
        .unwrap_or_default()
        .replace("uploads/", "")
        .to_owned()
}

#[tokio::main(basic_scheduler)]
pub async fn list_objects() -> Result<Vec<UploadObject>> {
    let bucket = "bits.spin-archive.org";

    let client = S3Client::new_with(
        rusoto_core::request::HttpClient::new().expect("Failed to create HTTP client"),
        StaticProvider::from(credentials()),
        region(),
    );

    let mut objects: Vec<Object> = Vec::with_capacity(1000);
    let mut reached_end = false;
    let mut continuation_token = None;

    while !reached_end {
        let request = ListObjectsV2Request {
            bucket: bucket.to_owned(),
            prefix: Some("uploads/".to_string()),
            continuation_token,
            ..Default::default()
        };

        let response = client.list_objects_v2(request).await?;

        objects.append(&mut response.contents.unwrap_or_default());

        continuation_token = response.continuation_token;
        reached_end = !response.is_truncated.unwrap_or(true);
    }

    let uploads: Vec<UploadObject> = objects
        .into_iter()
        .map(|object| UploadObject {
            md5: object.e_tag.unwrap_or_default().replace("\"", ""),
            file_id: key_to_file_id(object.key.unwrap_or_default()),
        })
        .collect();

    Ok(uploads)
}

/// Generates a pre-signed url for a given `file_name`.
pub fn generate_signed_url(folder_name: &str, file_name: &str) -> String {
    let bucket = "bits.spin-archive.org";

    let key = format!(
        "{folder}/{file_name}",
        folder = folder_name,
        file_name = file_name
    );

    let request = PutObjectRequest {
        bucket: bucket.to_owned(),
        key: key,
        ..Default::default()
    };

    let url = request.get_presigned_url(
        &region(),
        &credentials(),
        &PreSignedRequestOption {
            expires_in: Duration::from_secs(60 * 15),
        },
    );

    url
}
