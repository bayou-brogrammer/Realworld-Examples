SELECT users.id,
    users.username AS "username?",
    users.bio,
    users.image,
    (
        $2::INT4 IS NOT NULL
        AND EXISTS (
            SELECT 1
            FROM follows
            WHERE follows.follower_id = $2
                AND follows.followee_id = users.id
        )
    ) AS "following!"
FROM users
WHERE username = $1