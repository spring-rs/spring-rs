use diesel::table;


table! {
    users {
        id -> BigInt,
        name -> Text,
        active -> Bool,
    }
}
