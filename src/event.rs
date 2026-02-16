#[derive(Debug, PartialEq)]
pub enum Event {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Quit,
    Restart,
}
