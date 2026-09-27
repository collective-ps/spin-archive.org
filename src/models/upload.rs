use std::io::Write;

use chrono::{NaiveDate, NaiveDateTime};
use diesel::{
    deserialize::{self, FromSql},
    expression::{helper_types::AsExprOf, AsExpression},
    prelude::*,
    serialize::{self, Output, ToSql},
    sql_types, AsChangeset, Identifiable, SqliteConnection, Queryable,
};
use serde::{Deserialize, Serialize};

use crate::database::last_insert_rowid;
use crate::models::tag::Tag;
use crate::models::user::{User, UserRole};
use crate::schema::uploads;

type AllColumns = (
    uploads::id,
    uploads::status,
    uploads::file_id,
    uploads::file_size,
    uploads::file_name,
    uploads::md5_hash,
    uploads::uploader_user_id,
    uploads::source,
    uploads::created_at,
    uploads::updated_at,
    uploads::file_ext,
    uploads::tag_string,
    uploads::video_encoding_key,
    uploads::thumbnail_url,
    uploads::video_url,
    uploads::description,
    uploads::original_upload_date,
);

pub const ALL_COLUMNS: AllColumns = (
    uploads::id,
    uploads::status,
    uploads::file_id,
    uploads::file_size,
    uploads::file_name,
    uploads::md5_hash,
    uploads::uploader_user_id,
    uploads::source,
    uploads::created_at,
    uploads::updated_at,
    uploads::file_ext,
    uploads::tag_string,
    uploads::video_encoding_key,
    uploads::thumbnail_url,
    uploads::video_url,
    uploads::description,
    uploads::original_upload_date,
);

#[allow(dead_code)]
type All = diesel::dsl::Select<uploads::table, AllColumns>;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, FromSqlRow, AsExpression)]
#[repr(i16)]
pub enum UploadStatus {
    Pending = 0,
    Processing = 1,
    Completed = 2,
    Failed = 3,
    Deleted = 4,
    PendingApproval = 5,
}

impl std::fmt::Display for UploadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let role = match self {
            UploadStatus::Pending => "Pending",
            UploadStatus::Processing => "Processing",
            UploadStatus::Completed => "Completed",
            UploadStatus::Failed => "Failed",
            UploadStatus::Deleted => "Deleted",
            UploadStatus::PendingApproval => "Pending Approval",
        };

        write!(f, "{}", role)
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, AsChangeset, Associations)]
#[belongs_to(User, foreign_key = "uploader_user_id")]
#[table_name = "uploads"]
pub struct Upload {
    pub id: i32,
    pub status: UploadStatus,
    pub file_id: String,
    pub file_size: Option<i64>,
    pub file_name: Option<String>,
    pub md5_hash: Option<String>,
    pub uploader_user_id: Option<i32>,
    pub source: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub file_ext: String,
    pub tag_string: String,
    pub video_encoding_key: String,
    pub thumbnail_url: Option<String>,
    pub video_url: Option<String>,
    pub description: String,
    pub original_upload_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, QueryableByName)]
#[table_name = "uploads"]
pub struct FullUpload {
    pub id: i32,
    pub status: UploadStatus,
    pub file_id: String,
    pub file_size: Option<i64>,
    pub file_name: Option<String>,
    pub md5_hash: Option<String>,
    pub uploader_user_id: Option<i32>,
    pub source: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub file_ext: String,
    pub tag_string: String,
    pub video_encoding_key: String,
    pub thumbnail_url: Option<String>,
    pub video_url: Option<String>,
    pub description: String,
    pub original_upload_date: Option<NaiveDate>,

    #[sql_type = "sql_types::Text"]
    pub uploader_username: String,
    #[sql_type = "sql_types::SmallInt"]
    pub uploader_role: UserRole,
    #[sql_type = "sql_types::BigInt"]
    pub comment_count: i64,
    #[sql_type = "sql_types::BigInt"]
    pub view_count: i64,
    #[sql_type = "sql_types::BigInt"]
    pub count: i64,
}

