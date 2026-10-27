CREATE TABLE IF NOT EXISTS articles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    author_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    slug TEXT NOT NULL,
    title TEXT NOT NULL,
    short TEXT,
    content TEXT NOT NULL,
    published_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    edited_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS articles_author_id_idx ON articles (author_id);

CREATE UNIQUE INDEX IF NOT EXISTS articles_slug_lower_idx ON articles (lower(slug));
