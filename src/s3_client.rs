use std::env;
use std::time::Duration;

use rusoto_core::Region;
use rusoto_credential::AwsCredentials;
use rusoto_s3::util::{PreSignedRequest, PreSignedRequestOption};
use rusoto_s3::PutObjectRequest;

fn region() -> Region {
  Region::Custom {
    name: "us-west-1".to_owned(),
    endpoint: "s3.us-west-1.wasabisys.com".to_owned(),
  }
}

/// Generates a pre-signed url for a given `file_name`.
pub fn generate_signed_url(folder_name: &str, file_name: &str) -> String {
  let access_key = env::var("AWS_ACCESS_KEY_ID").unwrap();
  let secret_key = env::var("AWS_SECRET_ACCESS_KEY").unwrap();
  let bucket = "bits.spin-archive.org";
  let credentials = AwsCredentials::new(access_key, secret_key, None, None);

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
    &credentials,
    &PreSignedRequestOption {
      expires_in: Duration::from_secs(60 * 15),
    },
  );

  url
}
