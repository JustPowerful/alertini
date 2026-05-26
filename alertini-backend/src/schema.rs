// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Uuid,
        firstname -> Text,
        lastname -> Text,
        email -> Text,
        password -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    vehicles (id) {
        id -> Uuid,
        license_plate -> Text,
        car_desc -> Nullable<Text>,
        user_id -> Uuid,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(vehicles -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(users, vehicles,);
