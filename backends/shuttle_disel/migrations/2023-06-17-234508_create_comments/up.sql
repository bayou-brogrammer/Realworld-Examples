-- Article
CREATE TABLE IF NOT EXISTS comments (
    id SERIAL PRIMARY KEY,
    article_id UUID NOT NULL REFERENCES articles (id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS comments_article_id_idx ON comments (article_id);
CREATE INDEX IF NOT EXISTS comments_user_id_idx ON comments (user_id);
SELECT diesel_manage_updated_at('comments');