table! {
    users{
        id -> Integer,
        name -> Varchar,
        second_name -> Varchar,
        password -> Longtext,
        scores -> Integer,
    }
}
table! {
    tests{
        id -> Integer,
        Level -> Integer,
    }
}
