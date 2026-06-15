# Changelog

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
