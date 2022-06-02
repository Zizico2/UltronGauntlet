#[derive(Debug, Default)]
pub(crate) struct Ects(u16);

impl From<u16> for Ects {
    fn from(value: u16) -> Self {
        Ects(value)
    }
}
