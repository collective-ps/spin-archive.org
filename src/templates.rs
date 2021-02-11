use askama::Template;

mod filters {}

#[derive(Template)]
#[template(path = "base.html")]
pub struct Base {}

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index {}
