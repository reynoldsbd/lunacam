table! {
    cameras (id) {
        id -> Integer,
        friendly_name -> Text,
        hostname -> Text,
        device_key -> Text,
        enabled -> Bool,
        orientation -> Integer,
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

allow_tables_to_appear_in_same_query!(
    cameras,
    settings,
    users,
);
