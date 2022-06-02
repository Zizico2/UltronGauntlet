#[derive(Debug, Default)]
pub(crate) struct Course {
    pub(crate) code: Option<Code>,
    pub(crate) name: Option<Name>,
}

#[derive(Debug, Default)]
pub(crate) struct Code(String);

impl From<&str> for Code {
    fn from(value: &str) -> Self {
        Code(value.into())
    }
}

#[derive(Debug, Default)]
pub(crate) struct Name(String);

impl From<&str> for Name {
    fn from(value: &str) -> Self {
        Name(value.into())
    }
}

impl From<Code> for String {
    fn from(val: Code) -> Self {
        val.0
    }
}