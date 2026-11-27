-- 0005 was edited after it had been applied (the `compare` service was added to the CHECK), so databases that ran the first
-- version still refuse `compare` rows. SQLite cannot change a CHECK: the table is made again with the right one.
CREATE TABLE status_checks_new (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  service TEXT NOT NULL CHECK (service IN ('api', 'warframestat', 'compare')),
  checked_at INTEGER NOT NULL,
  ok INTEGER NOT NULL CHECK (ok IN (0, 1)),
  latency_ms INTEGER NOT NULL,
  detail TEXT
);
INSERT INTO status_checks_new (id, service, checked_at, ok, latency_ms, detail) SELECT id, service, checked_at, ok, latency_ms, detail FROM status_checks;
DROP TABLE status_checks;
ALTER TABLE status_checks_new RENAME TO status_checks;
CREATE INDEX IF NOT EXISTS status_checks_service_time ON status_checks (service, checked_at);
