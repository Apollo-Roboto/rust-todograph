#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use rust_firework_core::{MindTask, MindTaskState, Point, TaskManager};
use slint::ComponentHandle;

mod ui {
    slint::include_modules!();
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

impl From<ui::MindTaskState> for MindTaskState {
    fn from(value: ui::MindTaskState) -> Self {
        match value {
            ui::MindTaskState::Todo => MindTaskState::Todo,
            ui::MindTaskState::Doing => MindTaskState::Doing,
            ui::MindTaskState::Done => MindTaskState::Done,
        }
    }
}

fn main() {
    let main_window = ui::AppWindow::new().unwrap();

    let task_manager = Arc::new(Mutex::new(TaskManager::default()));

    let main_window_weak = main_window.as_weak();
    let task_manager_clone = task_manager.clone();
    main_window.on_load(move |path| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let mut task_manager = task_manager_clone.lock().unwrap();

        main_window.set_task_loading_state(ui::TaskLoadingState::Loading);

        *task_manager = TaskManager::load(path.to_string()).unwrap();

        let tasks: Vec<ui::MindTask> = task_manager
            .tasks
            .iter()
            .map(|t| ui::MindTask {
                id: t.id as i32,
                state: t.state.into(),
                title: t.title.clone().into(),
                x: t.position.x,
                y: t.position.y,
                completion: task_manager.calc_progress(t.id).unwrap_or(0.0),
            })
            .collect();

        let tasks_model = std::rc::Rc::new(slint::VecModel::from(tasks));

        main_window.set_tasks(tasks_model.into());
    });

    let main_window_weak = main_window.as_weak();
    let task_manager_clone = task_manager.clone();
    main_window.on_save(move |path| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let task_manager = task_manager_clone.lock().unwrap();

        match task_manager.save(path.to_string()) {
            Ok(_) => {
                main_window.set_task_saving_state(ui::TaskSavingState::None);
            }
            Err(e) => {
                main_window.set_task_saving_state(ui::TaskSavingState::Error);
                main_window.set_task_saving_error_message(e.into());
            }
        };
    });

    let main_window_weak = main_window.as_weak();
    let task_manager_clone = task_manager.clone();
    main_window.on_delete_task(move |task_id| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let mut task_manager = task_manager_clone.lock().unwrap();

        task_manager.remove_task(task_id as u32);

        let tasks: Vec<ui::MindTask> = task_manager
            .tasks
            .iter()
            .map(|t| ui::MindTask {
                id: t.id as i32,
                state: t.state.into(),
                title: t.title.clone().into(),
                x: t.position.x,
                y: t.position.y,
                completion: task_manager.calc_progress(t.id).unwrap_or(0.0),
            })
            .collect();

        let tasks_model = std::rc::Rc::new(slint::VecModel::from(tasks));

        main_window.set_tasks(tasks_model.into());
    });

    let main_window_weak = main_window.as_weak();
    let task_manager_clone = task_manager.clone();
    main_window.on_add_task(move |title, x, y| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let mut task_manager = task_manager_clone.lock().unwrap();
        let id = task_manager.generate_id();

        task_manager.tasks.push(MindTask {
            id,
            position: Point { x, y },
            title: title.into(),
            creation_date: chrono::Utc::now(),
            ..Default::default()
        });

        let tasks: Vec<ui::MindTask> = task_manager
            .tasks
            .iter()
            .map(|t| ui::MindTask {
                id: t.id as i32,
                state: t.state.into(),
                title: t.title.clone().into(),
                x: t.position.x,
                y: t.position.y,
                completion: task_manager.calc_progress(t.id).unwrap_or(0.0),
            })
            .collect();

        let tasks_model = std::rc::Rc::new(slint::VecModel::from(tasks));

        main_window.set_tasks(tasks_model.into());
    });

    let main_window_weak = main_window.as_weak();
    let task_manager_clone = task_manager.clone();
    main_window.on_change_state(move |task_id, state| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let mut task_manager = task_manager_clone.lock().unwrap();

        task_manager.set_task_state(task_id as u32, state.into());

        let tasks: Vec<ui::MindTask> = task_manager
            .tasks
            .iter()
            .map(|t| ui::MindTask {
                id: t.id as i32,
                state: t.state.into(),
                title: t.title.clone().into(),
                x: t.position.x,
                y: t.position.y,
                completion: task_manager.calc_progress(t.id).unwrap_or(0.0),
            })
            .collect();

        let tasks_model = std::rc::Rc::new(slint::VecModel::from(tasks));

        main_window.set_tasks(tasks_model.into());
    });

    let main_window_weak = main_window.as_weak();
    let task_manager_clone = task_manager.clone();
    main_window.on_task_moved(move |task_id, x, y| {
        let Some(_main_window) = main_window_weak.upgrade() else {
            return;
        };

        let mut task_manager = task_manager_clone.lock().unwrap();
        if let Some(task) = task_manager
            .tasks
            .iter_mut()
            .find(|t| t.id == task_id as u32)
        {
            task.position = Point { x, y };
        }
    });

    main_window.run().unwrap();
}
