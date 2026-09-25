use reqwest::Url;
use crate::config::env::*;
use crate::config::toggles::*;
use crate::domain::primitives::{Bet, DaysCount, Limit, Ratio};

#[derive(Clone)]
#[cfg_attr(test, derive(Default))]
pub struct AppConfig {
    pub features: FeatureToggles,
    pub top_limit: Limit,
    pub inactivity_days: DaysCount,
    pub dod_rich_exclusion_ratio: Option<Ratio>,
    pub pvp_default_bet: Bet,
    pub command_toggles: CachedEnvToggles,
}

#[derive(Clone)]
pub struct DatabaseConfig {
    pub url: Url,
    pub max_connections: u32
}

impl AppConfig {
    pub fn from_env() -> Self {
        let top_limit = get_env_value_or_default("TOP_LIMIT", Limit::literal(10));
        let inactivity_days = get_env_value_or_default("INACTIVITY_DAYS", DaysCount::new(7));
        let dod_selection_mode = get_optional_env_value("DOD_SELECTION_MODE");
        let dod_rich_exclusion_ratio = get_optional_env_ratio("DOD_RICH_EXCLUSION_RATIO");
        let chats_merging = get_env_value_or_default("CHATS_MERGING_ENABLED", false);
        let pvp_default_bet = get_env_value_or_default("PVP_DEFAULT_BET", Bet::literal(1));
        let check_acceptor_length = get_env_value_or_default("PVP_CHECK_ACCEPTOR_LENGTH", false);
        Self {
            features: FeatureToggles {
                chats_merging,
                dod_selection_mode,
                pvp: BattlesFeatureToggles {
                    check_acceptor_length,
                }
            },
            top_limit,
            inactivity_days,
            dod_rich_exclusion_ratio,
            pvp_default_bet,
            command_toggles: Default::default(),
        }
    }
}

impl DatabaseConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            url: get_env_mandatory_value("DATABASE_URL")?,
            max_connections: get_env_value_or_default("DATABASE_MAX_CONNECTIONS", 10)
        })
    }
}
