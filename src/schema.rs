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
