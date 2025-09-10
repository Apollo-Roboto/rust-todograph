#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use rust_firework_core::commands::{
    Command, CreateTaskCommand, DeleteTaskCommand, SetTaskParentCommand, SetTaskPositionCommand,
    SetTaskStateCommand, TaskCommandHistory,
};
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
    let command_history = Arc::new(Mutex::new(TaskCommandHistory::default()));

    let main_window_weak = main_window.as_weak();
    let task_manager_clone = task_manager.clone();
    main_window.on_refresh_other(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let task_manager = task_manager_clone.lock().unwrap();

        // get the tasks
        let tasks: Vec<ui::MindTask> = task_manager
            .tasks
            .iter()
            .map(|t| ui::MindTask {
                id: t.id as i32,
                state: t.state.into(),
                title: t.title.clone().into(),
                x: t.pos.x,
                y: t.pos.y,
                completion: task_manager.calc_progress(t.id).unwrap_or(0.0),
            })
            .collect();
        let tasks_model = std::rc::Rc::new(slint::VecModel::from(tasks));

        main_window.set_tasks(tasks_model.into());
        main_window.set_overall_progress(task_manager.calc_progress_all().unwrap_or(0.0));
    });

    let main_window_weak = main_window.as_weak();
    let task_manager_clone = task_manager.clone();
    main_window.on_refresh_edges(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let task_manager = task_manager_clone.lock().unwrap();

        // get the positions of the edges
        let edges: Vec<ui::MindEdge> = task_manager
            .get_all_edges()
            .iter()
            .map(|(from, to)| ui::MindEdge {
                from_x: from.x,
                from_y: from.y,
                to_x: to.x,
                to_y: to.y,
            })
            .collect();
        let edges_model = std::rc::Rc::new(slint::VecModel::from(edges));

        main_window.set_edges(edges_model.into());
    });

    let main_window_weak = main_window.as_weak();
    let task_manager_clone = task_manager.clone();
    let command_history_clone = command_history.clone();
    main_window.on_load(move |path| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let mut task_manager = task_manager_clone.lock().unwrap();
        let mut command_history = command_history_clone.lock().unwrap();

        // no commands in the history is relevent after loading a project
        command_history.clear();

        main_window.set_task_loading_state(ui::TaskLoadingState::Loading);

        *task_manager = TaskManager::load(&path).unwrap();

        // avoid deadlock from the next invoke
        std::mem::drop(task_manager);

        main_window.invoke_refresh_all();
    });

    let main_window_weak = main_window.as_weak();
    let task_manager_clone = task_manager.clone();
    main_window.on_save(move |path| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let task_manager = task_manager_clone.lock().unwrap();

        match task_manager.save(&path) {
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
    let command_history_clone = command_history.clone();
    main_window.on_undo(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let mut task_manager = task_manager_clone.lock().unwrap();
        let mut command_history = command_history_clone.lock().unwrap();

        command_history.undo(&mut task_manager).unwrap();

        // avoid deadlock from the next invoke
        std::mem::drop(task_manager);
        std::mem::drop(command_history);

        main_window.invoke_refresh_all();
    });

    let main_window_weak = main_window.as_weak();
    let task_manager_clone = task_manager.clone();
    let command_history_clone = command_history.clone();
    main_window.on_redo(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let mut task_manager = task_manager_clone.lock().unwrap();
        let mut command_history = command_history_clone.lock().unwrap();

        command_history.redo(&mut task_manager).unwrap();

        // avoid deadlock from the next invoke
        std::mem::drop(task_manager);
        std::mem::drop(command_history);

        main_window.invoke_refresh_all();
    });

    let main_window_weak = main_window.as_weak();
    let task_manager_clone = task_manager.clone();
    let command_history_clone = command_history.clone();
    main_window.on_delete_task(move |task_id| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let mut task_manager = task_manager_clone.lock().unwrap();
        let mut command_history = command_history_clone.lock().unwrap();

        if let Some(task) = task_manager.tasks.iter().find(|t| t.id == task_id as u32) {
            let cmd = Box::new(DeleteTaskCommand::new(task.clone()));
            command_history.execute(cmd, &mut task_manager).unwrap();
        }

        // avoid deadlock from the next invoke
        std::mem::drop(task_manager);
        std::mem::drop(command_history);

        main_window.invoke_refresh_all();
    });

    let main_window_weak = main_window.as_weak();
    let task_manager_clone = task_manager.clone();
    let command_history_clone = command_history.clone();
    main_window.on_create_task(move |title, x, y| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let mut task_manager = task_manager_clone.lock().unwrap();
        let mut command_history = command_history_clone.lock().unwrap();

        let id = task_manager.generate_id();

        let task = MindTask {
            id,
            pos: Point { x, y },
            title: title.into(),
            creation_date: chrono::Utc::now(),
            ..Default::default()
        };

        let cmd = Box::new(CreateTaskCommand::new(task));
        command_history.execute(cmd, &mut task_manager).unwrap();

        // avoid deadlock from the next invoke
        std::mem::drop(task_manager);
        std::mem::drop(command_history);

        main_window.invoke_refresh_all();
    });

    let main_window_weak = main_window.as_weak();
    let task_manager_clone = task_manager.clone();
    let command_history_clone = command_history.clone();
    main_window.on_change_state(move |task_id, state| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let mut task_manager = task_manager_clone.lock().unwrap();
        let mut command_history = command_history_clone.lock().unwrap();

        let cmd = Box::new(SetTaskStateCommand::new(task_id as u32, state.into()));
        command_history.execute(cmd, &mut task_manager).unwrap();

        // avoid deadlock from the next invoke
        std::mem::drop(task_manager);
        std::mem::drop(command_history);

        main_window.invoke_refresh_all();
    });

    let main_window_weak = main_window.as_weak();
    let task_manager_clone = task_manager.clone();
    let command_history_clone = command_history.clone();
    main_window.on_task_moved(move |task_id, x, y| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let mut task_manager = task_manager_clone.lock().unwrap();
        let mut _command_history = command_history_clone.lock().unwrap();

        let mut cmd = Box::new(SetTaskPositionCommand::new(task_id as u32, Point { x, y }));
        // TODO: this is called too often, I need to save only when it was dropped into the command history
        // for now I'll just not have the movement in the history at all
        // command_history.execute(cmd, &mut task_manager).unwrap();
        cmd.execute(&mut task_manager).unwrap();

        // avoid deadlock from the next invoke
        std::mem::drop(task_manager);
        std::mem::drop(_command_history);

        main_window.invoke_refresh_edges();
    });

    let main_window_weak = main_window.as_weak();
    let task_manager_clone = task_manager.clone();
    let command_history_clone = command_history.clone();
    main_window.on_set_parent_to_task(move |task_id, parent_id| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let mut task_manager = task_manager_clone.lock().unwrap();
        let mut command_history = command_history_clone.lock().unwrap();

        let Ok(task_id) = task_id.try_into() else {
            return;
        };
        let Ok(parent_id) = parent_id.try_into() else {
            return;
        };

        let cmd = Box::new(SetTaskParentCommand::new(task_id, Some(parent_id)));
        command_history.execute(cmd, &mut task_manager).unwrap();

        // avoid deadlock from the next invoke
        std::mem::drop(task_manager);
        std::mem::drop(command_history);

        main_window.invoke_refresh_all();
    });

    let main_window_weak = main_window.as_weak();
    let task_manager_clone = task_manager.clone();
    let command_history_clone = command_history.clone();
    main_window.on_unset_parent_from_task(move |task_id| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let mut task_manager = task_manager_clone.lock().unwrap();
        let mut command_history = command_history_clone.lock().unwrap();

        let cmd = Box::new(SetTaskParentCommand::new(task_id as u32, None));
        command_history.execute(cmd, &mut task_manager).unwrap();

        // avoid deadlock from the next invoke
        std::mem::drop(task_manager);
        std::mem::drop(command_history);

        main_window.invoke_refresh_all();
    });

    main_window.run().unwrap();
}
