use const_format::formatcp;
use ego_tree::NodeRef;
use voyager::scraper::Node;

use super::utils::non_empty_vector::NonEmptyVector;

#[derive(Copy, Clone, Debug)]
enum Stage {
    SingleChoice,
    GroupChoice,
    AllMandatory,
}

const NBSP: char = '\u{00A0}';

const NBSPX6: &'static str = formatcp!("{}{}{}{}{}{}", NBSP, NBSP, NBSP, NBSP, NBSP, NBSP);

const AND: &'static str = formatcp!("{}e", NBSPX6);
const OR: &'static str = formatcp!("{}ou", NBSPX6);

#[derive(Debug, Clone)]
struct Exam(String);

impl From<&str> for Exam {
    fn from(exam: &str) -> Self {
        Exam(exam.into())
    }
}

#[derive(Debug, Clone)]
struct ExamGroup(Vec<Exam>);

impl From<Vec<Exam>> for ExamGroup {
    fn from(val: Vec<Exam>) -> Self {
        ExamGroup(val)
    }
}
impl From<NonEmptyVector<Exam>> for ExamGroup {
    fn from(val: NonEmptyVector<Exam>) -> Self {
        ExamGroup(val.0)
    }
}
#[derive(Debug, Default)]
pub struct Exams {
    optional: Option<OptionalExams>,
    mandatory: Option<MandatoryExams>,
}
#[derive(Debug)]
struct OptionalExams(NonEmptyVector<NonEmptyVector<ExamGroup>>);
#[derive(Debug)]
struct MandatoryExams(NonEmptyVector<Exam>);

// refactor this once tested
//
// current_node = None;
// iter.next();
//
// The above 2 lines shouldn't be repeated everywhere

pub(crate) fn exams_section<'a>(iter: &mut impl Iterator<Item = NodeRef<'a, Node>>) -> Exams {
    let mut current_state = None;
    let mut current_node = None;

    let mut optional_exams: Option<NonEmptyVector<NonEmptyVector<ExamGroup>>> = None;
    let mut mandatory_exams: Option<NonEmptyVector<Exam>> = None;

    loop {
        if current_node.is_none() {
            current_node = iter.next();
        }
        if let Some(node) = current_node {
            match node.value().as_text() {
                Some(text) => {
                    match current_state {
                        Some(state) => {
                            match text as &str {
                                AND => {
                                    current_state = None;
                                    if let Some(ref mut optional_exams) = optional_exams {
                                        optional_exams.push(NonEmptyVector::new(vec![].into()));
                                    }
                                }
                                OR => {
                                    //current_state = None;
                                    if let Some(ref mut optional_exams) = optional_exams {
                                        optional_exams.last_mut().push(vec![].into());
                                    }
                                }
                                _ => {
                                    match state {
                                        Stage::SingleChoice => {
                                            if let Some(ref mut optional_exams) = optional_exams {
                                                optional_exams
                                                    .last_mut()
                                                    .push(vec![(text as &str).into()].into());
                                            } else {
                                                optional_exams =
                                                    Some(NonEmptyVector::new(NonEmptyVector::new(
                                                        vec![(text as &str).into()].into(),
                                                    )))
                                            }
                                        }
                                        Stage::GroupChoice => {
                                            //TODO: Shouldn't be using ".0"
                                            if let Some(ref mut optional_exams) = optional_exams {
                                                optional_exams
                                                    .last_mut()
                                                    .last_mut()
                                                    .0
                                                    .push((text as &str).into());
                                            } else {
                                                optional_exams =
                                                    Some(NonEmptyVector::new(NonEmptyVector::new(
                                                        vec![(text as &str).into()].into(),
                                                    )))
                                            }
                                        }
                                        Stage::AllMandatory => {
                                            if let Some(ref mut mandatory_exams) = mandatory_exams {
                                                mandatory_exams.push((text as &str).into());
                                            } else {
                                                mandatory_exams = Some(NonEmptyVector::new(
                                                    (text as &str).into(),
                                                ));
                                            }
                                        }
                                    };
                                }
                            }
                            current_node = None;
                            iter.next();
                        }
                        None => match text as &str {
                            "Uma das seguintes provas:" => {
                                current_state = Some(Stage::SingleChoice);
                                current_node = None;
                                iter.next();
                            }
                            "Um dos seguintes conjuntos:" => {
                                current_state = Some(Stage::GroupChoice);
                                current_node = None;
                                iter.next();
                            }
                            _ => {
                                current_state = Some(Stage::AllMandatory);
                            }
                        },
                    }
                }
                None => {
                    break;
                }
            }
        }
    }
    return Exams {
        optional: match optional_exams {
            Some(val) => Some(OptionalExams(val)),
            None => None,
        },
        mandatory: match mandatory_exams {
            Some(val) => Some(MandatoryExams(val)),
            None => None,
        },
    };
}
