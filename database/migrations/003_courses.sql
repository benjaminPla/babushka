CREATE TABLE IF NOT EXISTS courses (
    id                UUID         PRIMARY KEY  DEFAULT gen_random_uuid(),
    teacher_id        UUID         NOT NULL REFERENCES teachers(id),
    name              VARCHAR(100) NOT NULL,
    age_group         age_group    NOT NULL,
    capacity          SMALLINT     NOT NULL,
    price_cents       INTEGER      NOT NULL,
    class_price_cents INTEGER      NOT NULL,
    notes             VARCHAR(500),
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT courses_capacity_min
        CHECK (capacity > 0),

    CONSTRAINT courses_capacity_max
        CHECK (capacity <= 100),

    CONSTRAINT courses_price_positive
        CHECK (price_cents > 0),

    CONSTRAINT courses_class_price_positive
        CHECK (class_price_cents > 0)
);

DROP TRIGGER IF EXISTS courses_set_updated_at ON courses;
CREATE TRIGGER courses_set_updated_at
    BEFORE UPDATE ON courses
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS course_periods (
    id         UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id  UUID         NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    label      VARCHAR(50)  NOT NULL,
    start_date DATE         NOT NULL,
    end_date   DATE         NOT NULL,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT course_periods_unique
        UNIQUE (course_id, start_date),

    CONSTRAINT course_periods_dates_valid
        CHECK (end_date > start_date)
);

DROP TRIGGER IF EXISTS course_periods_set_updated_at ON course_periods;
CREATE TRIGGER course_periods_set_updated_at
    BEFORE UPDATE ON course_periods
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();
