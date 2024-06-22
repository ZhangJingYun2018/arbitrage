use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct TradeError;

impl fmt::Display for TradeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "不可交易")
    }
}

impl Error for TradeError {}
