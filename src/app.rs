use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;
use postgresql_embedded::PostgreSQL;
use tokio::runtime::Runtime;

use crate::domain::course::repository::CourseRepo;
use crate::domain::student::repository::StudentRepo;
use crate::domain::teacher::repository::TeacherRepo;
use crate::infrastructure::course::CoursePgRepo;
use crate::infrastructure::student::StudentPgRepo;
use crate::infrastructure::teacher::TeacherPgRepo;
use crate::presentation::courses::{self, CoursesState};
use crate::presentation::dashboard::{self, DashboardState};
use crate::presentation::Notifications;
use crate::presentation::render_notifications;
use crate::presentation::students::{self, StudentsState};
use crate::presentation::teachers::{self, TeachersState};
use crate::theme::{colors, sizes};

// ── Update state ──────────────────────────────────────────────────────────────

#[derive(Clone)]
pub enum UpdateState {
    Available(String),
    Downloading,
    Done,
    Failed(String),
}

// ── Init ──────────────────────────────────────────────────────────────────────

pub struct LoadingStatus {
    pub message:  String,
    pub progress: f32,
    pub result:   Option<Result<InitResult, String>>,
}

pub struct InitResult {
    pub pg:               PostgreSQL,
    pub client:           Client,
    pub rt:               Runtime,
    pub update_available: Option<UpdateState>,
}

// ── AppWrapper ────────────────────────────────────────────────────────────────

enum AppState {
    Loading(Arc<Mutex<LoadingStatus>>),
    Ready { app: App, pg: PostgreSQL, rt: Runtime },
    Failed(String),
}

pub struct AppWrapper {
    state: AppState,
}

impl AppWrapper {
    pub fn new(status: Arc<Mutex<LoadingStatus>>) -> Self {
        Self { state: AppState::Loading(status) }
    }
}

impl eframe::App for AppWrapper {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let should_transition = matches!(&self.state, AppState::Loading(s) if s.lock().unwrap().result.is_some());

        if should_transition {
            let old = std::mem::replace(&mut self.state, AppState::Failed(String::new()));
            if let AppState::Loading(s) = old {
                match s.lock().unwrap().result.take() {
                    Some(Ok(init)) => {
                        self.state = AppState::Ready {
                            app: App::new(Arc::new(Mutex::new(init.client)), init.update_available),
                            pg:  init.pg,
                            rt:  init.rt,
                        };
                    }
                    Some(Err(e)) => self.state = AppState::Failed(e),
                    None => unreachable!(),
                }
            }
        }

        match &mut self.state {
            AppState::Loading(status) => {
                let status = status.lock().unwrap();
                ui.ctx().request_repaint();
                let available_height = ui.available_height();
                ui.add_space(available_height / 3.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Babushka");
                    ui.add_space(sizes::SPACING_NORMAL);
                    ui.label(&status.message);
                    ui.add_space(sizes::SPACING_NORMAL);
                    ui.add(
                        egui::ProgressBar::new(status.progress)
                            .animate(true)
                            .desired_width(300.0),
                    );
                });
            }
            AppState::Ready { app, .. } => app.ui(ui, frame),
            AppState::Failed(e) => {
                let available_height = ui.available_height();
                ui.add_space(available_height / 3.0);
                ui.vertical_centered(|ui| {
                    ui.colored_label(colors::RED, format!("Error al iniciar: {e}"));
                });
            }
        }
    }

    fn on_exit(&mut self) {
        if let AppState::Ready { pg, rt, .. } = &mut self.state {
            rt.block_on(async { pg.stop().await.ok(); });
        }
    }
}

// ── App ───────────────────────────────────────────────────────────────────────

#[derive(PartialEq)]
enum View {
    Dashboard,
    Students,
    Courses,
    Teachers,
}

struct App {
    client:           Arc<Mutex<Client>>,
    course_repo:      Arc<dyn CourseRepo>,
    student_repo:     Arc<dyn StudentRepo>,
    teacher_repo:     Arc<dyn TeacherRepo>,
    current_view:     View,
    dashboard_state:  DashboardState,
    courses_state:    CoursesState,
    students_state:   StudentsState,
    teachers_state:   TeachersState,
    notifications:    Notifications,
    update_state:     Arc<Mutex<Option<UpdateState>>>,
}

