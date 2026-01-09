use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::servers)]
pub struct Server {
    pub id: i32,
    pub name: String,
    pub status: String,
    pub port: i32,
    pub cpu_model: String,
    pub max_ram: f32,
}