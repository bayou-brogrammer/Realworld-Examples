-- Article
CREATE TABLE IF NOT EXISTS articles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    body TEXT NOT NULL,
    description TEXT NOT NULL,
    title VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    author_id UUID NOT NULL REFERENCES users (id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS articles_slug_idx ON articles (slug);
CREATE INDEX IF NOT EXISTS articles_author_id_idx ON articles (author_id);
SELECT diesel_manage_updated_at('articles');
-- Fav Articles --
CREATE TABLE favorite_articles (
    PRIMARY KEY (user_id, article_id),
    user_id UUID NOT NULL REFERENCES users (id),
    article_id UUID NOT NULL REFERENCES articles (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);
CREATE INDEX favorite_articles_user_id_idx ON favorite_articles (user_id);
CREATE INDEX favorite_articles_article_id_idx ON favorite_articles (article_id);
SELECT diesel_manage_updated_at('favorite_articles');
-- Article Tags --
CREATE TABLE IF NOT EXISTS article_tags (
    PRIMARY KEY (article_id, tag_name),
    article_id UUID NOT NULL REFERENCES articles (id) ON DELETE CASCADE,
    tag_name TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);
CREATE INDEX IF NOT EXISTS article_tags_article_id_idx ON article_tags (article_id);
CREATE INDEX IF NOT EXISTS article_tags_tag_name_idx ON article_tags (tag_name);
SELECT diesel_manage_updated_at('article_tags');