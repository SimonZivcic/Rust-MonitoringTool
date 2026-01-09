use diesel::prelude::*;
pub use diesel::sqlite::SqliteConnection;
use std::fs;
use crate::models::Server;
use crate::schema::servers;

pub fn establish_connection() -> SqliteConnection {
    let database_url = "servers.db";
    let mut conn = SqliteConnection::establish(database_url)
        .expect("Nepodarilo sa pripojiť k servers.db");

    // TOTO VLOŽÍ SQL KÓD PRIAMO DO BINÁRKY PRI KOMPILÁCII
    // Cesta je relatívna k súboru db.rs (takže o jednu úroveň vyššie do src a potom do koreňa lib)
    let sql = include_str!("../../schema.sql");
    
    for query in sql.split(';') {
        if !query.trim().is_empty() {
            diesel::sql_query(query).execute(&mut conn)
                .expect("Chyba pri spúšťaní schémy");
        }
    }

    conn
}

pub fn add_server(conn: &mut SqliteConnection, name_str: &str, ram: f32, port_val: i32, cpu: &str) {
    diesel::insert_into(servers::table)
        .values((
            servers::name.eq(name_str),
            servers::max_ram.eq(ram),
            servers::port.eq(port_val),
            servers::cpu_model.eq(cpu),
            servers::status.eq("/") // Predvolený stav neznáme 
        ))
        .execute(conn).unwrap();
}

pub fn get_all_servers(conn: &mut SqliteConnection) -> QueryResult<Vec<Server>> {
    servers::table.load::<Server>(conn)
}

pub fn update_status(conn: &mut SqliteConnection, s_id: i32, new_status: &str) {
    diesel::update(servers::table.filter(servers::id.eq(s_id)))
        .set(servers::status.eq(new_status))
        .execute(conn).expect("Chyba pri zmene stavu");
}

pub fn remove_server(conn: &mut SqliteConnection, target_id: i32) -> QueryResult<usize> {
    diesel::delete(servers::table.filter(servers::id.eq(target_id)))
        .execute(conn)
}