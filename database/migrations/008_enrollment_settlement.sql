-- Enrollments become pure slot reservations — no price commitment at creation time
ALTER TABLE enrollments DROP CONSTRAINT enrollments_price_positive;
ALTER TABLE enrollments DROP COLUMN price_cents;

-- Payments can optionally settle an enrollment
ALTER TABLE payments
    ADD COLUMN enrollment_id UUID REFERENCES enrollments(id) ON DELETE SET NULL;
