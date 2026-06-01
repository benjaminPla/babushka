DO $$ BEGIN
    CREATE TYPE age_group AS ENUM ('adult', 'minor');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS students (
    age_group       age_group         NOT NULL,
    created_at      TIMESTAMPTZ       NOT NULL DEFAULT NOW(),
    email           VARCHAR(254)      NOT NULL,
    first_name      VARCHAR(100)      NOT NULL,
    id              UUID              PRIMARY KEY DEFAULT gen_random_uuid(),
    last_name       VARCHAR(100)      NOT NULL,
    notes           VARCHAR(500),
    phone           VARCHAR(25)       NOT NULL,
    updated_at      TIMESTAMPTZ       NOT NULL DEFAULT NOW(),

    CONSTRAINT students_email_unique
        UNIQUE (email),

    CONSTRAINT students_email_format
        CHECK (email ~* '^[a-z0-9._%+\-]+@[a-z0-9.\-]+\.[a-z]{2,}$'),

    CONSTRAINT students_first_name_not_blank
        CHECK (LENGTH(TRIM(first_name)) > 0),

    CONSTRAINT students_last_name_not_blank
        CHECK (LENGTH(TRIM(last_name)) > 0),

    CONSTRAINT students_phone_format
        CHECK (phone ~ '^\+?[0-9][0-9\s\-\(\)]{6,24}$')
);

DROP TRIGGER IF EXISTS students_set_updated_at ON students;
CREATE TRIGGER students_set_updated_at
    BEFORE UPDATE ON students
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();
