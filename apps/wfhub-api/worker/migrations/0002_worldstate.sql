-- The raw game state of each platform (an earlier design; dropped by 0003).
CREATE TABLE IF NOT EXISTS worldstate (
  platform TEXT PRIMARY KEY CHECK (platform IN ('pc', 'xb1', 'ps4', 'swi')),
  state_time INTEGER NOT NULL,
  fetched_at INTEGER NOT NULL,
  body TEXT NOT NULL
);
