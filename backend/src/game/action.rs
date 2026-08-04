use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Action {
    Fold,
    Check,
    Call,
    Raise(i32),
    AllIn,
}

impl Action {
    pub fn from_request(action_type: &str, value: Option<i32>) -> Action {
        match action_type {
            "fold" => Action::Fold,
            "check" => Action::Check,
            "call" => Action::Call,
            "raise" => Action::Raise(value.unwrap_or(0)),
            "all-in" => Action::AllIn,
            _ => Action::Fold,
        }
    }
}