impl App {
    fn new(client: Arc<Mutex<Client>>, update_available: Option<UpdateState>) -> Self {
        Self {
            course_repo:     Arc::new(CoursePgRepo::new(Arc::clone(&client))),
            student_repo:    Arc::new(StudentPgRepo::new(Arc::clone(&client))),
            teacher_repo:    Arc::new(TeacherPgRepo::new(Arc::clone(&client))),
            client,
            current_view:    View::Dashboard,
            dashboard_state: DashboardState::default(),
            courses_state:   CoursesState::default(),
            students_state:  StudentsState::default(),
            teachers_state:  TeachersState::default(),
            notifications:   Vec::new(),
            update_state:    Arc::new(Mutex::new(update_available)),
        }
    }

    fn render_update_banner(&mut self, ui: &mut egui::Ui) {
        let state = self.update_state.lock().unwrap().clone();
        let Some(state) = state else { return };

        let (color, msg, show_button) = match &state {
            UpdateState::Available(v) => (
                colors::YELLOW,
                format!("Nueva versión disponible: v{v}"),
                true,
            ),
            UpdateState::Downloading => (
                colors::YELLOW,
                "Descargando actualización… no cerrar la aplicación.".into(),
                false,
            ),
            UpdateState::Done => (
                colors::GREEN,
                "Actualización lista. Reiniciar para aplicar.".into(),
                false,
            ),
            UpdateState::Failed(e) => (
                colors::RED,
                format!("Error al actualizar: {e}"),
                false,
            ),
        };

        ui.horizontal(|ui| {
            ui.colored_label(color, msg);
            if show_button && ui.button("Actualizar").clicked() {
                let shared  = Arc::clone(&self.update_state);
                let ctx     = ui.ctx().clone();
                *shared.lock().unwrap() = Some(UpdateState::Downloading);
                ctx.request_repaint();
                std::thread::spawn(move || {
                    match crate::updater::apply() {
                        Ok(_)  => *shared.lock().unwrap() = Some(UpdateState::Done),
                        Err(e) => *shared.lock().unwrap() = Some(UpdateState::Failed(e.to_string())),
                    }
                    ctx.request_repaint();
                });
            }
        });
        ui.separator();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("menu")
            .default_size(100.0)
            .resizable(false)
            .frame(egui::Frame::new().fill(colors::BLACK).inner_margin(egui::Margin::same(sizes::MARGIN_NORMAL)))
            .show_inside(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    if ui.selectable_value(&mut self.current_view, View::Dashboard, "Panel").clicked() {
                        self.dashboard_state.needs_reload = true;
                    }
                    ui.selectable_value(&mut self.current_view, View::Students, "Alumnos");
                    ui.selectable_value(&mut self.current_view, View::Courses,  "Cursos");
                    ui.selectable_value(&mut self.current_view, View::Teachers, "Profesores");
                })
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(colors::BLACK).inner_margin(egui::Margin::same(sizes::MARGIN_NORMAL)))
            .show_inside(ui, |ui| {
                self.render_update_banner(ui);
                render_notifications(ui, &mut self.notifications);
                let navigate_to = match self.current_view {
                    View::Dashboard => dashboard::show(ui, &self.student_repo, &self.client, &mut self.dashboard_state, &mut self.notifications),
                    View::Courses   => { courses::show(ui, &self.course_repo,  &self.client, &mut self.courses_state,   &mut self.notifications); None }
                    View::Students  => { students::show(ui, &self.student_repo, &self.client, &mut self.students_state, &mut self.notifications); None }
                    View::Teachers  => { teachers::show(ui, &self.teacher_repo, &mut self.teachers_state, &mut self.notifications); None }
                };
                if let Some(id) = navigate_to {
                    use crate::application::student::dto::StudentDto;
                    match self.student_repo.get_by_id(id) {
                        Ok(student) => {
                            students::clear_detail_state(&mut self.students_state);
                            self.students_state.selected_student    = Some(StudentDto::from(&student));
                            self.students_state.needs_reload_ledger = true;
                            self.students_state.mode               = students::Mode::Detail;
                            self.current_view = View::Students;
                        }
                        Err(e) => crate::presentation::push_error(&mut self.notifications, e.to_string()),
                    }
                }
            });
    }
}
