use rusqlite::Connection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = "c:/Users/RyanB/.gemini/antigravity/vault.db";
    let conn = Connection::open(db_path)?;

    // We need to bypass the key if it's not encrypted, or use the dummy key.
    // Assuming the vault.db in the capture directory is not encrypted with the full SQLCipher logic for this check,
    // or we use the dummy key.

    let mut stmt =
        conn.prepare("SELECT session_id, summary FROM session_projection WHERE summary IS NULL")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

    println!("Unsummarized sessions:");
    for row in rows {
        println!("{}", row?);
    }

    Ok(())
}
