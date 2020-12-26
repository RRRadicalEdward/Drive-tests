table! {
    tests (id) {
        id -> Integer,
        description -> Text,
        answers -> Text,
        right_answer_id -> Integer,
        image -> Nullable<Binary>,
    }
}

table! {
    users (id) {
        id -> Integer,
        name -> Text,
        second_name -> Text,
        password -> Text,
        scores -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(tests, users,);
