CREATE TYPE payment_status AS ENUM ('pending', 'paid', 'overdue');
CREATE TYPE payment_method AS ENUM ('cash', 'transfer', 'card');

CREATE TABLE IF NOT EXISTS payments (
    id              UUID           PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id      UUID           NOT NULL REFERENCES students(id),
    amount_cents    INTEGER        NOT NULL,
    discount_cents  INTEGER        NOT NULL DEFAULT 0,
    discount_reason VARCHAR(500),
    due_date        DATE           NOT NULL,
    paid_at         TIMESTAMPTZ,
    payment_method  payment_method,
    status          payment_status NOT NULL DEFAULT 'paid',
    notes           VARCHAR(500),
    created_at      TIMESTAMPTZ    NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ    NOT NULL DEFAULT NOW(),

    CONSTRAINT payments_amount_positive
        CHECK (amount_cents > 0),

    CONSTRAINT payments_discount_valid
        CHECK (discount_cents >= 0 AND discount_cents <= amount_cents),

    CONSTRAINT payments_paid_at_requires_paid_status
        CHECK (paid_at IS NULL OR status = 'paid')
);

DROP TRIGGER IF EXISTS payments_set_updated_at ON payments;
CREATE TRIGGER payments_set_updated_at
    BEFORE UPDATE ON payments
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();
