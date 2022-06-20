table! {
    cnaef_areas (code) {
        code -> Text,
        name -> Nullable<Text>,
    }
}

table! {
    contests (name) {
        name -> Text,
    }
}

table! {
    course_institution (institution, course) {
        ects -> Nullable<Integer>,
        institution -> Text,
        course -> Text,
    }
}

table! {
    courses (code) {
        code -> Text,
        name -> Nullable<Text>,
    }
}

table! {
    degrees (name) {
        name -> Text,
    }
}

table! {
    duration_units (name) {
        name -> Text,
    }
}

table! {
    durations (institution, course) {
        institution -> Text,
        course -> Text,
        unit -> Nullable<Text>,
        ammount -> Nullable<Integer>,
    }
}

table! {
    education_types (name) {
        name -> Text,
    }
}

table! {
    exams (code) {
        code -> Text,
        name -> Nullable<Text>,
    }
}

table! {
    institutions (code) {
        code -> Text,
        name -> Nullable<Text>,
        address -> Nullable<Text>,
        phone_numbers -> Nullable<Text>,
        email_addresses -> Nullable<Text>,
    }
}

table! {
    mandatory_exams (rowid) {
        rowid -> Integer,
        exam -> Nullable<Text>,
        institution -> Text,
        course -> Text,
    }
}

joinable!(course_institution -> courses (course));
joinable!(course_institution -> institutions (institution));
joinable!(durations -> duration_units (unit));
joinable!(mandatory_exams -> exams (exam));

allow_tables_to_appear_in_same_query!(
    cnaef_areas,
    contests,
    course_institution,
    courses,
    degrees,
    duration_units,
    durations,
    education_types,
    exams,
    institutions,
    mandatory_exams,
);
