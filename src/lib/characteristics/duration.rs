#[derive(Debug, Default)]
pub(crate) struct Duration {
    pub(crate) ammount: Option<Ammount>,
    pub(crate) unit: Option<Unit>,
}

#[derive(Debug, Default)]
pub(crate) struct Unit(String);

impl From<&str> for Unit {
    fn from(value: &str) -> Self {
        Unit(value.into())
    }
}

impl From<Unit> for String {
    fn from(value: Unit) -> Self {
        value.0
    }
}

#[derive(Debug, Default)]
pub(crate) struct Ammount(u8);

impl From<u8> for Ammount {
    fn from(value: u8) -> Self {
        Ammount(value)
    }
}

impl From<Ammount> for u8 {
    fn from(value: Ammount) -> Self {
        value.0
    }
}
