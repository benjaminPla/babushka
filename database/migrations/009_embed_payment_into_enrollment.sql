BEGIN;

-- 1. New columns on enrollments
ALTER TABLE enrollments
    ADD COLUMN pricing_type TEXT NOT NULL DEFAULT 'monthly'
        CHECK (pricing_type IN ('monthly', 'class')),
    ADD COLUMN paid_amount_cents INTEGER CHECK (paid_amount_cents > 0),
    ADD COLUMN paid_at TIMESTAMPTZ;
ALTER TABLE enrollments ADD COLUMN payment_method payment_method;

-- 2. All-or-nothing payment fields constraint
ALTER TABLE enrollments ADD CONSTRAINT payment_complete CHECK (
    (paid_amount_cents IS NULL AND payment_method IS NULL AND paid_at IS NULL) OR
    (paid_amount_cents IS NOT NULL AND payment_method IS NOT NULL AND paid_at IS NOT NULL)
);

-- 3. Migrate existing payment data
UPDATE enrollments e
SET paid_amount_cents = p.amount_cents,
    payment_method    = p.payment_method,
    paid_at           = p.paid_at
FROM payments p
WHERE p.enrollment_id = e.id
  AND p.payment_method::text != 'discount';

-- 4. Drop payments table
DROP TABLE payments;

-- 5. Rebuild payment_method enum without 'discount'
ALTER TABLE enrollments
    ALTER COLUMN payment_method TYPE TEXT USING payment_method::text;
DROP TYPE payment_method;
CREATE TYPE payment_method AS ENUM ('cash', 'transfer', 'card');
ALTER TABLE enrollments
    ALTER COLUMN payment_method TYPE payment_method
        USING payment_method::payment_method;

-- 6. Drop migration default
ALTER TABLE enrollments ALTER COLUMN pricing_type DROP DEFAULT;

COMMIT;
