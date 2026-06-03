CREATE TABLE IF NOT EXISTS classes (
    id               UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    course_period_id UUID         NOT NULL REFERENCES course_periods(id) ON DELETE CASCADE,
    name             VARCHAR(100) NOT NULL,
    scheduled_at     TIMESTAMPTZ  NOT NULL,
    notes            VARCHAR(500),
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

DROP TRIGGER IF EXISTS classes_set_updated_at ON classes;
CREATE TRIGGER classes_set_updated_at
    BEFORE UPDATE ON classes
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();
