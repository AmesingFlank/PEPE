use super::{AddedImage, AppUiState};
use eframe::{
    egui::{self, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::{
    runtime::{ColorSpace, ImageFormat, ImageReaderJpeg, Runtime},
    session::Session,
};
use std::{future::Future, ops::Add, sync::Arc};

pub fn file_menu(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    ui.menu_button("File", |ui| {
        if ui.button("Import Image").clicked() {
            ui.close_menu();
            ui_state.import_image_dialog.open();
        }

        if ui.button("Export Image").clicked() {
            ui.close_menu();
            let ctx = ui.ctx().clone();
            file_dialogue_export_image(ctx, session, ui_state);
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn file_dialogue_export_image(
    context: egui::Context,
    session: &mut Session,
    ui_state: &mut AppUiState,
) {
    let task = rfd::AsyncFileDialog::new()
        .add_filter("extension", &["jpg"])
        .save_file();
    let runtime = session.runtime.clone();

    if let Some(ref input_img) = session.editor.current_input_image {
        let result = session
            .editor
            .execute_current_edit_original_scale(input_img.clone());
        let final_image = result.final_image.clone();
        let final_image = session
            .toolbox
            .convert_color_space(final_image, ColorSpace::sRGB);
        let final_image = session
            .toolbox
            .convert_image_format(final_image, ImageFormat::Rgba8Unorm);
        let mut image_reader =
            ImageReaderJpeg::new(runtime.clone(), session.toolbox.clone(), final_image);
        execute(async move {
            let file = task.await;
            let jpeg_data = image_reader.await_jpeg_data().await;
            if let Some(file) = file {
                file.write(&jpeg_data).await.expect("Write file failed");
            }
        });
    }
}

#[cfg(target_arch = "wasm32")]
fn file_dialogue_export_image(
    context: egui::Context,
    session: &mut Session,
    ui_state: &mut AppUiState,
) {
    let runtime = session.runtime.clone();

    if let Some(ref input_img) = session.editor.current_input_image {
        let result = session
            .editor
            .execute_current_edit_original_scale(input_img.clone());
        let final_image = result.final_image.clone();
        let final_image = session
            .toolbox
            .convert_color_space(final_image, ColorSpace::sRGB);
        let final_image = session
            .toolbox
            .convert_image_format(final_image, ImageFormat::Rgba8Unorm);
        let mut image_reader =
            ImageReaderJpeg::new(runtime.clone(), session.toolbox.clone(), final_image);
        execute(async move {
            let jpeg_data = image_reader.await_jpeg_data().await;
            let array = Uint8Array::from(jpeg_data.as_slice());
            let blob_parts = Array::new();
            blob_parts.push(&array.buffer());

            log::info!("downloading now!");

            let file = File::new_with_blob_sequence_and_options(
                &blob_parts.into(),
                "output.jpg",
                web_sys::FilePropertyBag::new().type_("application/octet-stream"),
            )
            .unwrap();
            let url = Url::create_object_url_with_blob(&file);
            if let Some(window) = web_sys::window() {
                window.location().set_href(&url.unwrap()).ok();
            }
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    std::thread::spawn(move || futures::executor::block_on(f));
}

#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use js_sys::{Array, ArrayBuffer, Uint8Array};
#[cfg(target_arch = "wasm32")]
use web_sys::{window, Blob, File, FileReader, HtmlInputElement, Url};

#[cfg(target_arch = "wasm32")]
pub struct ImageImportDialog {
    channel: (
        std::sync::mpsc::Sender<AddedImage>,
        std::sync::mpsc::Receiver<AddedImage>,
    ),
    runtime: Arc<Runtime>,
    context: egui::Context,
    input: HtmlInputElement,
    closure: Option<Closure<dyn FnMut()>>,
}

#[cfg(target_arch = "wasm32")]
impl Drop for ImageImportDialog {
    fn drop(&mut self) {
        self.input.remove();
        if self.closure.is_some() {
            std::mem::replace(&mut self.closure, None).unwrap().forget();
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl ImageImportDialog {
    pub fn new(runtime: Arc<Runtime>, context: egui::Context) -> Self {
        let document = window().unwrap().document().unwrap();
        let body = document.body().unwrap();
        let input = document
            .create_element("input")
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();
        input.set_attribute("type", "file").unwrap();
        input.style().set_property("display", "none").unwrap();
        body.append_child(&input).unwrap();

        Self {
            channel: std::sync::mpsc::channel(),
            runtime,
            context,

            input,
            closure: None,
        }
    }

    pub fn open(&mut self) {
        if let Some(closure) = &self.closure {
            self.input
                .remove_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
                .unwrap();
            std::mem::replace(&mut self.closure, None).unwrap().forget();
        }

        let runtime = self.runtime.clone();
        let context = self.context.clone();
        let sender = self.channel.0.clone();
        let input_clone = self.input.clone();

        let closure = Closure::once(move || {
            if let Some(file) = input_clone.files().and_then(|files| files.get(0)) {
                let file_name = file.name();
                let file_name_parts: Vec<&str> = file_name.split(".").collect();
                let ext = file_name_parts.last().unwrap().to_string();

                let reader = FileReader::new().unwrap();
                let reader_clone = reader.clone();
                let onload_closure = Closure::once(Box::new(move || {
                    let array_buffer = reader_clone
                        .result()
                        .unwrap()
                        .dyn_into::<ArrayBuffer>()
                        .unwrap();
                    let image_data = Uint8Array::new(&array_buffer).to_vec();
                    let image = runtime
                        .create_image_from_bytes_and_extension(image_data.as_slice(), ext.as_str());
                    match image {
                        Ok(img) => {
                            let added_img = AddedImage {
                                image: Arc::new(img),
                            };
                            let _ = sender.send(added_img);
                            context.request_repaint();
                        }
                        Err(_) => {}
                    }
                }));

                reader.set_onload(Some(onload_closure.as_ref().unchecked_ref()));
                reader.read_as_array_buffer(&file).unwrap();
                onload_closure.forget();
            }
        });

        self.input
            .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
            .unwrap();
        self.closure = Some(closure);
        self.input.click();
    }

    pub fn get_added_image(&mut self) -> Option<AddedImage> {
        if let Ok(added_image) = self.channel.1.try_recv() {
            Some(added_image)
        } else {
            None
        }
    }
}

// native

#[cfg(not(target_arch = "wasm32"))]
pub struct ImageImportDialog {
    channel: (
        std::sync::mpsc::Sender<AddedImage>,
        std::sync::mpsc::Receiver<AddedImage>,
    ),
    runtime: Arc<Runtime>,
    context: egui::Context,
}

#[cfg(not(target_arch = "wasm32"))]
impl ImageImportDialog {
    pub fn new(runtime: Arc<Runtime>, context: egui::Context) -> Self {
        Self {
            channel: std::sync::mpsc::channel(),
            runtime,
            context,
        }
    }

    pub fn open(&mut self) {
        let task = rfd::AsyncFileDialog::new()
            .add_filter("extension", &["png", "jpg", "jpeg"])
            .pick_file();
        let runtime = self.runtime.clone();
        let context = self.context.clone();

        let sender = self.channel.0.clone();

        execute(async move {
            let file = task.await;
            if let Some(file) = file {
                let file_name = file.file_name();
                let file_name_parts: Vec<&str> = file_name.split(".").collect();
                let ext = file_name_parts.last().unwrap().to_owned();

                let image_data = file.read().await;
                let image =
                    runtime.create_image_from_bytes_and_extension(image_data.as_slice(), ext);
                match image {
                    Ok(img) => {
                        let added_img = AddedImage {
                            image: Arc::new(img),
                        };
                        let _ = sender.send(added_img);
                        context.request_repaint();
                    }
                    Err(_) => {}
                }
            }
        });
    }

    pub fn get_added_image(&mut self) -> Option<AddedImage> {
        if let Ok(added_image) = self.channel.1.try_recv() {
            Some(added_image)
        } else {
            None
        }
    }
}