impl FullUpload {
    /// Gets the thumbnail URL
    pub fn get_thumbnail_url(&self) -> String {
        self.thumbnail_url
            .as_ref()
            .map(|s| s.to_owned())
            .unwrap_or("".to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, AsChangeset)]
#[table_name = "uploads"]
pub struct UpdateUpload {
    pub id: i32,
    pub status: UploadStatus,
    pub source: Option<String>,
    pub tag_string: String,
    pub description: String,
    pub original_upload_date: Option<NaiveDate>,
}

#[derive(Insertable)]
#[table_name = "uploads"]
pub struct PendingUpload {
    pub status: UploadStatus,
    pub file_id: String,
    pub video_encoding_key: String,
    pub uploader_user_id: i32,
    pub file_name: String,
    pub file_ext: String,
    pub file_size: i64,
    pub md5_hash: Option<String>,
}

#[derive(Insertable)]
#[table_name = "uploads"]
pub struct FinalizeUpload {
    pub status: UploadStatus,
    pub tag_string: String,
    pub original_upload_date: Option<NaiveDate>,
}

#[derive(AsChangeset)]
#[table_name = "uploads"]
pub struct FinishedEncodingUpload {
    pub status: UploadStatus,
    pub thumbnail_url: String,
    pub video_url: String,
}

#[derive(Insertable)]
#[table_name = "uploads"]
pub struct NewImmediateUpload {
    pub file_ext: String,
    pub file_id: String,
    pub file_name: String,
    pub file_size: i64,
    pub status: UploadStatus,
    pub thumbnail_url: String,
    pub uploader_user_id: i32,
    pub video_encoding_key: String,
    pub tag_string: String,
    pub source: String,
    pub description: String,
    pub original_upload_date: NaiveDate,
}

const ASSET_HOST: &'static str = "https://bits.spin-archive.org/uploads";

impl Upload {
    /// Gets the full URL to where the file is stored.
    pub fn get_file_url(&self) -> String {
        format!(
            "{host}/{file_id}.{ext}",
            host = ASSET_HOST,
            file_id = self.file_id,
            ext = self.file_ext
        )
    }

    /// Gets the encoded video URL, or falls back to the original URL.
    pub fn get_video_url(&self) -> String {
        self.video_url
            .as_ref()
            .unwrap_or(&self.get_file_url())
            .to_string()
    }

    /// Gets the thumbnail URL
    pub fn get_thumbnail_url(&self) -> String {
        self.thumbnail_url
            .as_ref()
            .map(|s| s.to_owned())
            .unwrap_or("".to_string())
    }

    pub fn is_video(&self) -> bool {
        let mime = mime_guess::from_ext(&self.file_ext).first_or_octet_stream();

        mime.to_string().starts_with("video/")
    }
}

impl<DB> ToSql<sql_types::SmallInt, DB> for UploadStatus
where
    DB: diesel::backend::Backend,
    i16: ToSql<sql_types::SmallInt, DB>,
{
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        (*self as i16).to_sql(out)
    }
}

