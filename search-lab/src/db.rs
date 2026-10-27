use rusqlite::{Connection, params};
use std::{error::Error, fs};

#[derive(Debug, Clone)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub url: String,
    pub difficulty: String,
    pub access_type: String,
    pub description: String,
    pub categories: Vec<String>,
}

pub fn open() -> Result<Connection, Box<dyn Error>> {
    fs::create_dir_all("data")?;

    let conn = Connection::open("data/lets-search.db")?;

    conn.execute_batch(
        "PRAGMA journal_mode = WAL;

         CREATE TABLE IF NOT EXISTS rooms (
             id TEXT PRIMARY KEY NOT NULL,
             name TEXT NOT NULL,
             url TEXT NOT NULL,
             difficulty TEXT NOT NULL DEFAULT 'Unknown',
             access_type TEXT NOT NULL DEFAULT 'Unknown',
             description TEXT NOT NULL DEFAULT '',
             categories TEXT NOT NULL DEFAULT '[]'
         );",
    )?;

    // Migrate existing databases without deleting saved rooms.
    let columns: Vec<String> = {
        let mut stmt = conn.prepare("PRAGMA table_info(rooms)")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    if !columns.iter().any(|c| c == "access_type") {
        conn.execute(
            "ALTER TABLE rooms ADD COLUMN access_type TEXT NOT NULL DEFAULT 'Unknown'",
            [],
        )?;
    }

    if !columns.iter().any(|c| c == "description") {
        conn.execute(
            "ALTER TABLE rooms ADD COLUMN description TEXT NOT NULL DEFAULT ''",
            [],
        )?;
    }

    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_rooms_name ON rooms(name);
         CREATE INDEX IF NOT EXISTS idx_rooms_access ON rooms(access_type);",
    )?;

    Ok(conn)
}

pub fn save(conn: &Connection, room: &Room) -> Result<(), Box<dyn Error>> {
    let categories = serde_json::to_string(&room.categories)?;

    conn.execute(
        "INSERT INTO rooms
            (id, name, url, difficulty, access_type, description, categories)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            url = excluded.url,
            difficulty = excluded.difficulty,
            access_type = excluded.access_type,
            description = excluded.description,
            categories = excluded.categories",
        params![
            room.id,
            room.name,
            room.url,
            room.difficulty,
            room.access_type,
            room.description,
            categories
        ],
    )?;

    Ok(())
}

pub fn search(
    conn: &Connection,
    query: &str,
    access: Option<&str>,
) -> Result<Vec<Room>, Box<dyn Error>> {
    let pattern = format!("%{}%", query);

    let mut stmt = conn.prepare(
        "SELECT id, name, url, difficulty, access_type, description, categories
         FROM rooms
         WHERE (?1 = ''
             OR name LIKE ?2 COLLATE NOCASE
             OR description LIKE ?2 COLLATE NOCASE
             OR categories LIKE ?2 COLLATE NOCASE)
           AND (?3 IS NULL OR lower(access_type) = lower(?3))
         ORDER BY name COLLATE NOCASE",
    )?;

    let rows = stmt.query_map(params![query, pattern, access], |row| {
        let categories_json: String = row.get(6)?;

        Ok(Room {
            id: row.get(0)?,
            name: row.get(1)?,
            url: row.get(2)?,
            difficulty: row.get(3)?,
            access_type: row.get(4)?,
            description: row.get(5)?,
            categories: serde_json::from_str(&categories_json).unwrap_or_default(),
        })
    })?;

    let mut rooms = Vec::new();
    for row in rows {
        rooms.push(row?);
    }

    Ok(rooms)
}

pub fn count(conn: &Connection) -> Result<i64, Box<dyn Error>> {
    Ok(conn.query_row("SELECT COUNT(*) FROM rooms", [], |row| row.get(0))?)
}
