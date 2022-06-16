table! {
    cnaef_areas (rowid) {
        rowid -> Integer,
        code -> Nullable<Text>,
        name -> Nullable<Text>,
    }
}

table! {
    duration_units (rowid) {
        rowid -> Integer,
        name -> Nullable<Text>,
    }
}

table! {
    durations (rowid) {
        rowid -> Integer,
        unit -> Nullable<Integer>,
        ammount -> Nullable<Integer>,
    }
}

table! {
    exams (rowid) {
        rowid -> Integer,
        code -> Nullable<Text>,
        name -> Nullable<Text>,
    }
}

table! {
    institutions (rowid) {
        rowid -> Integer,
        code -> Nullable<Text>,
        name -> Nullable<Text>,
        address -> Nullable<Text>,
        phone_numbers -> Nullable<Text>,
        email_addresses -> Nullable<Text>,
    }
}

table! {
    main (rowid) {
        rowid -> Integer,
        ects -> Nullable<Integer>,
        institution -> Nullable<Integer>,
    }
}

table! {
    mandatory_exams (rowid) {
        rowid -> Integer,
        exam -> Nullable<Integer>,
        main -> Nullable<Integer>,
    }
}

joinable!(durations -> duration_units (unit));
joinable!(main -> institutions (institution));
joinable!(mandatory_exams -> exams (exam));
joinable!(mandatory_exams -> main (main));

allow_tables_to_appear_in_same_query!(
    cnaef_areas,
    duration_units,
    durations,
    exams,
    institutions,
    main,
    mandatory_exams,
);
