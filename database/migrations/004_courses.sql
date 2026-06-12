CREATE TABLE IF NOT EXISTS courses (
    age_group         age_group    NOT NULL,
    capacity          SMALLINT     NOT NULL,
    class_price_cents INTEGER      NOT NULL,
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    id                UUID         PRIMARY KEY  DEFAULT gen_random_uuid(),
    month_price_cents INTEGER      NOT NULL,
    name              VARCHAR(100) NOT NULL,
    notes             VARCHAR(500),
    teacher_id        UUID         NOT NULL REFERENCES teachers(id),
    updated_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT courses_capacity_min
        CHECK (capacity > 0),
    CONSTRAINT courses_capacity_max
        CHECK (capacity <= 100),

    CONSTRAINT courses_class_price_positive
        CHECK (class_price_cents > 0),
    CONSTRAINT courses_month_price_positive
        CHECK (month_price_cents > 0),

    CONSTRAINT courses_name_unique
        UNIQUE (name)
);

DROP TRIGGER IF EXISTS courses_set_updated_at ON courses;
CREATE TRIGGER courses_set_updated_at
    BEFORE UPDATE ON courses
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS course_periods (
    course_id  UUID         NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    end_date   DATE         NOT NULL,
    id         UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    start_date DATE         NOT NULL,
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
