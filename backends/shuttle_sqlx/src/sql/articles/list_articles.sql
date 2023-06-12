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
        $6::INT4 IS NOT NULL
        AND EXISTS (
            SELECT 1
            FROM article_favs
            WHERE article_favs.article_id = articles.id
                AND article_favs.user_id = $6
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
        (
            $6 IS NOT NULL
            AND EXISTS (
                SELECT 1
                FROM follows
                WHERE follows.follower_id = $6
                    AND follows.followee_id = users.id
            )
        )
    ) AS "author!: UserProfile"
FROM articles
    INNER JOIN users ON articles.author_id = users.id
WHERE (
        $1::VARCHAR IS NULL
        OR users.username = $1
    )
    AND (
        $2::VARCHAR IS NULL
        OR EXISTS (
            SELECT 1
            FROM article_favs
                INNER JOIN users ON article_favs.user_id = users.id
            WHERE article_favs.article_id = articles.id
                AND users.username = $2
        )
    )
    AND (
        $3::VARCHAR IS NULL
        OR EXISTS (
            SELECT 1
            FROM article_tags
                INNER JOIN tags ON article_tags.tag_id = tags.id
            WHERE article_tags.article_id = articles.id
                AND tags.name = $3
        )
    )
ORDER BY created_at DESC
LIMIT $4 OFFSET $5