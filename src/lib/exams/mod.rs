use crate::lib::utils::non_empty_vector::NonEmptyVector;
use const_format::formatcp;
use ego_tree::NodeRef;
use voyager::scraper::Node;

pub use self::types::{Exam, ExamGroup, Exams, MandatoryExams, OptionalExams};

mod types;

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

    let optional_exams_mut_ref: &mut Option<NonEmptyVector<NonEmptyVector<ExamGroup>>> = &mut None;
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
                                    if let Some(optional_exams_mut_ref) = optional_exams_mut_ref {
                                        optional_exams_mut_ref
                                            .push(NonEmptyVector::new(ExamGroup::default()));
                                    }
                                }
                                OR => {
                                    //current_state = None;
                                    if let Some(optional_exams_mut_ref) = optional_exams_mut_ref {
                                        optional_exams_mut_ref
                                            .last_mut()
                                            .push(ExamGroup::default());
                                    }
                                }
                                _ => {
                                    match state {
                                        Stage::SingleChoice => {
                                            if let Ok(exam) = text.parse::<Exam>() {
                                                if let Some(optional_exams_mut_ref) =
                                                    optional_exams_mut_ref
                                                {
                                                    optional_exams_mut_ref
                                                        .last_mut()
                                                        .push(vec![exam].into());
                                                } else {
                                                    optional_exams = Some(NonEmptyVector::new(
                                                        NonEmptyVector::new(vec![exam].into()),
                                                    ));
                                                }
                                            }
                                        }
                                        Stage::GroupChoice => {
                                            //TODO: Shouldn't be using ".0"
                                            if let Some(optional_exams_mut_ref) =
                                                optional_exams_mut_ref
                                            {
                                                if let Ok(exam) = text.parse::<Exam>() {
                                                    let optional_exams_mut_ref: &mut Vec<Exam> =
                                                        optional_exams_mut_ref
                                                            .last_mut()
                                                            .last_mut()
                                                            .into();
                                                    optional_exams_mut_ref.push(exam);
                                                } else {
                                                    //TODO: what do?
                                                }
                                            } else {
                                                if let Ok(exam) = text.parse::<Exam>() {
                                                    optional_exams = Some(NonEmptyVector::new(
                                                        NonEmptyVector::new(vec![exam].into()),
                                                    ));
                                                }
                                            }
                                        }
                                        Stage::AllMandatory => {
                                            if let Ok(exam) = text.parse::<Exam>() {
                                                if let Some(ref mut mandatory_exams) =
                                                    mandatory_exams
                                                {
                                                    mandatory_exams.push(exam);
                                                } else {
                                                    mandatory_exams =
                                                        Some(NonEmptyVector::new(exam));
                                                }
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
            Some(val) => Some(val.into()),
            None => None,
        },
        mandatory: match mandatory_exams {
            Some(val) => Some(val.into()),
            None => None,
        },
    };
}
