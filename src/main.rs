use jieba_rs::Jieba;
use rusqlite::Connection;
use teloxide_core::{
    Bot,
    payloads::{GetUpdatesSetters, SendMessageSetters},
    prelude::{Request, Requester},
    types::{AllowedUpdate, BotCommand, ParseMode, UpdateKind},
};
use tokio::signal;
use tokio_util::sync::CancellationToken;

use crate::db::{config::Config, init_a_db, message::Message, user::User};
mod db;
mod handlers;

pub static ALLOWED_UPDATES: &[AllowedUpdate] = &[
    AllowedUpdate::Message,
    AllowedUpdate::MyChatMember,
    AllowedUpdate::InlineQuery,
];

async fn main_loop(cancel_token: CancellationToken) {
    println!("Initializing Database...");
    let mut conn = Connection::open("data.db").expect("Failed to open database");
    init_a_db(&mut conn).expect("Failed to initialize database");

    println!("Loading Jieba...");
    let jieba = Jieba::new();

    println!("Initializing Bot...");
    let bot = Bot::from_env();

    println!("Checking Network...");
    bot.set_my_commands(vec![
        BotCommand::new("q", "搜索消息"),
        BotCommand::new("toggle_my_searchability", "切换我的消息是否可被搜索"),
    ])
    .send()
    .await
    .unwrap();

    println!("Bot initialized.");

    let mut offset: i32 = 0;
    loop {
        let pms = bot
            .get_updates()
            .offset(offset)
            .timeout(10)
            .allowed_updates(ALLOWED_UPDATES.to_vec())
            .send();

        let res = tokio::select! {
            _ = cancel_token.cancelled() => {
                return;
            }
            res = pms => res
        };

        match res {
            Ok(updates) => {
                offset = updates.last().map(|u| u.id.0 as i32 + 1).unwrap_or(offset);

                for update in updates {
                    match update.kind {
                        UpdateKind::Message(msg) => {
                            handlers::message::handle(&mut conn, &jieba, &bot, &msg).await;
                        }
                        UpdateKind::MyChatMember(my_chat_member) => {
                            handlers::my_chat_member::handle(&bot, my_chat_member).await;
                        }
                        UpdateKind::InlineQuery(inline_query) => {
                            handlers::inline_query::handle(&mut conn, &bot, inline_query);
                        }
                        _ => {
                            // ignore other updates
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("[warn] GetUpdate Error: {}", err);
            }
        }
    }
}

async fn wait_for_ctrlc(cancel_token: CancellationToken) {
    signal::ctrl_c().await.expect("Failed to listen for Ctrl-C");
    println!(); // Print a newline to separate the Ctrl-C message from the previous output
    println!("Ctrl-C received, shutting down...");
    cancel_token.cancel();
}

#[tokio::main]
async fn main() {
    let cancel_token = CancellationToken::new();
    tokio::spawn(wait_for_ctrlc(cancel_token.clone()));
    main_loop(cancel_token.clone()).await;

    println!("bye bye");
}
