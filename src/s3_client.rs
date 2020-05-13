use std::time::Duration;

use futures::executor::block_on;
use rusoto_core::credential::EnvironmentProvider;
use rusoto_core::Region;
use rusoto_credential::ProvideAwsCredentials;
use rusoto_s3::util::{PreSignedRequest, PreSignedRequestOption};

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
pub fn generate_signed_url(upload: &Upload) -> String {
  let put_request = rusoto_s3::PutObjectRequest {
    bucket: TEMP.to_owned(),
    key: upload.file_id.to_owned(),
    ..Default::default()
  };

  put_request.get_presigned_url(
    &region(),
    &block_on(EnvironmentProvider::default().credentials()).unwrap(),
    &PreSignedRequestOption {
      expires_in: Duration::from_secs(60 * 2),
    },
  )
}
