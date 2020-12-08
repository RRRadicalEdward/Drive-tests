table! {
    users (id) {
        id -> Integer,
        name -> Varchar,
        second_name -> Varchar,
        password -> Longtext,
        scores -> Integer,
    }
}
table! {
    tests (id) {
        id -> Integer,
        level -> Integer,
        description -> Text,
        answers -> Text,
    }
}
