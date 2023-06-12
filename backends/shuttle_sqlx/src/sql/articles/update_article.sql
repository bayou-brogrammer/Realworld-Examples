WITH article AS (
    UPDATE articles
    SET title = COALESCE($1, title),
        description = COALESCE($2, description),
        body = COALESCE($3, body),
        slug = COALESCE($4, slug)
    WHERE slug = $5
        AND author_id = $6
    RETURNING *
)
SELECT article.id,
    article.slug,
    article.title,
    article.description,
    article.body,
    article.created_at,
    article.updated_at,
    FALSE AS "favorited!",
    '{}'::VARCHAR [] AS "tag_list!",
    CAST(0 as INT8) AS "favorites_count!",
    (
        users.id,
        users.username,
        users.bio,
        users.image,
        EXISTS (
            SELECT 1
            FROM follows
            WHERE follows.follower_id = $6
                AND follows.followee_id = users.id
        )
    ) AS "author!: UserProfile"
FROM article
    INNER JOIN users ON users.id = article.author_id