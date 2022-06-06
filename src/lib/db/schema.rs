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
        unit -> Text,
    }
}

table! {
    exams (rowid) {
        rowid -> Integer,
        code -> Text,
        name -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    cnaef_areas,
    duration_units,
    exams,
);
