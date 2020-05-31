pub mod tag;
pub mod uploads;

pub(crate) fn router() -> Vec<rocket::Route> {
    rocket::routes![
        tag::suggestions,
        uploads::search,
        uploads::new,
        uploads::finalize,
        uploads::twitter
    ]
}
