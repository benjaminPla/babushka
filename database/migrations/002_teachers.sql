CREATE TABLE IF NOT EXISTS teachers (
    created_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    email       VARCHAR(254)    NOT NULL,
    first_name  VARCHAR(50)     NOT NULL,
    id          UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    last_name   VARCHAR(50)     NOT NULL,
    notes       VARCHAR(500),
    phone       VARCHAR(25)     NOT NULL,
    updated_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    CONSTRAINT teachers_email_unique
        UNIQUE (email),
    CONSTRAINT teachers_email_format
        CHECK (email ~* '^[a-z0-9._%+\-]+@[a-z0-9.\-]+\.[a-z]{2,}$'),

    CONSTRAINT teachers_first_name_not_blank
        CHECK (LENGTH(TRIM(first_name)) >= 3),

    CONSTRAINT teachers_last_name_not_blank
        CHECK (LENGTH(TRIM(last_name)) >= 3),

    CONSTRAINT teachers_phone_unique
        UNIQUE (phone),

    CONSTRAINT teachers_phone_format
        CHECK (phone ~ '^\+?[0-9][0-9\s\-\(\)]{6,24}$')
);

DROP TRIGGER IF EXISTS teachers_set_updated_at ON teachers;
CREATE TRIGGER teachers_set_updated_at
    BEFORE UPDATE ON teachers
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();
