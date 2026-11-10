CREATE TABLE IF NOT EXISTS feedback (
  id TEXT PRIMARY KEY,
  category TEXT NOT NULL CHECK (category IN ('bug', 'suggestion', 'other')),
  title TEXT NOT NULL CHECK (length(title) BETWEEN 3 AND 120),
  message TEXT NOT NULL CHECK (length(message) BETWEEN 10 AND 4000),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE TABLE IF NOT EXISTS feedback_rate_limits (
  client_key TEXT PRIMARY KEY,
  window_start INTEGER NOT NULL,
  count INTEGER NOT NULL
);
