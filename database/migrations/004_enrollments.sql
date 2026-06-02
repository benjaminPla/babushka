CREATE TYPE enrollment_status AS ENUM ('active', 'dropped', 'completed');

CREATE TABLE IF NOT EXISTS enrollments (
    id          UUID              PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id  UUID              NOT NULL REFERENCES students(id),
    course_id   UUID              NOT NULL REFERENCES courses(id),
    status      enrollment_status NOT NULL DEFAULT 'active',
    enrolled_at TIMESTAMPTZ       NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ       NOT NULL DEFAULT NOW(),
    notes       VARCHAR(500),

    CONSTRAINT enrollments_unique
        UNIQUE (student_id, course_id)
);

DROP TRIGGER IF EXISTS enrollments_set_updated_at ON enrollments;
CREATE TRIGGER enrollments_set_updated_at
    BEFORE UPDATE ON enrollments
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

-- Enforce age_group match between student and course
CREATE OR REPLACE FUNCTION check_enrollment_age_group()
RETURNS TRIGGER AS $$
BEGIN
    IF (SELECT age_group FROM students WHERE id = NEW.student_id) <>
       (SELECT age_group FROM courses   WHERE id = NEW.course_id) THEN
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

-- Enforce course capacity (count only active enrollments)
CREATE OR REPLACE FUNCTION check_enrollment_capacity()
RETURNS TRIGGER AS $$
DECLARE
    active_count INTEGER;
    max_capacity SMALLINT;
BEGIN
    IF NEW.status = 'active' AND (TG_OP = 'INSERT' OR OLD.status <> 'active') THEN
        SELECT COUNT(*), c.capacity
        INTO active_count, max_capacity
        FROM enrollments e
        JOIN courses c ON c.id = NEW.course_id
        WHERE e.course_id = NEW.course_id
          AND e.status = 'active'
          AND e.id <> COALESCE(NEW.id, '00000000-0000-0000-0000-000000000000'::UUID)
        GROUP BY c.capacity;

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
