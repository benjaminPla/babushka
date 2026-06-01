CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE IF NOT EXISTS teachers (
    id          UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    first_name  VARCHAR(50)     NOT NULL,
    last_name   VARCHAR(50)     NOT NULL,
    phone       VARCHAR(25)     NOT NULL,
    email       VARCHAR(254)    NOT NULL,
    created_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    CONSTRAINT teachers_email_unique
        UNIQUE (email),

    CONSTRAINT teachers_phone_unique
        UNIQUE (phone),

    CONSTRAINT teachers_first_name_not_blank
        CHECK (LENGTH(TRIM(first_name)) > 0),

    CONSTRAINT teachers_last_name_not_blank
        CHECK (LENGTH(TRIM(last_name)) > 0),

    CONSTRAINT teachers_email_format
        CHECK (email ~* '^[a-z0-9._%+\-]+@[a-z0-9.\-]+\.[a-z]{2,}$'),

    CONSTRAINT teachers_phone_format
        CHECK (phone ~ '^\+?[0-9][0-9\s\-\(\)]{6,24}$')
);

--  CREATE TRIGGER teachers_set_updated_at
    --  BEFORE UPDATE ON teachers
    --  FOR EACH ROW
    --  EXECUTE FUNCTION set_updated_at();
