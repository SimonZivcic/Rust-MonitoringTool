CREATE TABLE IF NOT EXISTS servers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT '/', 
    port INTEGER NOT NULL,          
    cpu_model TEXT NOT NULL,         
    max_ram REAL NOT NULL           
);

CREATE TABLE IF NOT EXISTS history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    server_id INTEGER NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    response_ms INTEGER NOT NULL,    
    ram_usage REAL NOT NULL,         
    cpu_usage REAL NOT NULL,         
    FOREIGN KEY(server_id) REFERENCES servers(id)
);