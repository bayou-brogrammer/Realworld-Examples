SELECT articles.id,
    articles.slug,
    articles.title,
    articles.description,
    articles.body,
    articles.created_at,
    articles.updated_at,
    COALESCE(
        (
            SELECT array_agg(
                    tags.name
                    ORDER BY tags.name ASC
                )
            FROM article_tags
                INNER JOIN tags ON article_tags.tag_id = tags.id
            WHERE article_tags.article_id = articles.id
        ),
        '{}'::VARCHAR []
    ) AS "tag_list!",
    (
        $2::INT4 IS NOT NULL
        AND EXISTS (
            SELECT 1
            FROM article_favs
            WHERE article_favs.article_id = articles.id
                AND article_favs.user_id = $2
        )
    ) AS "favorited!",
    (
        SELECT COUNT(*)
        FROM article_favs
        WHERE article_favs.article_id = articles.id
    ) AS "favorites_count!",
    (
        users.id,
        users.username,
        users.bio,
        users.image,
        EXISTS (
            SELECT 1
            FROM follows
            WHERE follows.follower_id = $2
                AND follows.followee_id = users.id
        )
    ) AS "author!: UserProfile"
FROM articles
    INNER JOIN users ON articles.author_id = users.id
WHERE articles.slug = $1
ORDER BY created_at DESC