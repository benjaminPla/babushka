ALTER TABLE students ALTER COLUMN email DROP NOT NULL;
ALTER TABLE students DROP CONSTRAINT students_email_format;
ALTER TABLE students ADD CONSTRAINT students_email_format
    CHECK (email IS NULL OR email ~* '^[a-z0-9._%+\-]+@[a-z0-9.\-]+\.[a-z]{2,}$');

ALTER TABLE teachers ALTER COLUMN email DROP NOT NULL;
ALTER TABLE teachers DROP CONSTRAINT teachers_email_format;
ALTER TABLE teachers ADD CONSTRAINT teachers_email_format
    CHECK (email IS NULL OR email ~* '^[a-z0-9._%+\-]+@[a-z0-9.\-]+\.[a-z]{2,}$');
