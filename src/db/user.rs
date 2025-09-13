// user_id      INTEGER PRIMARY KEY,
// first_name   TEXT NOT NULL,
// last_name    TEXT,
// username     TEXT

use rusqlite::{Connection, params};

#[derive(Debug)]
pub struct User {
    pub id: u64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
}

impl User {
    pub fn insert_or_update(&self, conn: &mut Connection) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT INTO users (id, first_name, last_name, username)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(id) DO UPDATE SET
                first_name = excluded.first_name,
                last_name = excluded.last_name,
                username = excluded.username;",
            params![self.id, self.first_name, self.last_name, self.username],
        )?;

        Ok(())
    }

    pub fn get_by_id(conn: &mut Connection, user_id: u64) -> rusqlite::Result<Option<User>> {
        let mut stmt = conn.prepare(
            "SELECT id, first_name, last_name, username
            FROM users
            WHERE id = ?1;",
        )?;

        let mut rows = stmt.query_map(params![user_id], |row| {
            Ok(User {
                id: row.get(0)?,
                first_name: row.get(1)?,
                last_name: row.get(2)?,
                username: row.get(3)?,
            })
        })?;

        if let Some(user) = rows.next() {
            return Ok(Some(user?));
        }

        Ok(None)
    }
}
