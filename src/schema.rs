table! {
    tags (id) {
        id -> Int4,
        name -> Text,
    }
}

table! {
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
    }
}

table! {
    uploads_tags (id) {
        id -> Int4,
        tag_id -> Nullable<Int4>,
        upload_id -> Nullable<Int4>,
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

joinable!(uploads -> users (uploader_user_id));
joinable!(uploads_tags -> tags (tag_id));
joinable!(uploads_tags -> uploads (upload_id));

allow_tables_to_appear_in_same_query!(
    tags,
    uploads,
    uploads_tags,
    users,
);
