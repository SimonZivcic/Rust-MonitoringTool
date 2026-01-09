diesel::table! {
    servers (id) {
        id -> Integer,
        name -> Text,
        status -> Text,
        port -> Integer,
        cpu_model -> Text,
        max_ram -> Float,
    }
}

diesel::table! {
    history (id) {
        id -> Integer,
        server_id -> Integer,
        timestamp -> Timestamp,
        response_ms -> Integer,
        ram_usage -> Float,
        cpu_usage -> Float,
    }
}