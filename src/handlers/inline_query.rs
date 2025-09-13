use rusqlite::Connection;
use teloxide_core::{
    Bot,
    types::ChatType,
};


pub fn handle(conn: &mut Connection, bot: &Bot, inline_query: teloxide_core::types::InlineQuery) {
    let Some(chat_type) = inline_query.chat_type.as_ref() else {
        return;
    };
    match chat_type {
        ChatType::Channel | ChatType::Group | ChatType::Supergroup => {
            // do nothing
        }
        _ => {
            // ignore other chat types
        }
    }
    // todo
}
