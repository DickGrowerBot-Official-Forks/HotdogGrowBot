use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use rust_i18n::t;
use teloxide::Bot;
use teloxide::macros::BotCommands;
use teloxide::payloads::{AnswerCallbackQuerySetters, SendMessageSetters};
use teloxide::requests::Requester;
use teloxide::types::{CallbackQuery, ChatMember, ChatMemberKind, InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::types::{Message, UserId};

use crate::domain::primitives::LanguageCode;
use crate::handlers::utils::callbacks::{CallbackAnswerParams, CallbackDataWithPrefix, InvalidCallbackData};
use crate::handlers::utils::callbacks::{InvalidCallbackDataBuilder, parse_part, prepare_callback_answer_params};
use crate::handlers::{HandlerResult, reply_html};
use crate::{metrics, reply_html, repo};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum ResetCommands {
    #[command(description = "reset")]
    Reset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, strum_macros::Display, strum_macros::EnumString)]
#[strum(serialize_all = "lowercase")]
enum ResetAction {
    Confirm,
    Cancel,
}

impl ResetAction {
    const fn button_label(&self) -> &'static str {
        match self {
            Self::Confirm => "commands.reset.buttons.confirm",
            Self::Cancel => "commands.reset.buttons.cancel",
        }
    }

    const fn result_label(&self) -> &'static str {
        match self {
            Self::Confirm => "commands.reset.success",
            Self::Cancel => "commands.reset.cancelled",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ResetCallbackData {
    uid: UserId,
    action: ResetAction,
}

impl Display for ResetCallbackData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.uid.0, self.action)
    }
}

impl CallbackDataWithPrefix for ResetCallbackData {
    fn prefix() -> &'static str {
        "reset"
    }
}

impl TryFrom<String> for ResetCallbackData {
    type Error = InvalidCallbackData;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let err = InvalidCallbackDataBuilder(&value);
        let mut parts = value.split(':');
        let uid = UserId(parse_part(&mut parts, &err, "uid")?);
        let action = parse_part(&mut parts, &err, "action")?;
        if parts.next().is_some() {
            return Err(err.split_err());
        }
        Ok(Self { uid, action })
    }
}

pub async fn cmd_handler(bot: Bot, msg: Message) -> HandlerResult {
    metrics::CMD_RESET.invoked();
    let lang_code = LanguageCode::from_maybe_user(msg.from.as_ref());
    let Some(from) = msg.from.as_ref().filter(|_| msg.sender_chat.is_none()) else {
        return reply_owner_only(bot, msg, &lang_code).await;
    };
    if !user_is_chat_owner(&bot, msg.chat.id, from.id).await? {
        return reply_owner_only(bot, msg, &lang_code).await;
    }

    let button = |action: ResetAction| InlineKeyboardButton::callback(
        t!(action.button_label(), locale = &lang_code).to_string(),
        ResetCallbackData { uid: from.id, action }.to_data_string(),
    );
    let keyboard = InlineKeyboardMarkup::new(vec![vec![
        button(ResetAction::Confirm),
        button(ResetAction::Cancel),
    ]]);
    reply_html(bot, &msg, t!("commands.reset.warning", locale = &lang_code))
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

fn owner_only(lang_code: &LanguageCode) -> Cow<'_, str> {
    t!("commands.reset.errors.owner_only", locale = lang_code)
}

async fn reply_owner_only(bot: Bot, msg: Message, lang_code: &LanguageCode) -> HandlerResult {
    reply_html!(bot, msg, owner_only(lang_code));
    Ok(())
}

pub fn callback_filter(query: CallbackQuery) -> bool {
    ResetCallbackData::check_prefix(query)
}

pub async fn callback_handler(bot: Bot, query: CallbackQuery, repos: repo::Repositories) -> HandlerResult {
    let data = ResetCallbackData::parse(&query)?;
    let (answer, lang_code) = match prepare_callback_answer_params(&bot, &query, data.uid).await? {
        CallbackAnswerParams::Answer { answer, lang_code } => (answer, lang_code),
        CallbackAnswerParams::AnotherUser => return Ok(()),
    };
    let message = query.message.as_ref()
        .ok_or(anyhow::anyhow!("a reset callback without an attached message"))?;
    let chat_id = message.chat().id;

    if data.action == ResetAction::Confirm && !user_is_chat_owner(&bot, chat_id, query.from.id).await? {
        answer
            .show_alert(true)
            .text(owner_only(&lang_code))
            .await?;
        return Ok(());
    }

    if data.action == ResetAction::Confirm {
        repos.dicks.reset_chat(&chat_id.into()).await?;
        metrics::CMD_RESET.finished();
    }
    bot.edit_message_text(chat_id, message.id(), t!(data.action.result_label(), locale = &lang_code).to_string()).await?;
    answer.await?;
    Ok(())
}

async fn user_is_chat_owner(bot: &Bot, chat_id: teloxide::types::ChatId,
                            user_id: UserId) -> Result<bool, teloxide::RequestError> {
    let administrators = bot.get_chat_administrators(chat_id).await?;
    Ok(is_owner(&administrators, user_id))
}

fn is_owner(administrators: &[ChatMember], user_id: UserId) -> bool {
    administrators.iter().any(|member| member.user.id == user_id && is_owner_kind(&member.kind))
}

fn is_owner_kind(kind: &ChatMemberKind) -> bool {
    matches!(kind, ChatMemberKind::Owner(_))
}

#[cfg(test)]
mod tests {
    use super::*;

    use teloxide::types::{Member, Owner};

    #[test]
    fn callback_data_roundtrip() {
        for action in [ResetAction::Confirm, ResetAction::Cancel] {
            let expected = ResetCallbackData { uid: UserId(12345), action };
            let encoded = expected.to_data_string();
            let parsed = encoded.strip_prefix("reset:").unwrap().to_owned().try_into().unwrap();
            assert_eq!(expected, parsed);
        }
    }

    #[test]
    fn callback_data_rejects_unknown_action() {
        assert!(ResetCallbackData::try_from("12345:erase".to_owned()).is_err());
    }

    #[test]
    fn only_owner_status_is_authorized() {
        let owner = ChatMemberKind::Owner(Owner { custom_title: None, is_anonymous: false });
        let member = ChatMemberKind::Member(Member { until_date: None });

        assert!(is_owner_kind(&owner));
        assert!(!is_owner_kind(&member));
        assert!(!is_owner_kind(&ChatMemberKind::Left));
    }
}
