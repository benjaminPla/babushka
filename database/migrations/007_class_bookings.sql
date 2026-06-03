CREATE TABLE IF NOT EXISTS class_bookings (
    id              UUID           PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id      UUID           NOT NULL REFERENCES students(id),
    class_id        UUID           NOT NULL REFERENCES classes(id),
    amount_cents    INTEGER        NOT NULL,
    discount_cents  INTEGER        NOT NULL DEFAULT 0,
    discount_reason VARCHAR(500),
    paid_at         TIMESTAMPTZ,
    payment_method  payment_method,
    status          payment_status NOT NULL DEFAULT 'pending',
    notes           VARCHAR(500),
    created_at      TIMESTAMPTZ    NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ    NOT NULL DEFAULT NOW(),

    CONSTRAINT class_bookings_unique
        UNIQUE (student_id, class_id),

    CONSTRAINT class_bookings_amount_positive
        CHECK (amount_cents > 0),

    CONSTRAINT class_bookings_discount_valid
        CHECK (discount_cents >= 0 AND discount_cents <= amount_cents),

    CONSTRAINT class_bookings_paid_at_requires_paid_status
        CHECK (paid_at IS NULL OR status = 'paid')
);

DROP TRIGGER IF EXISTS class_bookings_set_updated_at ON class_bookings;
CREATE TRIGGER class_bookings_set_updated_at
    BEFORE UPDATE ON class_bookings
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

-- Enforce age_group match between student and class's course
CREATE OR REPLACE FUNCTION check_class_booking_age_group()
RETURNS TRIGGER AS $$
BEGIN
    IF (SELECT age_group FROM students WHERE id = NEW.student_id) <>
       (SELECT c.age_group FROM classes cl JOIN courses c ON c.id = cl.course_id WHERE cl.id = NEW.class_id) THEN
        RAISE EXCEPTION 'student age_group does not match course age_group';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS class_bookings_check_age_group ON class_bookings;
CREATE TRIGGER class_bookings_check_age_group
    BEFORE INSERT ON class_bookings
    FOR EACH ROW
    EXECUTE FUNCTION check_class_booking_age_group();
