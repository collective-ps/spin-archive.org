table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

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
        tag_index -> Nullable<TsVector>,
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

allow_tables_to_appear_in_same_query!(uploads, users,);
