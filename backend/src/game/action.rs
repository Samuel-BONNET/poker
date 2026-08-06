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
    pub fn from_request(action_type: &str, value: Option<i32>) -> Result<Action, String> {
        match action_type {
            "fold" => Ok(Action::Fold),
            "check" => Ok(Action::Check),
            "call" => Ok(Action::Call),
            "raise" => Ok(Action::Raise(value.unwrap_or(0))),
            "all-in" => Ok(Action::AllIn),
            _ => Err(format!("Unknown action: {action_type}")),
        }
    }
}