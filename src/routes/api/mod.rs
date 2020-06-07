pub mod tag;
pub mod uploads;
pub mod user;

pub(crate) fn router() -> Vec<rocket::Route> {
    let mut routes: Vec<rocket::Route> = Vec::new();

    routes.append(&mut tag::routes());
    routes.append(&mut uploads::routes());
    routes.append(&mut user::routes());

    routes
}
