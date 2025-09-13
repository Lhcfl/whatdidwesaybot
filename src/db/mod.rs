use rusqlite::Connection;

pub mod config;
pub mod global_message;
pub mod message;
pub mod user;

pub fn init_a_db(conn: &mut Connection) -> rusqlite::Result<()> {
    println!("初始化数据库...");
    conn.execute_batch(include_str!("../assets/initdb.sql"))?;
    Ok(())
}
