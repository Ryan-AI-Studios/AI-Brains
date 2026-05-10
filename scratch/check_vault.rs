use rusqlite::Connection;
fn main() {
    let conn = Connection::open("C:\\dev\\ai-brains\\vault.db").unwrap();
    let mut stmt = conn.prepare("SELECT status, count(*) FROM session_projection GROUP BY status").unwrap();
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    }).unwrap();
    println!("Session Status Summary:");
    for row in rows {
        let (status, count) = row.unwrap();
        println!("  {}: {}", status, count);
    }

    let mut stmt = conn.prepare("SELECT count(*) FROM session_projection WHERE summary_memory_id IS NOT NULL").unwrap();
    let summarized_count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
    println!("Summarized Sessions: {}", summarized_count);
}
