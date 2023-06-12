WITH article AS (
    INSERT INTO articles (slug, title, description, body, author_id)
    VALUES ($1, $2, $3, $4, $5)
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
            WHERE follows.follower_id = $5
                AND follows.followee_id = users.id
        )
    ) AS "author!: UserProfile"
FROM article
    INNER JOIN users ON users.id = article.author_id