use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

use crate::models::user::User;
use crate::schema::invitations;

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, QueryableByName, Clone)]
#[table_name = "invitations"]
pub struct Invitation {
    pub id: i64,
    pub code: String,
    pub creator_id: i32,
    pub consumer_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[table_name = "invitations"]
pub struct NewInvitation {
    pub code: String,
    pub creator_id: i32,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset)]
#[table_name = "invitations"]
pub struct UpdateInvitation {
    pub consumer_id: i32,
}

/// Inserts a new [`Invitation`] into the database.
pub fn insert(conn: &PgConnection, invitation: &NewInvitation) -> QueryResult<Invitation> {
    invitation
        .insert_into(invitations::table)
        .returning(invitations::all_columns)
        .get_result(conn)
}

/// Deletes an [`Invitation`] from the database.
pub fn revoke(conn: &PgConnection, user_id: i32, invitation_id: i64) -> QueryResult<usize> {
    diesel::delete(
        invitations::table
            .filter(invitations::id.eq(invitation_id))
            .filter(invitations::creator_id.eq(user_id)),
    )
    .execute(conn)
}

/// Updates a given [`Invitation`] with new column values.
pub fn update(
    conn: &PgConnection,
    id: i64,
    invitation: &UpdateInvitation,
) -> QueryResult<Invitation> {
    diesel::update(invitations::table.filter(invitations::id.eq(id)))
        .set(invitation)
        .get_result::<Invitation>(conn)
}

/// Gets an [`Invitation`] that is unused for the given code.
pub fn by_code(conn: &PgConnection, code: &str) -> Option<Invitation> {
    invitations::table
        .filter(invitations::code.eq(code))
        .filter(invitations::consumer_id.is_null())
        .first::<Invitation>(conn)
        .ok()
}

/// Gets all invitations for a given creator_id.
pub fn get_invitations_by_user(
    conn: &PgConnection,
    user_id: i32,
) -> Vec<(Invitation, Option<User>)> {
    use crate::schema::users;

    invitations::table
        .filter(invitations::creator_id.eq(user_id))
        .left_join(users::table.on(users::id.nullable().eq(invitations::consumer_id)))
        .load::<(Invitation, Option<User>)>(conn)
        .unwrap_or_default()
}
