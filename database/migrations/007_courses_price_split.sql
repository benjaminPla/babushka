-- Rename existing price columns to their cash variants
ALTER TABLE courses RENAME COLUMN class_price_cents TO class_price_cash_cents;
ALTER TABLE courses RENAME COLUMN month_price_cents TO month_price_cash_cents;

-- Add transfer price columns (nullable while we seed)
ALTER TABLE courses
    ADD COLUMN class_price_transfer_cents INTEGER,
    ADD COLUMN month_price_transfer_cents INTEGER;

-- Seed transfer prices from cash prices
UPDATE courses
    SET class_price_transfer_cents = class_price_cash_cents,
        month_price_transfer_cents = month_price_cash_cents;

-- Enforce NOT NULL on new columns
ALTER TABLE courses
    ALTER COLUMN class_price_transfer_cents SET NOT NULL,
    ALTER COLUMN month_price_transfer_cents  SET NOT NULL;

-- Drop old constraints (names reference pre-rename columns, now renamed)
ALTER TABLE courses
    DROP CONSTRAINT courses_class_price_positive,
    DROP CONSTRAINT courses_month_price_positive;

-- Add constraints for all four price columns
ALTER TABLE courses
    ADD CONSTRAINT courses_class_price_cash_positive     CHECK (class_price_cash_cents > 0),
    ADD CONSTRAINT courses_class_price_transfer_positive CHECK (class_price_transfer_cents > 0),
    ADD CONSTRAINT courses_month_price_cash_positive     CHECK (month_price_cash_cents > 0),
    ADD CONSTRAINT courses_month_price_transfer_positive CHECK (month_price_transfer_cents > 0);
