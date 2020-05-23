pub mod tag;

pub(crate) fn router() -> Vec<rocket::Route> {
  rocket::routes![tag::suggestions]
}
