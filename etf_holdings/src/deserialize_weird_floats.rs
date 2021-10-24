// Custom deserializer for numbers with comma separators (e.g. 123,342.28)
use serde::{Deserialize, Deserializer};
use std::num::ParseFloatError;

pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    parse_weird_float(&String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
}

pub fn parse_weird_float(float_str: &str) -> Result<f64, ParseFloatError> {
    float_str.replace(",", "").parse()
}
