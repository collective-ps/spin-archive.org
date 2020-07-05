table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector};

    api_tokens (id) {
        id -> Int8,
        token -> Text,
        user_id -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector};

    audit_log (id) {
        id -> Int8,
        table_name -> Text,
        column_name -> Text,
        row_id -> Int4,
        changed_date -> Timestamp,
        changed_by -> Int4,
        old_value -> Text,
        new_value -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector};

    forums (id) {
        id -> Int8,
        title -> Text,
        description -> Text,
        order_key -> Int4,
        is_open -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector};

    posts (id) {
        id -> Int8,
        thread_id -> Int8,
        author_id -> Int4,
        content -> Text,
        is_deleted -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector};

    tags (id) {
        id -> Int8,
        name -> Text,
        description -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        upload_count -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector};

    threads (id) {
        id -> Int8,
        title -> Text,
        forum_id -> Int8,
        author_id -> Int4,
        is_sticky -> Bool,
        is_open -> Bool,
        is_deleted -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector};

    upload_comments (id) {
        id -> Int8,
        upload_id -> Int4,
        user_id -> Int4,
        comment -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector};

    upload_views (id) {
        id -> Int8,
        upload_id -> Int4,
        viewed_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector};

    uploads (id) {
        id -> Int4,
        status -> Int2,
        file_id -> Text,
        file_size -> Nullable<Int8>,
        file_name -> Nullable<Text>,
        md5_hash -> Nullable<Text>,
        uploader_user_id -> Nullable<Int4>,
        source -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        file_ext -> Text,
        tag_string -> Text,
        tag_index -> Tsvector,
        video_encoding_key -> Text,
        thumbnail_url -> Nullable<Text>,
        video_url -> Nullable<Text>,
        description -> Text,
        original_upload_date -> Nullable<Date>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector};

    users (id) {
        id -> Int4,
        username -> Text,
        password_hash -> Text,
        email -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        role -> Int2,
        daily_upload_limit -> Int4,
    }
}

joinable!(api_tokens -> users (user_id));
joinable!(audit_log -> users (changed_by));
joinable!(posts -> threads (thread_id));
joinable!(posts -> users (author_id));
joinable!(threads -> forums (forum_id));
joinable!(threads -> users (author_id));
joinable!(upload_comments -> uploads (upload_id));
joinable!(upload_comments -> users (user_id));
joinable!(upload_views -> uploads (upload_id));
joinable!(uploads -> users (uploader_user_id));

allow_tables_to_appear_in_same_query!(
    api_tokens,
    audit_log,
    forums,
    posts,
    tags,
    threads,
    upload_comments,
    upload_views,
    uploads,
    users,
);
