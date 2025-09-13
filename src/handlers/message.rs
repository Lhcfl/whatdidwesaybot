use jieba_rs::Jieba;
use rusqlite::Connection;
use teloxide_core::{
    Bot,
    payloads::{GetUpdatesSetters, SendMessageSetters},
    prelude::{Request, Requester},
    types::{AllowedUpdate, BotCommand, ParseMode, UpdateKind},
};

use crate::db::{config::Config, message::Message, user::User};

pub async fn handle(
    conn: &mut Connection,
    jieba: &Jieba,
    bot: &Bot,
    msg: &teloxide_core::types::Message,
) {
    if msg.chat.is_private() {
        // we don't handle private chat
        return;
    }
    let Some(from) = msg.from.as_ref() else {
        return;
    };
    let Some(text) = msg.text() else {
        return;
    };

    // cache the user
    if let Err(err) = (User {
        id: from.id.0,
        first_name: from.first_name.clone(),
        last_name: from.last_name.clone(),
        username: from.username.clone(),
    })
    .insert_or_update(conn)
    {
        eprintln!("[ERROR] Failed to insert or update user {from:?}: {err}");
    }

    let mut cfg = Config::get_or_default(conn, msg.chat.id.0, from.id.0).unwrap();
    if text.starts_with("/") {
        if text.starts_with("/q ") {
            let query = text[3..].trim();

            match Message::query(conn, jieba, msg.chat.id.0, query, 20) {
                Ok(results) => {
                    if results.is_empty() {
                        if let Err(e) = bot.send_message(msg.chat.id, "未找到相关消息。").await
                        {
                            eprintln!("[ERROR] Failed to send message: {}", e);
                        }
                    } else {
                        let response = results
                            .into_iter()
                            .map(|m| m.to_message_url_detailed())
                            .collect::<Vec<_>>()
                            .join("\n");

                        if let Err(e) = bot
                            .send_message(msg.chat.id, response)
                            .parse_mode(ParseMode::Html)
                            .await
                        {
                            eprintln!("[ERROR] Failed to send message: {}", e);
                        }
                    }
                }
                Err(e) => {
                    let _ = bot
                        .send_message(msg.chat.id, "发生错误，无法搜索消息。请检查日志。")
                        .await;
                    panic!("[ERROR] Failed to query messages: {}", e);
                }
            }
        } else if text == "/toggle_my_searchability" {
            cfg.allow = 1 - cfg.allow;
            if let Err(e) = cfg.insert_or_update(conn) {
                eprintln!(
                    "[ERROR] Failed to update config for user {} in group {}: {}",
                    from.id.0, msg.chat.id.0, e
                );
                if let Err(e) = bot
                    .send_message(msg.chat.id, "发生错误，设置未被保存。请检查日志。")
                    .await
                {
                    eprintln!("[ERROR] Failed to send message: {}", e);
                }
            }
            let status = if cfg.allow == 1 {
                "已启用"
            } else {
                "已禁用"
            };
            if let Err(e) = bot
                .send_message(msg.chat.id, format!("您的消息记录 {}", status))
                .await
            {
                eprintln!("[ERROR] Failed to send message: {}", e);
            }
        }

        return;
    }

    if cfg.allow == 0 {
        // user has disabled message logging
        return;
    }

    // handle normal message
    let message = Message {
        text: text.to_string(),
        user_id: from.id.0,
        group_id: msg.chat.id.0,
        message_id: msg.id.0,
        score: 0.0,
    };
    if let Err(e) = message.insert(conn, jieba) {
        bot.send_message(msg.chat.id, "发生错误，消息未被记录。请检查日志。")
            .await
            .unwrap();
        panic!("[ERROR] Failed to insert message: {}", e);
    }
}
