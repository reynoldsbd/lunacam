table! {
    cameras (id) {
        id -> Integer,
        name -> Text,
        address -> Text,
        enabled -> Bool,
        orientation -> Integer,
        local -> Bool,
    }
}

table! {
    sessions (id) {
        id -> Integer,
        key -> Text,
        user_id -> Integer,
    }
}

table! {
    settings (name) {
        name -> Text,
        value -> Text,
    }
}

table! {
    users (id) {
        id -> Integer,
        username -> Text,
        pwhash -> Text,
    }
}

joinable!(sessions -> users (user_id));

allow_tables_to_appear_in_same_query!(
    cameras,
    sessions,
    settings,
    users,
);
