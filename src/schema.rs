table! {
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
    users (id) {
        id -> Int4,
        username -> Text,
        password_hash -> Text,
        email -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        role -> Int2,
    }
}

joinable!(audit_log -> users (changed_by));
joinable!(upload_comments -> uploads (upload_id));
joinable!(upload_comments -> users (user_id));
joinable!(upload_views -> uploads (upload_id));
joinable!(uploads -> users (uploader_user_id));

allow_tables_to_appear_in_same_query!(
    audit_log,
    tags,
    upload_comments,
    upload_views,
    uploads,
    users,
);
