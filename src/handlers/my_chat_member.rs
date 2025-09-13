use teloxide_core::{
    Bot,
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{ChatMemberUpdated, ParseMode},
};

pub async fn handle(bot: &Bot, my_chat_member: ChatMemberUpdated) {
    let _ = bot
        .send_message(
            my_chat_member.chat.id,
            "我是一个可以用来搜索群消息的机器人。注意，消息被<b>明文</b>保存在数据库中。可以使用 /toggle_my_searchability 来切换是否记录你的消息。",
        )
        .parse_mode(ParseMode::Html)
        .await;
}
