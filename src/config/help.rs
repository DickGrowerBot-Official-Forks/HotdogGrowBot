use teloxide::types::Me;
use crate::config::env::get_env_mandatory_value;
use crate::domain::primitives::Username;
use crate::handlers::utils::Incrementor;
use crate::help;

pub fn build_context_for_help_messages(me: Me, incr: &Incrementor) -> anyhow::Result<help::Context> {
    let incr_cfg = incr.get_config();
    let admin_chat: Username = get_env_mandatory_value("HELP_ADMIN_CHAT")?;

    Ok(help::Context {
        bot_name: Username::from(me.username()),
        grow_min: incr_cfg.growth_range_min().to_string(),
        grow_max: incr_cfg.growth_range_max().to_string(),
        admin_website: get_env_mandatory_value("HELP_ADMIN_WEBSITE")?,
        admin_chat: admin_chat.value_with_at_sign(),
        git_repo: get_env_mandatory_value("HELP_GIT_REPO")?,
    })
}
