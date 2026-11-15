use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AmountError {
    #[error("amount must not be negative")]
    Negative,
    #[error("currency must be a three-letter uppercase ISO 4217 code")]
    InvalidCurrency,
}

/// An exact monetary amount in minor units (kopecks, cents, ...).
///
/// Floating point is never used for money.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "AmountRepr")]
pub struct Amount {
    minor_units: i64,
    currency: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AmountRepr {
    minor_units: i64,
    currency: String,
}

impl TryFrom<AmountRepr> for Amount {
    type Error = AmountError;

    fn try_from(value: AmountRepr) -> Result<Self, Self::Error> {
        Self::new(value.minor_units, value.currency)
    }
}

impl Amount {
    pub fn new(minor_units: i64, currency: impl Into<String>) -> Result<Self, AmountError> {
        if minor_units < 0 {
            return Err(AmountError::Negative);
        }
        let currency = currency.into();
        if !is_currency_code(&currency) {
            return Err(AmountError::InvalidCurrency);
        }
        Ok(Self {
            minor_units,
            currency,
        })
    }

    pub fn rub(minor_units: i64) -> Self {
        Self::new(minor_units, "RUB").expect("RUB is a valid currency")
    }

    pub fn minor_units(&self) -> i64 {
        self.minor_units
    }

    pub fn currency(&self) -> &str {
        &self.currency
    }

    pub fn major_units(&self) -> i64 {
        self.minor_units / 100
    }

    pub fn fraction(&self) -> i64 {
        self.minor_units % 100
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}.{:02} {}",
            self.major_units(),
            self.fraction(),
            self.currency
        )
    }
}

fn is_currency_code(currency: &str) -> bool {
    currency.len() == 3 && currency.bytes().all(|byte| byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_minor_units_without_floats() {
        assert_eq!(
            Amount::new(79_900, "RUB").unwrap().to_string(),
            "799.00 RUB"
        );
        assert_eq!(Amount::new(5, "USD").unwrap().to_string(), "0.05 USD");
    }

    #[test]
    fn rejects_invalid_amounts() {
        assert_eq!(Amount::new(-1, "RUB"), Err(AmountError::Negative));
        assert_eq!(Amount::new(1, "rub"), Err(AmountError::InvalidCurrency));
        assert_eq!(Amount::new(1, "RU"), Err(AmountError::InvalidCurrency));
        assert_eq!(Amount::new(1, "RUBLE"), Err(AmountError::InvalidCurrency));
    }

    #[test]
    fn serializes_as_exact_values() {
        let amount = Amount::rub(79_900);
        let json = serde_json::to_value(&amount).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "minor_units": 79900, "currency": "RUB" })
        );
        assert_eq!(serde_json::from_value::<Amount>(json).unwrap(), amount);
    }

    #[test]
    fn invalid_json_is_rejected() {
        let negative = serde_json::json!({ "minor_units": -1, "currency": "RUB" });
        assert!(serde_json::from_value::<Amount>(negative).is_err());

        let bad_currency = serde_json::json!({ "minor_units": 1, "currency": "rub" });
        assert!(serde_json::from_value::<Amount>(bad_currency).is_err());
    }
}
