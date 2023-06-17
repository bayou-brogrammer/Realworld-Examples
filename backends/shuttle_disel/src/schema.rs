// @generated automatically by Diesel CLI.

diesel::table! {
    article_tags (article_id, tag_name) {
        article_id -> Uuid,
        tag_name -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    articles (id) {
        id -> Uuid,
        body -> Text,
        description -> Text,
        #[max_length = 255]
        title -> Varchar,
        #[max_length = 255]
        slug -> Varchar,
        author_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    favorite_articles (user_id, article_id) {
        user_id -> Uuid,
        article_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    followers (user_id, follower_id) {
        user_id -> Uuid,
        follower_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Text,
        #[max_length = 254]
        email -> Varchar,
        password -> Text,
        bio -> Nullable<Text>,
        image -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(article_tags -> articles (article_id));
diesel::joinable!(articles -> users (author_id));
diesel::joinable!(favorite_articles -> articles (article_id));
diesel::joinable!(favorite_articles -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    article_tags,
    articles,
    favorite_articles,
    followers,
    users,
);
