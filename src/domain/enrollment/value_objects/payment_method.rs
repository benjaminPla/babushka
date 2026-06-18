pub struct PaymentMethod(String);

impl PaymentMethod {
    pub fn new(value: impl Into<String>) -> Result<Self, PaymentMethodError> {
        let s = value.into().trim().to_owned();
        match s.as_str() {
            "cash" | "transfer" | "card" => Ok(Self(s)),
            _ => Err(PaymentMethodError::Invalid),
        }
    }

    pub fn value(&self) -> &str { &self.0 }
}

#[derive(Debug, thiserror::Error)]
pub enum PaymentMethodError {
    #[error("método de pago inválido: debe ser cash, transfer o card")]
    Invalid,
}
