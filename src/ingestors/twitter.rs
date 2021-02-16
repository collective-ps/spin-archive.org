use std::path::PathBuf;

use anyhow::Result;
use egg_mode::entities::{MediaType, VideoVariant};
use lazy_static::lazy_static;
use nanoid::nanoid;
use url::Url;

use super::errors::IngestorError;
use super::transfer_file;
use crate::config;
use crate::database::DatabaseConnection;
use crate::models::upload::Upload;
use crate::models::user::User;
use crate::s3_client;
use crate::services::upload_service;

/// Extracts the status ID from the URL.
fn extract_id_from_url(url: &str) -> Option<u64> {
    lazy_static! {
        static ref RE: regex::Regex = regex::Regex::new(
            r"\Ahttps?://(?:(?:www|mobile)\.)?twitter\.com/(?:i/web|\w+)/status/(\d+)"
        )
        .unwrap();
    }

    match RE.captures(&url) {
        None => None,
        Some(matches) => {
            let id = &matches[1];
            let parsed_id = id.parse::<u64>().unwrap();

            Some(parsed_id.to_owned())
        }
    }
}

#[tokio::main(basic_scheduler)]
pub async fn download_from_tweet(
    conn: DatabaseConnection,
    user: &User,
    tweet_url: &str,
    tag_string: &str,
) -> Result<Upload> {
    let consumer_key = config::get_twitter_consumer_key();
    let consumer_secret = config::get_twitter_consumer_secret();
    let consumer_token = egg_mode::KeyPair::new(consumer_key, consumer_secret);
    let token = egg_mode::bearer_token(&consumer_token).await?;

    let id = extract_id_from_url(&tweet_url);

    if id.is_none() {
        return Err(IngestorError::InvalidUrl.into());
    }

    let status = egg_mode::tweet::show(id.unwrap(), &token).await?;

    if status.extended_entities.is_none() {
        return Err(IngestorError::NoMediaFound.into());
    }

    let entities = status.extended_entities.as_ref().unwrap();

    let url_results: Option<(String, String)> = entities.media.iter().find_map(|media| {
        if media.media_type == MediaType::Video {
            let variants = &media.video_info.as_ref().unwrap().variants;
            let videos: Vec<&VideoVariant> = variants
                .iter()
                .filter(|variant| variant.content_type == "video/mp4")
                .collect();
            let video: Option<&VideoVariant> =
                videos.into_iter().max_by(|a, b| a.bitrate.cmp(&b.bitrate));

            video.map(|video| (media.media_url.clone(), video.url.clone()))
        } else {
            None
        }
    });

    if url_results.is_none() {
        return Err(IngestorError::NoMediaFound.into());
    }

    let urls = url_results.unwrap();
    let thumbnail_url = urls.0;
    let video_url = urls.1;

    let url = Url::parse(&video_url).unwrap();
    let file_path = url.path_segments().unwrap().last().unwrap();
    let path: PathBuf = file_path.into();
    let file_name = path.file_name().unwrap();
    let file_ext = path.extension().unwrap();

    let generated_video_id = nanoid!();
    let generated_thumbnail_id = nanoid!();

    let s3_video_url = s3_client::generate_signed_url(
        "uploads",
        &format!("{}.{}", &generated_video_id, file_ext.to_str().unwrap()),
    );
    let s3_thumbnail_url =
        s3_client::generate_signed_url("t", &format!("{}.jpg", &generated_thumbnail_id));

    let size = transfer_file(&video_url, &s3_video_url).await?;
    let _ = transfer_file(&thumbnail_url, &s3_thumbnail_url).await?;

    let output_thumbnail_url = format!(
        "https://bits.spin-archive.org/t/{}.jpg",
        &generated_thumbnail_id
    );

    upload_service::immediate_upload(
        &conn,
        &user,
        &generated_video_id,
        file_name.to_str().unwrap(),
        file_ext.to_str().unwrap(),
        &output_thumbnail_url,
        size as i64,
        &format!("{} twitter_rip", tag_string),
        &tweet_url,
        &status.text,
        status.created_at.naive_utc().date(),
    )
    .map_err(|err| err.into())
}
