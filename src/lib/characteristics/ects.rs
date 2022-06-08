#[derive(Debug, Default)]
pub(crate) struct Ects(u16);

impl From<u16> for Ects {
    fn from(value: u16) -> Self {
        Ects(value)
    }
}

impl From<Ects> for u16 {
    fn from(val: Ects) -> Self {
        val.0
    }
}
