//definicia struktury

//tabulka zakladnych informacii
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

//tabulka z casovymi zaznammi a vykone
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