CREATE TYPE payment_status AS ENUM ('pending', 'paid', 'overdue');

CREATE TABLE IF NOT EXISTS payments (
    id            UUID           PRIMARY KEY DEFAULT gen_random_uuid(),
    enrollment_id UUID           NOT NULL REFERENCES enrollments(id),
    amount_cents  INTEGER        NOT NULL,
    due_date      DATE           NOT NULL,
    paid_at       TIMESTAMPTZ,
    status        payment_status NOT NULL DEFAULT 'pending',
    notes         VARCHAR(500),
    created_at    TIMESTAMPTZ    NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ    NOT NULL DEFAULT NOW(),

    CONSTRAINT payments_unique
        UNIQUE (enrollment_id, due_date),

    CONSTRAINT payments_amount_positive
        CHECK (amount_cents > 0),

    CONSTRAINT payments_paid_at_requires_paid_status
        CHECK (paid_at IS NULL OR status = 'paid')
);

DROP TRIGGER IF EXISTS payments_set_updated_at ON payments;
CREATE TRIGGER payments_set_updated_at
    BEFORE UPDATE ON payments
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();
