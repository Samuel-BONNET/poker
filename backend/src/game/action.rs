use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Action {
    Fold,
    Check,
    Call,
    Raise(i32),
}