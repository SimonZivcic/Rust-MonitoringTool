CREATE TABLE IF NOT EXISTS servers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT '/', -- Stav: ON, OFF, / (Unknown)
    port INTEGER NOT NULL,            -- Tooltip info
    cpu_model TEXT NOT NULL,         -- Tooltip info
    max_ram REAL NOT NULL            -- Celková RAM
);

CREATE TABLE IF NOT EXISTS history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    server_id INTEGER NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    response_ms INTEGER NOT NULL,    -- Náhodná odozva
    ram_usage REAL NOT NULL,         -- Náhodná RAM (0 ak OFF)
    cpu_usage REAL NOT NULL,         -- Náhodné CPU (0 ak OFF)
    FOREIGN KEY(server_id) REFERENCES servers(id)
);