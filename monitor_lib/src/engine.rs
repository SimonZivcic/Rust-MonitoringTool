use rand::Rng;
use crate::models::Server;

pub async fn simulate_server_metrics(server: &Server) -> (i32, f32, f32) {
    let mut rng = rand::rng(); // OPRAVENÉ: thread_rng() -> rng()

    if server.status == "OFF" || server.status == "/" {
        return (0, 0.0, 0.0);
    }

    let ms = rng.random_range(1..200);        // OPRAVENÉ: gen_range -> random_range
    let cpu = rng.random_range(0.1..100.0);
    let ram = rng.random_range(0.1..server.max_ram);

    (ms, cpu, ram)
}