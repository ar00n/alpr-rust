-- Add migration script here
CREATE TABLE custom_actions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    method TEXT NOT NULL CHECK(method IN ('GET', 'POST', 'PUT', 'DELETE', 'PATCH')),
    url TEXT NOT NULL,
    auth_type TEXT NOT NULL CHECK(auth_type IN ('NONE', 'BASIC', 'BEARER', 'API_KEY')),
    auth_data TEXT, -- Store ENCRYPTED JSON containing secrets (username, password, or token)
    headers TEXT, -- Store JSON map of custom headers
    body_template TEXT, -- e.g., '{"plate": "${LICENCE_PLATE}"}'
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);