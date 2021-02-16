use std::collections::hash_map::DefaultHasher;
use std::env;
use std::hash::Hasher;

use chrono::{NaiveDateTime, Utc};
use lazy_static::lazy_static;
use rocket::http::hyper::header::{ETag, EntityTag};
use rocket::http::{Method, Status};
use rocket::request::FlashMessage;
use rocket::request::Request;
use rocket::response::{self, content::Html as HtmlCt, Responder, Response};

use crate::models::user::User;
use crate::templates::Html;

lazy_static! {
    #[derive(Copy, Clone, Debug)]
    static ref APP_VERSION: String = env::var("GIT_REV").unwrap_or_default();
}

pub fn split_tags(tags: &str) -> Option<Vec<String>> {
    Some(tags.split_whitespace().map(|s| s.to_owned()).collect())
}

pub fn humanized_past(date: NaiveDateTime) -> String {
    let now = Utc::now().naive_utc();
    let duration = now - date;
    let formatter = timeago::Formatter::new();
    let humanized_date = formatter.convert(duration.to_std().unwrap());
    humanized_date
}

pub fn to_tag_url(name: &str) -> String {
    format!("/?q={}", name)
}

pub fn static_file(content: String) -> Html<String> {
    Html(format!("{}?v={}", content, &**APP_VERSION))
}

pub fn from_markdown(content: &str) -> Html<String> {
    use comrak::{markdown_to_html, ComrakOptions};
    let mut html_output = markdown_to_html(
        content,
        &ComrakOptions {
            ext_strikethrough: true,
            ext_autolink: true,
            ..Default::default()
        },
    );

    html_output = html_output.replace("<a ", "<a rel=\"noopener noreferrer\" target=\"_blank\" ");

    Html(html_output)
}

pub fn truncate<T: std::fmt::Display>(s: T, len: usize) -> String {
    let mut s = s.to_string();
    if s.len() <= len {
        s
    } else {
        let mut real_len = len;
        while !s.is_char_boundary(real_len) {
            real_len += 1;
        }
        s.truncate(real_len);
        s.push_str("...");
        s
    }
}

pub struct Flash {
    pub name: String,
    pub msg: String,
}

pub struct BaseContext<'a> {
    pub user: Option<&'a User>,
    pub flash: Option<Flash>,
}

impl<'a> BaseContext<'a> {
    pub fn new(user: Option<&'a User>, flash: Option<FlashMessage>) -> BaseContext<'a> {
        BaseContext {
            user,
            flash: flash.map(|f| Flash {
                name: f.name().to_owned(),
                msg: f.msg().to_owned(),
            }),
        }
    }
}

pub struct Pagination {
    pub page_count: i64,
    pub current_page: i64,
    pub total_count: i64,
}

#[derive(Debug)]
pub struct Ructe(pub Vec<u8>);

impl<'r> Responder<'r> for Ructe {
    fn respond_to(self, r: &Request<'_>) -> response::Result<'r> {
        //if method is not Get or page contain a form, no caching
        if r.method() != Method::Get || self.0.windows(6).any(|w| w == b"<form ") {
            return HtmlCt(self.0).respond_to(r);
        }
        let mut hasher = DefaultHasher::new();
        hasher.write(&self.0);
        let etag = format!("{:x}", hasher.finish());
        if r.headers()
            .get("If-None-Match")
            .any(|s| s[1..s.len() - 1] == etag)
        {
            Response::build()
                .status(Status::NotModified)
                .header(ETag(EntityTag::strong(etag)))
                .ok()
        } else {
            Response::build()
                .merge(HtmlCt(self.0).respond_to(r)?)
                .header(ETag(EntityTag::strong(etag)))
                .ok()
        }
    }
}

#[macro_export]
macro_rules! render {
    ($group:tt :: $page:tt ( $( $param:expr ),* ) ) => {
        {
            use crate::templates;

            let mut res = vec![];
            templates::$group::$page(
                &mut res,
                $(
                    $param
                ),*
            ).unwrap();
            Ructe(res)
        }
    }
}
