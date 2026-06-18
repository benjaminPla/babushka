pub struct PricingType(String);

impl PricingType {
    pub fn new(value: impl Into<String>) -> Result<Self, PricingTypeError> {
        let s = value.into().trim().to_owned();
        match s.as_str() {
            "monthly" | "class" => Ok(Self(s)),
            _ => Err(PricingTypeError::Invalid),
        }
    }

    pub fn value(&self) -> &str { &self.0 }
    pub fn is_monthly(&self) -> bool { self.0 == "monthly" }
}

#[derive(Debug, thiserror::Error)]
pub enum PricingTypeError {
    #[error("tipo de precio inválido: debe ser monthly o class")]
    Invalid,
}
