use jieba_rs::Jieba;
use rusqlite::{Connection, params};

#[derive(Debug)]
pub struct GlobalMessage {
    pub user_id: u64,
    pub text: String,
}

impl GlobalMessage {
    pub fn insert(&self, conn: &mut Connection, jieba: &Jieba) -> rusqlite::Result<()> {
        let text = self.text.trim();
        if text.is_empty() {
            return Ok(());
        }

        let words = jieba
            .cut_for_search(text, false)
            .into_iter()
            .filter(|x| !x.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        let tx = conn.transaction().unwrap();

        tx.execute(
            "INSERT INTO global_messages (text, user_id) VALUES (?1, ?2)",
            params![text, self.user_id],
        )
        .unwrap();

        tx.execute(
            "INSERT INTO global_messages_fts (rowid, text) VALUES (last_insert_rowid(), ?1)",
            params![words],
        )
        .unwrap();

        tx.commit().unwrap();

        Ok(())
    }

    pub fn query(
        conn: &mut Connection,
        jieba: &Jieba,
        query: &str,
        limit: usize,
    ) -> rusqlite::Result<Vec<GlobalMessage>> {
        let query_str = jieba
            .cut(query, false)
            .into_iter()
            .filter(|x| !x.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        let mut stmt = conn.prepare(
            "SELECT m.text, m.user_id
                    FROM global_messages m
                    JOIN global_messages_fts f ON m.id = f.rowid
                    WHERE f.text MATCH ?1
                    ORDER BY bm25(global_messages_fts) ASC
                    LIMIT ?2;",
        )?;

        let rows = stmt.query_map(params![query_str, limit], |row| {
            Ok(GlobalMessage {
                text: row.get(0)?,
                user_id: row.get(1)?,
            })
        })?;

        rows.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::db::init_a_db;

    use super::*;
    use jieba_rs::Jieba;
    use rusqlite::Connection;

    #[test]
    fn test_insert_and_query() {
        let jieba = Jieba::new();
        let mut conn = Connection::open_in_memory().unwrap();

        init_a_db(&mut conn).unwrap();

        let texts = ["喵喵喵", "汪汪汪", "你说得对", "啦啦啦"];

        for text in texts {
            let msg = GlobalMessage {
                user_id: 1,
                text: text.to_string(),
            };

            msg.insert(&mut conn, &jieba).unwrap();
        }

        let results = GlobalMessage::query(&mut conn, &jieba, "喵", 10).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "喵喵喵");

        let results = GlobalMessage::query(&mut conn, &jieba, "汪", 10).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "汪汪汪");

        let results = GlobalMessage::query(&mut conn, &jieba, "啦", 10).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "啦啦啦");

        let results = GlobalMessage::query(&mut conn, &jieba, "你", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "你说得对");
    }
}
