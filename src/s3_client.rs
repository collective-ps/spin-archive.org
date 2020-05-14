use std::time::Duration;

use futures::executor::block_on;
use rusoto_core::Region;
use rusoto_credential::{EnvironmentProvider, ProvideAwsCredentials};
use rusoto_s3::util::{PreSignedRequest, PreSignedRequestOption};
use rusoto_s3::PutObjectRequest;

use crate::models::upload::Upload;

const TEMP: &'static str = "temp";

#[allow(dead_code)]
const UPLOADS: &'static str = "uploads";

fn region() -> Region {
  Region::Custom {
    name: "nyc3".to_owned(),
    endpoint: "https://nyc3.digitaloceanspaces.com".to_owned(),
  }
}

/// Generates a pre-signed url for a given `Upload`.
///
/// Uploads to `TEMP` bucket.
pub fn generate_signed_url(upload: &Upload, content_length: i32) -> String {
  let bucket = "spin-archive";

  let credentials = block_on(EnvironmentProvider::default().credentials()).unwrap();

  let key = format!(
    "{folder}/{name}.{ext}",
    folder = TEMP,
    name = upload.file_id,
    ext = upload.file_ext
  );

  let request = PutObjectRequest {
    bucket: bucket.to_owned(),
    key: key,
    content_length: Some(content_length.into()),
    ..Default::default()
  };

  let url = request.get_presigned_url(
    &region(),
    &credentials,
    &PreSignedRequestOption {
      expires_in: Duration::from_secs(60 * 15),
    },
  );

  url
}
