-- ============================================================
-- DEV SEED — remove this file before shipping to production
-- Safe to re-run: all inserts use ON CONFLICT DO NOTHING
-- ============================================================

-- ── Teachers ─────────────────────────────────────────────────────────────────

INSERT INTO teachers (id, first_name, last_name, email, phone) VALUES
    ('a1000000-0000-0000-0000-000000000001', 'Laura',  'García',   'laura.garcia@babushka.dev',  '+541112345001'),
    ('a1000000-0000-0000-0000-000000000002', 'Marcos', 'Sánchez',  'marcos.sanchez@babushka.dev', '+541112345002')
ON CONFLICT DO NOTHING;

-- ── Students ──────────────────────────────────────────────────────────────────

INSERT INTO students (id, first_name, last_name, email, phone, age_group, notes) VALUES
    ('b1000000-0000-0000-0000-000000000001', 'Juan',      'Pérez',     'juan.perez@mail.com',     '+541155550001', 'adult',  NULL),
    ('b1000000-0000-0000-0000-000000000002', 'Valentina', 'López',     'valen.lopez@mail.com',    '+541155550002', 'adult',  'Viene los martes'),
    ('b1000000-0000-0000-0000-000000000003', 'Sofía',     'Rodríguez', 'sofi.rodriguez@mail.com', '+541155550003', 'minor',  NULL),
    ('b1000000-0000-0000-0000-000000000004', 'Tomás',     'Martínez',  'tomas.mtz@mail.com',      '+541155550004', 'minor',  'Pendiente de DNI')
ON CONFLICT DO NOTHING;

-- ── Courses ───────────────────────────────────────────────────────────────────
-- price_cents: $8.000,00 ARS monthly / $1.500,00 ARS per class

INSERT INTO courses (id, teacher_id, name, age_group, capacity, price_cents, class_price_cents) VALUES
    ('c1000000-0000-0000-0000-000000000001', 'a1000000-0000-0000-0000-000000000001', 'Tango Adultos',  'adult', 12, 800000, 150000),
    ('c1000000-0000-0000-0000-000000000002', 'a1000000-0000-0000-0000-000000000002', 'Tango Junior',   'minor',  8, 600000, 120000)
ON CONFLICT DO NOTHING;

-- ── Course periods ────────────────────────────────────────────────────────────
-- Tango Adultos: past (Apr), active (Jun), future (Aug)
-- Tango Junior:  past (Apr), active (Jun)

INSERT INTO course_periods (id, course_id, label, start_date, end_date) VALUES
    ('d1000000-0000-0000-0000-000000000001', 'c1000000-0000-0000-0000-000000000001', 'Abril 2026',  '2026-04-01', '2026-04-30'),
    ('d1000000-0000-0000-0000-000000000002', 'c1000000-0000-0000-0000-000000000001', 'Junio 2026',  '2026-06-01', '2026-06-30'),
    ('d1000000-0000-0000-0000-000000000003', 'c1000000-0000-0000-0000-000000000001', 'Agosto 2026', '2026-08-01', '2026-08-31'),
    ('d1000000-0000-0000-0000-000000000004', 'c1000000-0000-0000-0000-000000000002', 'Abril 2026',  '2026-04-01', '2026-04-30'),
    ('d1000000-0000-0000-0000-000000000005', 'c1000000-0000-0000-0000-000000000002', 'Junio 2026',  '2026-06-01', '2026-06-30')
ON CONFLICT DO NOTHING;

-- ── Enrollments ───────────────────────────────────────────────────────────────

INSERT INTO enrollments (id, student_id, course_period_id, agreed_price_cents) VALUES
    -- Juan: Tango Adultos Abril (past) + Junio (active)
    ('e1000000-0000-0000-0000-000000000001', 'b1000000-0000-0000-0000-000000000001', 'd1000000-0000-0000-0000-000000000001', 800000),
    ('e1000000-0000-0000-0000-000000000002', 'b1000000-0000-0000-0000-000000000001', 'd1000000-0000-0000-0000-000000000002', 800000),
    -- Valentina: Tango Adultos Junio (active), with discount
    ('e1000000-0000-0000-0000-000000000003', 'b1000000-0000-0000-0000-000000000002', 'd1000000-0000-0000-0000-000000000002', 650000),
    -- Sofía: Tango Junior Abril (past) + Junio (active)
    ('e1000000-0000-0000-0000-000000000004', 'b1000000-0000-0000-0000-000000000003', 'd1000000-0000-0000-0000-000000000004', 600000),
    ('e1000000-0000-0000-0000-000000000005', 'b1000000-0000-0000-0000-000000000003', 'd1000000-0000-0000-0000-000000000005', 600000),
    -- Tomás: Tango Junior Junio only (new student)
    ('e1000000-0000-0000-0000-000000000006', 'b1000000-0000-0000-0000-000000000004', 'd1000000-0000-0000-0000-000000000005', 600000)
ON CONFLICT DO NOTHING;

-- ── Payments ─────────────────────────────────────────────────────────────────
-- Payments are now linked to students directly (decoupled from enrollments)

INSERT INTO payments (id, student_id, amount_cents, status, payment_method, paid_at, due_date) VALUES
    -- Juan paid April in full, June partial
    ('f1000000-0000-0000-0000-000000000001', 'b1000000-0000-0000-0000-000000000001', 800000, 'paid',    'transfer', '2026-04-03 10:00:00-03', '2026-04-05'),
    ('f1000000-0000-0000-0000-000000000002', 'b1000000-0000-0000-0000-000000000001', 400000, 'paid',    'cash',     '2026-06-02 14:30:00-03', '2026-06-05'),
    -- Valentina paid June in full
    ('f1000000-0000-0000-0000-000000000003', 'b1000000-0000-0000-0000-000000000002', 650000, 'paid',    'transfer', '2026-06-01 09:00:00-03', '2026-06-05'),
    -- Sofía paid April, June pending
    ('f1000000-0000-0000-0000-000000000004', 'b1000000-0000-0000-0000-000000000003', 600000, 'paid',    'cash',     '2026-04-05 11:00:00-03', '2026-04-05'),
    ('f1000000-0000-0000-0000-000000000005', 'b1000000-0000-0000-0000-000000000003', 600000, 'pending', NULL,       NULL,                     '2026-06-05')
    -- Tomás: no payment yet (owes $6.000,00)
ON CONFLICT DO NOTHING;
