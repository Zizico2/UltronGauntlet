table! {
    cnaef_areas (rowid) {
        rowid -> Integer,
        code -> Text,
        name -> Text,
    }
}

table! {
    duration_units (rowid) {
        rowid -> Integer,
        name -> Text,
    }
}

table! {
    durations (rowid) {
        rowid -> Integer,
        unit -> Integer,
        ammount -> Integer,
    }
}

table! {
    exams (rowid) {
        rowid -> Integer,
        code -> Text,
        name -> Text,
    }
}

table! {
    main (rowid) {
        rowid -> Integer,
        ects -> Integer,
    }
}

table! {
    mandatory_exams (rowid) {
        rowid -> Integer,
        exam -> Integer,
        main -> Integer,
    }
}

joinable!(durations -> duration_units (unit));
joinable!(durations -> main (rowid));
joinable!(mandatory_exams -> exams (exam));
joinable!(mandatory_exams -> main (main));

allow_tables_to_appear_in_same_query!(
    cnaef_areas,
    duration_units,
    durations,
    exams,
    main,
    mandatory_exams,
);
