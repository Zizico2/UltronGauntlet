#[derive(Debug, Default)]
pub(crate) struct Degree(String);

impl From<&str> for Degree {
    fn from(value: &str) -> Self {
        Degree(value.into())
    }
}