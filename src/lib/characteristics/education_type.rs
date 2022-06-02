#[derive(Debug, Default)]
pub(crate) struct EducationType(String);

impl From<&str> for EducationType {
    fn from(value: &str) -> Self {
        EducationType(value.into())
    }
}