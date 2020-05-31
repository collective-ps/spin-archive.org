use anyhow::Result;
use reqwest::Body;
use std::io::SeekFrom;
use tempfile::tempfile;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

mod errors;
pub mod twitter;

fn file_to_body(file: File) -> Body {
  let stream = FramedRead::new(file, BytesCodec::new());
  reqwest::Body::wrap_stream(stream)
}

pub async fn transfer_file(source_url: &str, destination_url: &str) -> Result<u64> {
  let client = reqwest::Client::new();

  let resp = client.get(source_url).send().await?;
  let bytes = resp.bytes().await?;

  let temp_file = tempfile().unwrap();
  let mut async_temp_file = File::from_std(temp_file);

  let size = tokio::io::copy(&mut &*bytes, &mut async_temp_file).await?;

  async_temp_file.seek(SeekFrom::Start(0)).await?;

  let upload = client
    .put(destination_url)
    .header("content-length", size)
    .body(file_to_body(async_temp_file))
    .send()
    .await?;

  dbg!(&upload);

  Ok(size)
}
