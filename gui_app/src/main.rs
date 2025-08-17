#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rust_firework_core::{MindTask, MindTaskState};
use slint::{ComponentHandle, SharedString};

mod ui {
    slint::include_modules!();
}

impl From<MindTask> for ui::MindTask {
    fn from(_value: MindTask) -> Self {
        todo!()
    }
}

impl From<MindTaskState> for ui::MindTaskState {
    fn from(value: MindTaskState) -> Self {
        match value {
            MindTaskState::Todo => ui::MindTaskState::Todo,
            MindTaskState::Doing => ui::MindTaskState::Doing,
            MindTaskState::Done => ui::MindTaskState::Done,
        }
    }
}

fn main() {
    let main_window = ui::AppWindow::new().unwrap();

    let main_window_weak = main_window.as_weak();
    main_window.on_load(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let tasks: Vec<ui::MindTask> = vec![
            ui::MindTask {
                state: ui::MindTaskState::Todo,
                title: SharedString::from("I have a thing to do"),
                x: 0.,
                y: -100.,
            },
            ui::MindTask {
                state: ui::MindTaskState::Doing,
                title: SharedString::from("I'm doing great"),
                x: 0.,
                y: -150.,
            },
            ui::MindTask {
                state: ui::MindTaskState::Done,
                title: SharedString::from("Hell yeah"),
                x: 0.,
                y: -200.,
            },
        ];

        let tasks_model = std::rc::Rc::new(slint::VecModel::from(tasks));

        main_window.set_tasks(tasks_model.into());
    });

    main_window.run().unwrap();
}
