-- One row per check of a service, written every 5 minutes by the Worker's cron (worker/src/status.rs) and read by /api/status.
-- Rows older than 90 days are deleted by the cron.
CREATE TABLE IF NOT EXISTS status_checks (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  service TEXT NOT NULL CHECK (service IN ('api', 'warframestat')),
  checked_at INTEGER NOT NULL,
  ok INTEGER NOT NULL CHECK (ok IN (0, 1)),
  latency_ms INTEGER NOT NULL,
  detail TEXT
);
CREATE INDEX IF NOT EXISTS status_checks_service_time ON status_checks (service, checked_at);
