CREATE TABLE frames (
    id      TEXT PRIMARY KEY,
    project TEXT NOT NULL,
    start   TEXT NOT NULL,
    end     TEXT NOT NULL
);

CREATE TABLE frame_tags (
    frame_id TEXT    NOT NULL REFERENCES frames(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    tag      TEXT    NOT NULL,
    PRIMARY KEY (frame_id, position)
);

-- Single-row table enforced by CHECK (lock = 1)
CREATE TABLE active_frame (
    lock    INTEGER PRIMARY KEY DEFAULT 1 CHECK (lock = 1),
    project TEXT NOT NULL,
    start   TEXT NOT NULL
);

CREATE TABLE active_frame_tags (
    position INTEGER PRIMARY KEY,
    tag      TEXT NOT NULL
);
