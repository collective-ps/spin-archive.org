#[rocket_contrib::database("spin_archive")]
pub struct DatabaseConnection(diesel::PgConnection);
