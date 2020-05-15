use std::time::Duration;

use futures::executor::block_on;
use rusoto_core::Region;
use rusoto_credential::{EnvironmentProvider, ProvideAwsCredentials};
use rusoto_s3::util::{PreSignedRequest, PreSignedRequestOption};
use rusoto_s3::PutObjectRequest;

const UPLOADS: &'static str = "uploads";

fn region() -> Region {
  Region::Custom {
    name: "nyc3".to_owned(),
    endpoint: "https://nyc3.digitaloceanspaces.com".to_owned(),
  }
}

/// Generates a pre-signed url for a given `file_name`.
///
/// Uploads to `UPLOADS` bucket.
pub fn generate_signed_url(file_name: &str) -> String {
  let bucket = "spin-archive";

  let credentials = block_on(EnvironmentProvider::default().credentials()).unwrap();

  let key = format!(
    "{folder}/{file_name}",
    folder = UPLOADS,
    file_name = file_name
  );

  let request = PutObjectRequest {
    bucket: bucket.to_owned(),
    key: key,
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
