use std::io::Cursor;

use rocket::http::ContentType;
use rocket::request::{self, FromRequest, Request};
use rocket::response::Response;

use crate::database::DatabaseConnection;
use crate::models::upload;

/// Crawlers are only allowed to see the home page, the static pages and the
/// individual upload pages. Everything combinatorial (search queries, tag and
/// uploader filters, pagination, logs) is disallowed, as are known AI/LLM and
/// SEO crawlers, which bring no visitors to an archive site.
const ROBOTS_TXT: &str = "\
# AI/LLM training and AI search crawlers, plus SEO crawlers: no access.
User-agent: GPTBot
User-agent: OAI-SearchBot
User-agent: ClaudeBot
User-agent: Claude-SearchBot
User-agent: Claude-Web
User-agent: anthropic-ai
User-agent: CCBot
User-agent: Bytespider
User-agent: TikTokSpider
User-agent: Amazonbot
User-agent: meta-externalagent
User-agent: FacebookBot
User-agent: PerplexityBot
User-agent: Google-Extended
User-agent: Applebot-Extended
User-agent: cohere-ai
User-agent: cohere-training-data-crawler
User-agent: Diffbot
User-agent: ImagesiftBot
User-agent: Omgilibot
User-agent: omgili
User-agent: Timpibot
User-agent: YouBot
User-agent: AI2Bot
User-agent: Ai2Bot-Dolma
User-agent: PanguBot
User-agent: Webzio-Extended
User-agent: img2dataset
User-agent: Scrapy
User-agent: AhrefsBot
User-agent: SemrushBot
User-agent: MJ12bot
User-agent: DotBot
User-agent: DataForSeoBot
User-agent: BLEXBot
User-agent: Barkrowler
User-agent: serpstatbot
Disallow: /

# Twitter/X fetches /u/<id> and the /u/<id>/embed player for link cards.
User-agent: Twitterbot
Disallow:

User-agent: *
Disallow: /?
Disallow: /*?
Disallow: /log
Disallow: /random
Disallow: /login
Disallow: /logout
Disallow: /register
Disallow: /upload
Disallow: /queue
Disallow: /admin
Disallow: /api/
Disallow: /webhooks/
Disallow: /user/
Disallow: /u/*/log
Disallow: /u/*/edit
Disallow: /u/*/embed
Disallow: /u/*/comments
Crawl-delay: 10

Sitemap: https://spin-archive.org/sitemap.xml
";

#[rocket::get("/robots.txt")]
pub(crate) fn robots_txt() -> Response<'static> {
    Response::build()
        .header(ContentType::Plain)
        .raw_header("Cache-Control", "public, max-age=86400")
        .sized_body(Cursor::new(ROBOTS_TXT))
        .finalize()
}

/// Lists every completed upload page, so crawlers can find uploads without
/// walking the paginated/filtered index (which robots.txt disallows).
#[rocket::get("/sitemap.xml")]
pub(crate) fn sitemap(conn: DatabaseConnection) -> Response<'static> {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n\
         <url><loc>https://spin-archive.org/</loc></url>\n\
         <url><loc>https://spin-archive.org/tags</loc></url>\n\
         <url><loc>https://spin-archive.org/about</loc></url>\n",
    );

    for (file_id, updated_at) in upload::get_sitemap_entries(&conn) {
        xml.push_str(&format!(
            "<url><loc>https://spin-archive.org/u/{}</loc><lastmod>{}</lastmod></url>\n",
            file_id,
            updated_at.format("%Y-%m-%d")
        ));
    }

    xml.push_str("</urlset>\n");

    Response::build()
        .header(ContentType::XML)
        .raw_header("Cache-Control", "public, max-age=86400")
        .sized_body(Cursor::new(xml))
        .finalize()
}

const BOT_USER_AGENT_MARKERS: &[&str] = &[
    "bot",
    "crawl",
    "spider",
    "slurp",
    "scrap",
    "facebookexternalhit",
    "headless",
    "curl",
    "wget",
    "python",
    "go-http-client",
    "java/",
    "okhttp",
    "httpclient",
];

/// Request guard that flags obvious bots by User-Agent (a missing
/// User-Agent counts as a bot). Always succeeds.
pub(crate) struct IsBot(pub bool);

impl<'a, 'r> FromRequest<'a, 'r> for IsBot {
    type Error = ();

    fn from_request(req: &'a Request<'r>) -> request::Outcome<Self, ()> {
        let is_bot = match req.headers().get_one("User-Agent") {
            Some(user_agent) => {
                let user_agent = user_agent.to_lowercase();
                BOT_USER_AGENT_MARKERS
                    .iter()
                    .any(|marker| user_agent.contains(marker))
            }
            None => true,
        };

        request::Outcome::Success(IsBot(is_bot))
    }
}
