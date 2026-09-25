use teloxide::Bot;
use teloxide::macros::BotCommands;
use teloxide::types::Message;
use crate::handlers::{HandlerResult, reply_html};
use crate::{metrics, reply_html};
use crate::domain::primitives::{LanguageCode, Username};
use crate::help::HelpContainer;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum StartCommands {
    Start,
}

pub async fn start_cmd_handler(bot: Bot, msg: Message, help: HelpContainer) -> HandlerResult {
    metrics::CMD_START_COUNTER.inc();
    let lang_code = LanguageCode::from_maybe_user(msg.from.as_ref());
    let answer = if msg.from.as_ref().is_none() {
        log::warn!("The /start command was invoked without a FROM field for message: {:?}", msg);
        help.get_help_message(lang_code).to_owned()
    } else {
        let username = Username::new(msg.from.as_ref().unwrap().first_name.clone());
        help.get_start_message(username, lang_code)
    };
    reply_html!(bot, msg, answer);
    Ok(())
}
