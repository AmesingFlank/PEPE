use super::{
    file_dialogues::{file_dialogue_export_edit, file_dialogue_export_image},
    AddedImageOrAlbum, AppPage, AppUiState,
};
use eframe::{
    egui::{self, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::{
    library::{LibraryImageIdentifier, LibraryImageMetaData},
    runtime::{ColorSpace, ImageFormat, ImageReaderJpeg, Runtime, Toolbox},
    session::Session,
};
use std::{future::Future, ops::Add, sync::Arc};

pub fn file_menu(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    ui.menu_button("File", |ui| {
        if ui
            .add_enabled(true, egui::Button::new("Import Image"))
            .clicked()
        {
            ui.close_menu();
            ui_state.import_image_dialog.open_pick_images();
        }

        #[cfg(not(target_arch = "wasm32"))]
        if ui
            .add_enabled(true, egui::Button::new("Import Folder as Album"))
            .clicked()
        {
            ui.close_menu();
            ui_state.import_image_dialog.open_pick_folder();
        }

        let has_current_img = session.editor.current_edit_context_ref().is_some();
        if ui
            .add_enabled(has_current_img, egui::Button::new("Export Editted Image"))
            .clicked()
        {
            ui.close_menu();
            let ctx = ui.ctx().clone();
            //ui_state.app_page = AppPage::Export;
            file_dialogue_export_image(ctx, session, ui_state);
        }

        if ui
            .add_enabled(has_current_img, egui::Button::new("Export Edit JSON"))
            .clicked()
        {
            ui.close_menu();
            let ctx = ui.ctx().clone();
            file_dialogue_export_edit(ctx, session, ui_state);
        }
    });
}
