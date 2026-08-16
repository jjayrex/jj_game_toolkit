#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEvent {
    NumEnter,
    NumDigit(u8),
    NumDecimal,
    Backspace,
}