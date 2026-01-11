use rand::Rng;
use crate::models::Server;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn simulate_server_metrics(server: &Server) -> (i32, f32, f32) {
    if matches!(server.status.as_str(), "OFF" | "/" | "STOPPING") {
        return (0, 0.0, 0.0);
    }

    let mut rng = rand::rng();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as f64;

    let offset = (server.id as f64) * 1337.42;

    let base_ms = match server.port {
        80 | 443 => 15.0,
        27017    => 5.0,
        _        => 40.0,
    };
    
    let ms = (base_ms + rng.random_range(-0.5..0.5)) as i32;

    let (eff_mult, base_load) = match server.cpu_model.to_lowercase() {
        m if m.contains("i9") || m.contains("ryzen 9") => (0.4, 15.0), 
        m if m.contains("i7") || m.contains("ryzen 7") => (0.7, 25.0),
        _ => (1.0, 35.0), 
    };

    let cpu_wave = ((now + offset) / 5000.0).sin(); 
    let cpu_jitter = ((now + offset) / 1000.0).cos() * 0.5;
    let cpu = (base_load + (cpu_wave * 15.0 * eff_mult) + cpu_jitter).clamp(1.0, 99.0) as f32;

    let ram_offset = (server.id as f64) * 9876.54;
    let ram_wave = ((now + ram_offset) / 12000.0).sin(); 
    let server_base_ram = 0.2 + ((server.id % 5) as f32 * 0.1); 
    let ram_percent = (server_base_ram + (ram_wave as f32 * 0.15)).clamp(0.1, 0.9);
    let ram = (server.max_ram * ram_percent).clamp(0.1, server.max_ram);

    (if server.port == 0 { -1 } else { ms.max(1) }, cpu, ram)
}