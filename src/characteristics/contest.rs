#[derive(Debug, Default)]
pub(crate) struct Contest(String);

impl From<&str> for Contest {
    fn from(value: &str) -> Self {
        Contest(value.into())
    }
}