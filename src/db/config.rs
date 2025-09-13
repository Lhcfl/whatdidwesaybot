// user_id      INTEGER PRIMARY KEY,
// first_name   TEXT NOT NULL,
// last_name    TEXT,
// username     TEXT

use rusqlite::{Connection, params};

#[derive(Debug)]
pub struct Config {
    // group_id INTEGER NOT NULL,
    // user_id INTEGER NOT NULL,
    // allow INTEGER NOT NULL,
    pub group_id: i64,
    pub user_id: u64,
    pub allow: u64,
}

impl Config {
    pub fn insert_or_update(&self, conn: &mut Connection) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT INTO configs (group_id, user_id, allow)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(group_id, user_id) DO UPDATE SET
                allow = excluded.allow;",
            params![self.group_id, self.user_id, self.allow],
        )?;

        Ok(())
    }

    pub fn get_or_default(
        conn: &mut Connection,
        group_id: i64,
        user_id: u64,
    ) -> rusqlite::Result<Config> {
        let mut stmt = conn.prepare(
            "SELECT group_id, user_id, allow
            FROM configs
            WHERE group_id = ?1 AND user_id = ?2;",
        )?;

        let mut rows = stmt.query_map(params![group_id, user_id], |row| {
            Ok(Config {
                group_id: row.get(0)?,
                user_id: row.get(1)?,
                allow: row.get(2)?,
            })
        })?;

        if let Some(config) = rows.next() {
            return config;
        }

        Ok(Config {
            group_id,
            user_id,
            allow: 1,
        })
    }
}
