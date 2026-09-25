use std::ops::RangeInclusive;
use num_traits::PrimInt;
use rand::distr::uniform::SampleUniform;
use rand::RngExt;
use crate::config;
use crate::domain::primitives::{DaysCount, LengthChange, Ratio, SignedLengthChange};

#[derive(Clone)]
pub struct Incrementor {
    config: Config,
}

#[derive(Clone)]
pub struct Config {
    growth_range: RangeInclusive<i16>,
    grow_shrink_ratio: Ratio,
    newcomers_grace_days: DaysCount,
    dod_bonus_range: RangeInclusive<u8>,
}

impl Config {
    pub fn growth_range_min(&self) -> i16 {
        self.growth_range.clone()
            .min()
            .unwrap_or(0)
    }

    pub fn growth_range_max(&self) -> i16 {
        self.growth_range.clone()
            .max()
            .unwrap_or(0)
    }
}

impl Incrementor {
    pub fn from_env() -> Self {
        let growth_range_min = config::get_env_value_or_default("GROWTH_MIN", -3);
        let growth_range_max = config::get_env_value_or_default("GROWTH_MAX", 12);
        let dod_max_bonus = config::get_env_value_or_default("GROWTH_DOD_BONUS_MAX", 5);

        Self {
            config: Config {
                growth_range: growth_range_min..=growth_range_max,
                grow_shrink_ratio: config::get_env_value_or_default("GROW_SHRINK_RATIO", Ratio::literal(0.5)),
                newcomers_grace_days: config::get_env_value_or_default("NEWCOMERS_GRACE_DAYS", DaysCount::new(7)),
                dod_bonus_range: 1..=dod_max_bonus,
            },
        }
    }

    pub fn get_config(&self) -> Config {
        self.config.clone()
    }

    pub fn growth_increment(&self, days_since_registration: DaysCount) -> LengthChange {
        let grow_shrink_ratio = if days_since_registration > self.config.newcomers_grace_days {
            self.config.grow_shrink_ratio
        } else {
            Ratio::literal(1.0)
        };
        let base = get_base_increment(self.config.growth_range.clone(), grow_shrink_ratio);
        SignedLengthChange::new(base.into()).into()
    }

    pub fn dod_increment(&self) -> LengthChange {
        let base = rand::rng().random_range(self.config.dod_bonus_range.clone());
        SignedLengthChange::new(base.into()).into()
    }
}

fn get_base_increment<T>(range: RangeInclusive<T>, sign_ratio: Ratio) -> T
where
    T: PrimInt + PartialOrd + SampleUniform + From<i8>
{
    let sign_ratio_percent = match (sign_ratio.value() * 100.0).round() as u32 {
        ..=0 => 0,
        100.. => 100,
        x => x
    };
    let mut rng = rand::rng();
    let zero = <T as From<i8>>::from(0);
    if range.start() > &zero {
        return rng.random_range(range)
    }
    let positive = rng.random_ratio(sign_ratio_percent, 100);
    if positive {
        let end = *range.end();
        let one = <T as From<i8>>::from(1);
        rng.random_range(one..=end)
    } else {
        let start = *range.start();
        let minus_one = <T as From<i8>>::from(-1);
        rng.random_range(start..=minus_one)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn incrementor() -> Incrementor {
        Incrementor {
            config: Config {
                growth_range: -3..=12,
                grow_shrink_ratio: Ratio::literal(0.5),
                newcomers_grace_days: DaysCount::new(7),
                dod_bonus_range: 1..=5,
            },
        }
    }

    #[test]
    fn growth_stays_in_configured_non_zero_range() {
        let incr = incrementor();
        let values: Vec<_> = (0..200).map(|_| incr.growth_increment(DaysCount::new(8)).value()).collect();
        assert!(values.iter().all(|value| (-3..=12).contains(value) && *value != 0));
        assert!(values.iter().any(|value| *value < 0));
        assert!(values.iter().any(|value| *value > 0));
    }

    #[test]
    fn newcomers_only_receive_positive_growth() {
        let incr = incrementor();
        assert!((0..200).all(|_| incr.growth_increment(DaysCount::new(7)).value() > 0));
    }

    #[test]
    fn dod_bonus_stays_in_configured_range() {
        let incr = incrementor();
        assert!((0..200).all(|_| (1..=5).contains(&incr.dod_increment().value())));
    }
}