impl<DB> FromSql<sql_types::SmallInt, DB> for UploadStatus
where
    DB: diesel::backend::Backend,
    i16: FromSql<sql_types::SmallInt, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        match i16::from_sql(bytes)? {
            0 => Ok(UploadStatus::Pending),
            1 => Ok(UploadStatus::Processing),
            2 => Ok(UploadStatus::Completed),
            3 => Ok(UploadStatus::Failed),
            4 => Ok(UploadStatus::Deleted),
            5 => Ok(UploadStatus::PendingApproval),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl AsExpression<sql_types::SmallInt> for UploadStatus {
    type Expression = AsExprOf<i16, sql_types::SmallInt>;

    fn as_expression(self) -> Self::Expression {
        <i16 as AsExpression<sql_types::SmallInt>>::as_expression(self as i16)
    }
}

impl AsExpression<sql_types::SmallInt> for &UploadStatus {
    type Expression = AsExprOf<i16, sql_types::SmallInt>;

    fn as_expression(self) -> Self::Expression {
        <i16 as AsExpression<sql_types::SmallInt>>::as_expression(*self as i16)
    }
}

/// Gets an [`Upload`] by `id`.
fn get_by_id(conn: &SqliteConnection, upload_id: i32) -> QueryResult<Upload> {
    uploads::table
        .filter(uploads::id.eq(upload_id))
        .select(ALL_COLUMNS)
        .first::<Upload>(conn)
}

/// Gets an [`Upload`] by `file_id`.
pub fn get_by_file_id(conn: &SqliteConnection, search_file_id: &str) -> Option<Upload> {
    use crate::schema::uploads::dsl::*;

    uploads
        .filter(file_id.eq(search_file_id))
        .select(ALL_COLUMNS)
        .first::<Upload>(conn)
        .ok()
}

/// Gets an [`Upload`] by `source`.
pub fn get_by_source(conn: &SqliteConnection, source_url: &str) -> Option<Upload> {
    use crate::schema::uploads::dsl::*;

    uploads
        .filter(source.eq(source_url))
        .select(ALL_COLUMNS)
        .first::<Upload>(conn)
        .ok()
}

/// Gets an [`Upload`] by `video_encoding_key`.
pub fn get_by_video_encoding_key(conn: &SqliteConnection, search_key: &str) -> Option<Upload> {
    use crate::schema::uploads::dsl::*;

    uploads
        .filter(video_encoding_key.eq(search_key))
        .select(ALL_COLUMNS)
        .first::<Upload>(conn)
        .ok()
}

/// Gets an [`Upload`] by `file_name` + `file_ext` + `file_size`.
pub fn get_by_original_file(
    conn: &SqliteConnection,
    file_name: &str,
    file_ext: &str,
    file_size: i64,
) -> Option<Upload> {
    uploads::table
        .filter(uploads::file_name.eq(file_name))
        .filter(uploads::file_ext.eq(file_ext))
        .filter(uploads::file_size.eq(file_size))
        .filter(uploads::status.ne(UploadStatus::Pending))
        .filter(uploads::status.ne(UploadStatus::Deleted))
        .select(ALL_COLUMNS)
        .first::<Upload>(conn)
        .ok()
}

/// Gets an [`Upload`] by `md5_hash`.
pub fn get_by_md5(conn: &SqliteConnection, md5_hash: &str) -> Option<Upload> {
    uploads::table
        .filter(uploads::md5_hash.eq(md5_hash))
        .filter(uploads::status.ne(UploadStatus::Pending))
        .filter(uploads::status.ne(UploadStatus::Deleted))
        .select(ALL_COLUMNS)
        .first::<Upload>(conn)
        .ok()
}

/// Get uploads where matching by md5 hashes.
pub fn where_md5(conn: &SqliteConnection, hashes: &Vec<String>) -> Vec<Upload> {
    uploads::table
        .select(uploads::md5_hash)
        .filter(uploads::md5_hash.eq_any(hashes))
        .filter(uploads::md5_hash.is_not_null())
        .filter(uploads::status.ne(UploadStatus::Pending))
        .filter(uploads::status.ne(UploadStatus::Deleted))
        .select(ALL_COLUMNS)
        .load::<Upload>(conn)
        .unwrap_or_default()
}

/// Updates a given [`Upload`] with new column values.
pub fn update(conn: &SqliteConnection, upload: &UpdateUpload) -> QueryResult<Upload> {
    diesel::update(uploads::table.filter(uploads::id.eq(upload.id)))
        .set(upload)
        .execute(conn)?;

    get_by_id(conn, upload.id)
}

/// Updates a given [`Upload`] based on encoding response.
pub fn update_encoding(
    conn: &SqliteConnection,
    id: i32,
    upload: &FinishedEncodingUpload,
) -> QueryResult<Upload> {
    diesel::update(uploads::table.filter(uploads::id.eq(id)))
        .set(upload)
        .execute(conn)?;

    get_by_id(conn, id)
}

/// Updates a given [`Upload`] to given [`UploadStatus`].
pub fn update_status(
    conn: &SqliteConnection,
    upload_id: i32,
    status: UploadStatus,
) -> QueryResult<Upload> {
    diesel::update(uploads::table.filter(uploads::id.eq(upload_id)))
        .set(uploads::status.eq(status))
        .execute(conn)?;

    get_by_id(conn, upload_id)
}

/// Inserts a given [`PendingUpload`] into the database.
pub fn insert_pending_upload(
    conn: &SqliteConnection,
    pending_upload: &PendingUpload,
) -> QueryResult<Upload> {
    diesel::insert_into(uploads::table)
        .values(pending_upload)
        .execute(conn)?;

    get_by_id(conn, last_insert_rowid(conn)? as i32)
}

/// Inserts a given [`NewImmediateUpload`] into the database.
pub fn insert_immediate_upload(
    conn: &SqliteConnection,
    immediate_upload: &NewImmediateUpload,
) -> QueryResult<Upload> {
    diesel::insert_into(uploads::table)
        .values(immediate_upload)
        .execute(conn)?;

    get_by_id(conn, last_insert_rowid(conn)? as i32)
}

/// Gets all pending approval uploads and their uploader.
pub fn get_pending_approval_uploads(conn: &SqliteConnection) -> Vec<(Upload, User)> {
    use crate::schema::users;

    uploads::table
        .filter(uploads::status.eq(UploadStatus::PendingApproval))
        .inner_join(users::table)
        .select((ALL_COLUMNS, users::all_columns))
        .load::<(Upload, User)>(conn)
        .unwrap_or_default()
}

/// Wraps a query selecting `uploads.*` plus a `count` column into one returning [`FullUpload`]s,
/// adding the uploader and comment/view counts for just the selected page of rows.
fn full_upload_query(page_query: &str) -> String {
    format!(
        "
        SELECT page.*,
            users.username AS uploader_username,
            users.role AS uploader_role,
            (SELECT count(*) FROM upload_comments WHERE upload_comments.upload_id = page.id) AS comment_count,
            (SELECT count(*) FROM upload_views WHERE upload_views.upload_id = page.id) AS view_count
        FROM ({page_query}) page
        LEFT JOIN users ON page.uploader_user_id = users.id
        ORDER BY page.created_at DESC
        ",
        page_query = page_query
    )
}

/// Encodes whitespace-separated search terms as a JSON array, for use with `json_each()`.
fn tags_json<'a, I: IntoIterator<Item = &'a str>>(tags: I) -> String {
    let tags: Vec<String> = tags.into_iter().map(|tag| tag.to_lowercase()).collect();

    serde_json::to_string(&tags).unwrap_or("[]".to_owned())
}

/// Index query for uploads, fetches completed uploads by the page number provided.
///
/// A non-empty `query` matches uploads that have every term as a tag, or whose
/// original file name contains the query.
///
/// Returns a tuple: (Vec<Upload>, page_count, total_count).
pub fn index(
    conn: &SqliteConnection,
    page: i64,
    per_page: i64,
    query: &str,
    uploader: Option<User>,
) -> (Vec<FullUpload>, i64, i64) {
    use diesel::sql_types::*;

    let query = query.trim();
    let search = if query.is_empty() { None } else { Some(query) };

    let result = diesel::sql_query(full_upload_query(
        "
        SELECT uploads.*, count(*) OVER () AS count
        FROM uploads
        WHERE uploads.status = ?1
        AND (?2 IS NULL OR uploads.uploader_user_id = ?2)
        AND (?3 IS NULL
            OR uploads.file_name LIKE '%' || ?3 || '%'
            OR NOT EXISTS (
                SELECT 1 FROM json_each(?4) AS term
                WHERE instr(' ' || uploads.tag_string || ' ', ' ' || term.value || ' ') = 0
            ))
        ORDER BY uploads.created_at DESC
        LIMIT ?5
        OFFSET ?6
        ",
    ))
    .bind::<SmallInt, _>(UploadStatus::Completed)
    .bind::<Nullable<Integer>, _>(uploader.map(|user| user.id))
    .bind::<Nullable<Text>, _>(search)
    .bind::<Text, _>(tags_json(query.split_whitespace()))
    .bind::<BigInt, _>(per_page)
    .bind::<BigInt, _>((page - 1) * per_page)
    .load::<FullUpload>(conn);

    let full_uploads = result.unwrap_or_else(|err| {
        log::error!("Failed to load uploads index: {:?}", err);
        Vec::new()
    });

    let total_count = full_uploads
        .first()
        .map(|full_upload| full_upload.count)
        .unwrap_or(0);

    let total_pages = (total_count as f64 / per_page as f64).ceil() as i64;

    (full_uploads, total_pages, total_count)
}

pub fn get_upload_count_by_user_id(conn: &SqliteConnection, user_id: i32) -> i64 {
    use diesel::dsl::count;

    uploads::table
        .select(count(uploads::id))
        .filter(uploads::uploader_user_id.eq(user_id))
        .filter(uploads::status.eq(UploadStatus::Completed))
        .first::<i64>(conn)
        .unwrap_or_default()
}

pub fn update_md5(conn: &SqliteConnection, file_id: &str, md5: &str) -> QueryResult<usize> {
    diesel::update(uploads::table.filter(uploads::file_id.eq(file_id)))
        .set(uploads::md5_hash.eq(md5))
        .execute(conn)
}

pub fn random(conn: &SqliteConnection) -> Option<Upload> {
    use diesel::dsl::sql;
    use diesel::sql_types::Integer;

    uploads::table
        .select(ALL_COLUMNS)
        .order(sql::<Integer>("random()"))
        .filter(uploads::status.eq(UploadStatus::Completed))
        .limit(1)
        .first::<Upload>(conn)
        .ok()
}

/// (file_id, updated_at) of every completed upload, newest first, for the sitemap.
pub fn get_sitemap_entries(conn: &SqliteConnection) -> Vec<(String, NaiveDateTime)> {
    uploads::table
        .select((uploads::file_id, uploads::updated_at))
        .filter(uploads::status.eq(UploadStatus::Completed))
        .order(uploads::created_at.desc())
        .load(conn)
        .unwrap_or_default()
}

pub fn get_with_any_tags(
    conn: &SqliteConnection,
    tags: Vec<&Tag>,
    excluding_id: i32,
) -> Vec<FullUpload> {
    use diesel::sql_types::*;

    // `abs(random()) % 2 = 0` stands in for Postgres' `TABLESAMPLE BERNOULLI (50)`,
    // so recommendations vary between page loads.
    diesel::sql_query(full_upload_query(
        "
        SELECT uploads.*, count(*) OVER () AS count
        FROM uploads
        WHERE uploads.status = ?1
        AND uploads.id != ?2
        AND abs(random()) % 2 = 0
        AND EXISTS (
            SELECT 1 FROM json_each(?3) AS tag
            WHERE instr(' ' || uploads.tag_string || ' ', ' ' || tag.value || ' ') > 0
        )
        ORDER BY uploads.created_at DESC
        LIMIT 6
        ",
    ))
    .bind::<SmallInt, _>(UploadStatus::Completed)
    .bind::<Integer, _>(excluding_id)
    .bind::<Text, _>(tags_json(tags.iter().map(|tag| tag.name.as_ref())))
    .load::<FullUpload>(conn)
    .expect("Could not get_with_any_tags()")
}
