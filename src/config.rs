use std::env;

pub fn secret_key() -> String {
    env::var("SECRET_KEY").unwrap_or("SECRET_KEY_PLEASE_CHANGE_ME".to_owned())
}

pub fn get_webhook_url() -> String {
    env::var("DISCORD_WEBHOOK_URL").unwrap_or_default()
}

pub fn get_contributor_webhook_url() -> String {
    env::var("DISCORD_CONTRIBUTOR_WEBHOOK_URL").unwrap_or_default()
}

pub fn get_twitter_consumer_key() -> String {
    env::var("TWITTER_CONSUMER_KEY").unwrap_or_default()
}

pub fn get_twitter_consumer_secret() -> String {
    env::var("TWITTER_CONSUMER_SECRET").unwrap_or_default()
}
