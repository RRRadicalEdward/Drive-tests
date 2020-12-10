table! {
    users (id) {
        id -> Integer,
        name -> Text,
        second_name -> Text,
        password -> Text,
        scores -> Integer,
    }
}
table! {
    tests (id) {
        id -> Integer,
        level -> Integer,
        description -> Text,
        answers -> Text,
        right_answer_id -> Integer,
    }
}
