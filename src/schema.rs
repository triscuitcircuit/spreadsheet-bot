table! {
    ban (id) {
        id -> Int4,
        users -> Int4,
        guild -> Nullable<Varchar>,
        end_epoch -> Nullable<Varchar>,
    }
}

table! {
    crossroles (id) {
        id -> Int4,
        roles -> Int4,
        color -> Varchar,
        mentionable -> Bool,
        guild -> Varchar,
        users -> Int4,
    }
}

table! {
    roles (id) {
        id -> Int4,
        role_id -> Varchar,
        guild -> Varchar,
    }
}

table! {
    users (id) {
        id -> Int4,
        discord_id -> Varchar,
    }
}

joinable!(ban -> users (users));
joinable!(crossroles -> users (users));

allow_tables_to_appear_in_same_query!(
    ban,
    crossroles,
    roles,
    users,
);
