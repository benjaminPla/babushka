use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;
use postgresql_embedded::PostgreSQL;
use tokio::runtime::Runtime;

use crate::presentation::{
    render_notifications,
    courses::{self, CoursesState},
    enrollments::{self, EnrollmentsState},
    payments::{self, PaymentsState},
    students::{self, StudentsState},
    teachers::{self, TeachersState},
    Notifications,
};

pub struct LoadingStatus {
    pub message:  String,
    pub progress: f32,
    pub result:   Option<Result<InitResult, String>>,
}

pub struct InitResult {
    pub pg:     PostgreSQL,
    pub client: Client,
    pub rt:     Runtime,
}

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
        let should_transition = match &self.state {
            AppState::Loading(s) => s.lock().unwrap().result.is_some(),
            _ => false,
        };

        if should_transition {
            let old = std::mem::replace(&mut self.state, AppState::Failed(String::new()));
            if let AppState::Loading(s) = old {
                match s.lock().unwrap().result.take() {
                    Some(Ok(init)) => {
                        self.state = AppState::Ready {
                            app: App::new(Arc::new(Mutex::new(init.client))),
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
                    ui.add_space(8.0);
                    ui.label(&status.message);
                    ui.add_space(8.0);
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
                    ui.colored_label(egui::Color32::RED, format!("Error al iniciar: {e}"));
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

#[derive(PartialEq)]
enum View {
    Courses,
    Enrollments,
    Payments,
    Students,
    Teachers,
}

struct App {
    client:             Arc<Mutex<Client>>,
    current_view:       View,
    courses_state:      CoursesState,
    enrollments_state:  EnrollmentsState,
    payments_state:     PaymentsState,
    students_state:     StudentsState,
    teachers_state:     TeachersState,
    notifications:      Notifications,
}

impl App {
    fn new(client: Arc<Mutex<Client>>) -> Self {
        Self {
            client,
            current_view:      View::Courses,
            courses_state:     CoursesState::default(),
            enrollments_state: EnrollmentsState::default(),
            payments_state:    PaymentsState::default(),
            students_state:    StudentsState::default(),
            teachers_state:    TeachersState::default(),
            notifications:     Vec::new(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        use crate::theme::{colors, panel_frame};

        egui::Panel::left("menu")
            .frame(panel_frame(colors::SIDEBAR))
            .show_inside(ui, |ui| {
                ui.heading("Babushka");
                ui.add_space(2.0);
                ui.separator();
                ui.add_space(4.0);
                ui.selectable_value(&mut self.current_view, View::Courses,     "Cursos");
                ui.selectable_value(&mut self.current_view, View::Enrollments, "Inscripciones");
                ui.selectable_value(&mut self.current_view, View::Payments,    "Pagos");
                ui.selectable_value(&mut self.current_view, View::Teachers,    "Profesores");
                ui.selectable_value(&mut self.current_view, View::Students,    "Alumnos");
            });

        egui::CentralPanel::default()
            .frame(panel_frame(colors::BACKGROUND))
            .show_inside(ui, |ui| {
                render_notifications(ui, &mut self.notifications);
                match self.current_view {
                    View::Courses     => courses::show(ui, &self.client, &mut self.courses_state,         &mut self.notifications),
                    View::Enrollments => enrollments::show(ui, &self.client, &mut self.enrollments_state, &mut self.notifications),
                    View::Payments    => payments::show(ui, &self.client, &mut self.payments_state,       &mut self.notifications),
                    View::Teachers    => teachers::show(ui, &self.client, &mut self.teachers_state,       &mut self.notifications),
                    View::Students    => students::show(ui, &self.client, &mut self.students_state,       &mut self.notifications),
                }
            });
    }
}
