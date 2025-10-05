use std::path::Path;

use crate::ui::{
    FileDialogButtonGlobal, FileDialogResult, FileDialogResultStatus, FileDialogType,
    HyperlinkGlobal,
};
use log::error;
use slint::Model;

pub fn setup_hyperlink(widget: HyperlinkGlobal) {
    widget.on_open(|path| {
        if let Err(e) = open::that(&path) {
            error!("Error opening path: {path} {:?}", e);
        }
    })
}

pub fn setup_file_dialog_button(widget: FileDialogButtonGlobal) {
    widget.on_open(|title, directory, filters, default_file, dialog_type| {
        let mut dialog = rfd::FileDialog::new()
            .set_title(title.to_string())
            .set_directory(Path::new(&directory.to_string()))
            .set_file_name(default_file.to_string());

        for f in filters.iter() {
            let extensions: Vec<String> = f.extensions.iter().map(|e| e.to_string()).collect();
            dialog = dialog.add_filter(f.text, &extensions);
        }

        let picked_file = match dialog_type {
            FileDialogType::Load => dialog.pick_file(),
            FileDialogType::Save => dialog.save_file(),
        };

        match picked_file {
            Some(path) => FileDialogResult {
                file: path
                    .to_str()
                    .map_or(slint::SharedString::new(), |p| p.into()),
                status: FileDialogResultStatus::Picked,
            },
            None => FileDialogResult {
                file: slint::SharedString::new(),
                status: FileDialogResultStatus::Closed,
            },
        }
    });
}
