CREATE TABLE plate_reads (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plate TEXT NOT NULL,
    confidence REAL NOT NULL,
    snapshot_image TEXT, -- Stores file path
    was_allowed BOOLEAN NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX idx_plate_reads_timestamp ON plate_reads (timestamp DESC);
CREATE INDEX idx_plate_reads_plate ON plate_reads (plate);

CREATE TABLE allow_list (
    plate TEXT PRIMARY KEY NOT NULL,
    expiry_date DATETIME -- NULL means it never expires
);

CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT 0
);

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);