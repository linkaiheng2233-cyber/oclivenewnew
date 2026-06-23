CREATE TABLE IF NOT EXISTS role_scene_identity (
    role_id TEXT NOT NULL,
    scene_id TEXT NOT NULL,
    user_relation TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (role_id, scene_id),
    FOREIGN KEY (role_id) REFERENCES role_runtime(role_id) ON DELETE CASCADE
);
