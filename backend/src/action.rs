#[derive(Debug, Clone, Copy)]
pub enum Action {
    Fold,
    Check,
    Call,
    Raise(i32),
}