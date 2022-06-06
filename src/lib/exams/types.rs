use std::str::FromStr;

use crate::lib::utils::non_empty_vector::NonEmptyVector;

#[derive(Debug, Clone, Default)]
pub struct Exam {
    pub code: Option<ExamCode>,
    pub name: Option<ExamName>,
}

// newtype
#[derive(Debug, Clone)]
pub struct ExamCode(String);
impl From<&str> for ExamCode {
    fn from(exam: &str) -> Self {
        ExamCode(exam.into())
    }
}

impl From<ExamCode> for String {
    fn from(val: ExamCode) -> Self {
        val.0
    }
}

// newtype
#[derive(Debug, Clone)]
pub struct ExamName(String);
impl From<&str> for ExamName {
    fn from(exam: &str) -> Self {
        ExamName(exam.into())
    }
}

impl From<ExamName> for String {
    fn from(val: ExamName) -> Self {
        val.0
    }
}

// newtype
#[derive(Debug, Clone, Default)]
pub struct ExamGroup(Vec<Exam>);

impl IntoIterator for ExamGroup {
    type Item = Exam;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> From<&'a mut ExamGroup> for &'a mut Vec<Exam> {
    fn from(val: &'a mut ExamGroup) -> Self {
        &mut val.0
    }
}

impl<'a> From<&'a ExamGroup> for &'a Vec<Exam> {
    fn from(val: &'a ExamGroup) -> Self {
        &val.0
    }
}

impl From<Vec<Exam>> for ExamGroup {
    fn from(val: Vec<Exam>) -> Self {
        ExamGroup(val)
    }
}

impl From<NonEmptyVector<Exam>> for ExamGroup {
    fn from(val: NonEmptyVector<Exam>) -> Self {
        ExamGroup(val.into())
    }
}

#[derive(Debug, Default)]
pub struct Exams {
    pub optional: Option<OptionalExams>,
    pub mandatory: Option<MandatoryExams>,
}
#[derive(Debug)]
pub struct OptionalExams(NonEmptyVector<NonEmptyVector<ExamGroup>>);

impl From<NonEmptyVector<NonEmptyVector<ExamGroup>>> for OptionalExams {
    fn from(val: NonEmptyVector<NonEmptyVector<ExamGroup>>) -> Self {
        OptionalExams(val)
    }
}

impl From<OptionalExams> for NonEmptyVector<NonEmptyVector<ExamGroup>> {
    fn from(val: OptionalExams) -> Self {
        val.0
    }
}

impl From<OptionalExams> for Vec<NonEmptyVector<ExamGroup>> {
    fn from(val: OptionalExams) -> Self {
        let val: NonEmptyVector<NonEmptyVector<ExamGroup>> = val.into();
        val.into()
    }
}

#[derive(Debug)]
pub struct MandatoryExams(NonEmptyVector<Exam>);

impl From<NonEmptyVector<Exam>> for MandatoryExams {
    fn from(val: NonEmptyVector<Exam>) -> Self {
        MandatoryExams(val)
    }
}

impl From<MandatoryExams> for NonEmptyVector<Exam> {
    fn from(val: MandatoryExams) -> Self {
        val.0
    }
}

impl From<MandatoryExams> for Vec<Exam> {
    fn from(val: MandatoryExams) -> Self {
        let val: NonEmptyVector<Exam> = val.into();
        val.into()
    }
}

impl FromStr for Exam {
    //TODO: do this
    type Err = ();

    //? should do some checks maybe
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        return match s.split_once(" ") {
            Some((code, name)) => Ok(Exam {
                code: Some(code.trim().into()),
                name: Some(name.trim().into()),
            }),
            None => Err(()),
        };
    }
}

impl TryInto<Exam> for &str {
    //TODO: do this
    type Error = ();

    fn try_into(self) -> Result<Exam, Self::Error> {
        self.parse::<Exam>()
    }
}
