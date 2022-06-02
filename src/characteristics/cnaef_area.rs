use std::fmt::{Display, Write};

#[derive(Debug, Default)]
pub(crate) struct CnaefArea {
    pub(crate) code: Option<Code>,
    pub(crate) name: Option<Name>,
}

/* CNAEF Area code */
#[derive(Debug, Default)]
pub(crate) struct Code(String);

impl From<&str> for Code {
    fn from(value: &str) -> Self {
        Code(value.into())
    }
}

impl From<String> for Code {
    fn from(value: String) -> Self {
        Code(value)
    }
}

impl From<Code> for String {
    fn from(value: Code) -> Self {
        value.0
    }
}
impl Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
/* ------------------- */

/* CNAEF Name code */
#[derive(Debug, Default)]
pub(crate) struct Name(String);

impl From<&str> for Name {
    fn from(value: &str) -> Self {
        Name(value.into())
    }
}

impl From<String> for Name {
    fn from(value: String) -> Self {
        Name(value)
    }
}

impl From<Name> for String {
    fn from(value: Name) -> Self {
        value.0
    }
}
impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
/* ------------------- */
