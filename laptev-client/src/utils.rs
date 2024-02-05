use rand::{rngs::StdRng, RngCore, SeedableRng};
use std::time::{Duration, SystemTime};

use iced::{
    color,
    Theme,
    widget::{horizontal_rule, rule}
};

#[allow(dead_code)]
pub fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

pub fn get_rng() -> StdRng {
    rand::rngs::StdRng::from_entropy()
}

pub fn rng_fill_bytes(bytes: &mut [u8]) {
    get_rng().fill_bytes(bytes);
}

pub fn invisible_rule() -> iced::widget::Rule {
    horizontal_rule(1)
        .style(iced::theme::Rule::Custom(Box::new(InvisibleHorizontalRuleCustomStyle)))
}

struct InvisibleHorizontalRuleCustomStyle;

impl rule::StyleSheet for InvisibleHorizontalRuleCustomStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> rule::Appearance {
        rule::Appearance { color: color!(229, 241, 237), width: 1, radius: 0.0.into(), fill_mode: rule::FillMode::Full }
    }
}

