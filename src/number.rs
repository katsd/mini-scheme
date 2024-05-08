#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Number {
    Int(i64),
    Float(f64),
}

impl From<i64> for Number {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<f64> for Number {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}
