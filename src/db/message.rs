use jieba_rs::Jieba;
use rusqlite::{Connection, params};

#[derive(Debug)]
pub struct Message {
    pub user_id: u64,
    pub group_id: i64,
    pub message_id: i32,
    pub text: String,
    pub score: f32,
}

impl Message {
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
            "INSERT INTO messages (text, user_id, message_id, group_id) VALUES (?1, ?2, ?3, ?4)",
            params![text, self.user_id, self.message_id, self.group_id],
        )
        .unwrap();

        tx.execute(
            "INSERT INTO messages_fts (rowid, text) VALUES (last_insert_rowid(), ?1)",
            params![words],
        )
        .unwrap();

        tx.commit().unwrap();

        Ok(())
    }

    pub fn query(
        conn: &mut Connection,
        jieba: &Jieba,
        group_id: i64,
        query: &str,
        limit: usize,
    ) -> rusqlite::Result<Vec<Message>> {
        let query_str = jieba
            .cut(query, false)
            .into_iter()
            .filter(|x| !x.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        let mut stmt = conn.prepare(
            "SELECT m.text, m.user_id, m.message_id, m.group_id, bm25(messages_fts) as score
                    FROM messages m
                    JOIN messages_fts f ON m.id = f.rowid
                    WHERE f.text MATCH ?1
                      AND m.group_id = ?2
                    ORDER BY score ASC
                    LIMIT ?3;",
        )?;

        let rows = stmt.query_map(params![query_str, group_id, limit], |row| {
            Ok(Message {
                text: row.get(0)?,
                user_id: row.get(1)?,
                message_id: row.get(2)?,
                group_id: row.get(3)?,
                score: row.get(4)?,
            })
        })?;

        rows.into_iter().collect()
    }

    pub fn to_message_url(&self) -> String {
        format!(
            "https://t.me/c/{}/{}",
            self.group_id.abs() - 1000000000000,
            self.message_id
        )
    }

    fn escape_html(str: &str) -> String {
        let mut ret = String::new();
        ret.reserve(str.len() * 2);
        for ch in str.chars() {
            match ch {
                '<' => ret.push_str("&lt;"),
                '>' => ret.push_str("&gt;"),
                '&' => ret.push_str("&amp;"),
                x => ret.push(x),
            }
        }
        ret
    }

    pub fn to_message_url_detailed(&self) -> String {
        let url = self.to_message_url();
        let idx = self.text.ceil_char_boundary(30);
        let text = Message::escape_html(&self.text[..idx].replace('\n', " "));
        let score = self.score;
        format!("<a href=\"{url}\">{text}</a> = {score:.2}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_url() {
        let msg = Message {
            user_id: 123,
            group_id: -1001234567890,
            message_id: 42,
            text: "Hello, world!".to_string(),
            score: 0.0,
        };
        assert_eq!(
            msg.to_message_url(),
            "https://t.me/c/1234567890/42".to_string()
        );
    }
}
