#[rocket_contrib::database("spin_archive")]
pub(crate) struct DatabaseConnection(diesel::PgConnection);
