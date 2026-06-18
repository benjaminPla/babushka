# Changelog

## [1.1.0] - 2026-06-18

### Added
- Income widget on dashboard: last month (actual), current month (running), next month (expected at cash reference prices)
- Enrolled students roster in course detail view, side-by-side with course info, showing name and paid/pending status
- Search filters on students, teachers, and courses tables
- Dashboard enroll button: quick-enroll a student into a course period without leaving the dashboard
- Dashboard debtors table: one row per unpaid enrollment with student/course filters and per-row payment action

### Changed
- Payment model reworked: payments are now embedded directly in enrollments (1-to-1); no standalone payments table
- Course prices split into four variants: monthly cash, monthly transfer, class cash, class transfer
- Enrollment now records pricing type (monthly or per-class)
- Debtors table shows course + period in a single column; reference prices shown in the payment form
- Removed discount as a payment method (cash, transfer, card only)
- Student ledger replaced by per-enrollment table showing paid amount or pending status
- Email is now optional for students and teachers

### Internal
- Migrations 007–011 applied automatically on startup

## [1.0.0] - 2026-06-15

### Added
- Panel (dashboard) view with summary widgets: student count by age group, course count by age group, teacher total, and a ranked debtors table with quick navigation to each student's ledger
- Student management: create, edit, delete, per-column filtering, detail view
- Teacher management: create, edit, delete, per-column filtering, detail view
- Course management: create, edit, delete, per-column filtering, detail view with course periods
- Course periods: create and delete periods per course
- Enrollments: enroll students into course periods from the student detail view
- Payments: register payments (cash, transfer, card, discount) per student
- Student ledger: unified debt + payment history with running balance per student
- Self-update: version check on startup with one-click in-app update
- Embedded PostgreSQL with persistent data across restarts
- Migrations bundled in binary and applied automatically on startup
- Dark desktop theme with Phosphor icons
- Argentine Spanish UI throughout
