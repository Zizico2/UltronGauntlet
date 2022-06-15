#[derive(Debug, Default)]
pub(crate) struct Institution {
    pub(crate) code: Option<Code>,
    pub(crate) name: Option<Name>,
    pub(crate) address: Option<Address>,
    pub(crate) phone_numbers: Option<PhoneNumberList>,
    pub(crate) email_addresses: Option<EmailAddressList>,
    /*
    Website (array?)
    Google Maps Link / Coordinates
    */
}
//-------------

#[derive(Debug, Default)]
pub(crate) struct EmailAddressList(Vec<EmailAddress>);

impl IntoIterator for EmailAddressList {
    type Item = EmailAddress;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Default)]
pub(crate) struct EmailAddress(String);

impl From<&str> for EmailAddress {
    fn from(value: &str) -> Self {
        EmailAddress(value.into())
    }
}

impl From<EmailAddress> for String {
    fn from(val: EmailAddress) -> Self {
        val.0
    }
}

//----------------------------------

#[derive(Debug, Default)]
pub(crate) struct PhoneNumberList(Vec<PhoneNumber>);

impl IntoIterator for PhoneNumberList {
    type Item = PhoneNumber;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl PhoneNumberList {
    pub fn push(&mut self, value: String) {
        self.0.push(value.into());
    }
}

#[derive(Debug, Default)]
pub(crate) struct PhoneNumber(String);

impl From<&str> for PhoneNumber {
    fn from(value: &str) -> Self {
        PhoneNumber(value.into())
    }
}

impl From<String> for PhoneNumber {
    fn from(value: String) -> Self {
        PhoneNumber(value.into())
    }
}

impl From<PhoneNumber> for String {
    fn from(val: PhoneNumber) -> Self {
        val.0
    }
}

//----------------------------------

#[derive(Debug, Default)]
pub(crate) struct Address {
    lines: Vec<String>,
}


impl Address {
    pub fn push(&mut self, value: String) {
        self.lines.push(value);
    }
}

//----------------------------------

#[derive(Debug, Default)]
pub(crate) struct Code(String);

impl From<&str> for Code {
    fn from(value: &str) -> Self {
        Code(value.into())
    }
}

impl From<Code> for String {
    fn from(val: Code) -> Self {
        val.0
    }
}

//----------------------------------

#[derive(Debug, Default)]
pub(crate) struct Name(String);

impl From<&str> for Name {
    fn from(value: &str) -> Self {
        Name(value.into())
    }
}
