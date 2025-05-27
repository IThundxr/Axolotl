// @generated automatically by Diesel CLI.

diesel::table! {
    groups (id) {
        id -> Uuid,
        name -> Text,
    }
}

diesel::table! {
    groups_permissions (group_id, permission_id) {
        group_id -> Uuid,
        permission_id -> Uuid,
    }
}

diesel::table! {
    permissions (id) {
        id -> Uuid,
        name -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        parent_id -> Nullable<Uuid>,
        username -> Text,
        password_hash -> Text,
    }
}

diesel::table! {
    users_groups (user_id, group_id) {
        user_id -> Uuid,
        group_id -> Uuid,
    }
}

diesel::joinable!(groups_permissions -> groups (group_id));
diesel::joinable!(groups_permissions -> permissions (permission_id));
diesel::joinable!(users_groups -> groups (group_id));
diesel::joinable!(users_groups -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    groups,
    groups_permissions,
    permissions,
    users,
    users_groups,
);
