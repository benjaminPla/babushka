CREATE TABLE IF NOT EXISTS enrollments (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id       UUID        NOT NULL REFERENCES students(id),
    course_period_id UUID        NOT NULL REFERENCES course_periods(id) ON DELETE CASCADE,
    dropped_at       TIMESTAMPTZ,
    enrolled_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    notes            VARCHAR(500),

    CONSTRAINT enrollments_unique
        UNIQUE (student_id, course_period_id)
);

DROP TRIGGER IF EXISTS enrollments_set_updated_at ON enrollments;
CREATE TRIGGER enrollments_set_updated_at
    BEFORE UPDATE ON enrollments
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

CREATE OR REPLACE FUNCTION check_enrollment_age_group()
RETURNS TRIGGER AS $$
BEGIN
    IF (SELECT age_group FROM students WHERE id = NEW.student_id) <>
       (SELECT c.age_group
        FROM course_periods cp
        JOIN courses c ON c.id = cp.course_id
        WHERE cp.id = NEW.course_period_id) THEN
        RAISE EXCEPTION 'student age_group does not match course age_group';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS enrollments_check_age_group ON enrollments;
CREATE TRIGGER enrollments_check_age_group
    BEFORE INSERT ON enrollments
    FOR EACH ROW
    EXECUTE FUNCTION check_enrollment_age_group();

CREATE OR REPLACE FUNCTION check_enrollment_capacity()
RETURNS TRIGGER AS $$
DECLARE
    active_count INTEGER;
    max_capacity SMALLINT;
BEGIN
    IF NEW.dropped_at IS NULL AND (TG_OP = 'INSERT' OR OLD.dropped_at IS NOT NULL) THEN
        SELECT COUNT(e.id),
               (SELECT c.capacity
                FROM course_periods cp
                JOIN courses c ON c.id = cp.course_id
                WHERE cp.id = NEW.course_period_id)
        INTO active_count, max_capacity
        FROM enrollments e
        WHERE e.course_period_id = NEW.course_period_id
          AND e.dropped_at IS NULL
          AND e.id <> COALESCE(NEW.id, '00000000-0000-0000-0000-000000000000'::UUID);

        IF COALESCE(active_count, 0) >= max_capacity THEN
            RAISE EXCEPTION 'course has reached maximum capacity of %', max_capacity;
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS enrollments_check_capacity ON enrollments;
CREATE TRIGGER enrollments_check_capacity
    BEFORE INSERT OR UPDATE ON enrollments
    FOR EACH ROW
    EXECUTE FUNCTION check_enrollment_capacity();
