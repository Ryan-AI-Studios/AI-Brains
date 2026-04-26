-- Migration 0008: FTS Triggers
-- Keep FTS in sync with memory_projection

CREATE TRIGGER memory_fts_ai AFTER INSERT ON memory_projection BEGIN
  INSERT INTO memory_fts(rowid, content, memory_id) VALUES (new.rowid, new.content, new.memory_id);
END;

CREATE TRIGGER memory_fts_ad AFTER DELETE ON memory_projection BEGIN
  INSERT INTO memory_fts(memory_fts, rowid, content, memory_id) VALUES('delete', old.rowid, old.content, old.memory_id);
END;

CREATE TRIGGER memory_fts_au AFTER UPDATE ON memory_projection BEGIN
  INSERT INTO memory_fts(memory_fts, rowid, content, memory_id) VALUES('delete', old.rowid, old.content, old.memory_id);
  INSERT INTO memory_fts(rowid, content, memory_id) VALUES (new.rowid, new.content, new.memory_id);
END;
