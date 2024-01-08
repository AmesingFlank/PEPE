use std::primitive;

use eframe::{
    egui::{self, CollapsingHeader, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::{
    editor::{Edit, GlobalEdit, MaskedEdit},
    ir::{Mask, MaskPrimitive, MaskTerm, RadialGradientMask},
    session::Session,
};

use super::{widgets::MaskIndicatorCallback, AppUiState};

pub fn masking(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState, edit: &mut Edit) {
    CollapsingHeader::new("Masking")
        .default_open(true)
        .show(ui, |ui| {
            ui.group(|ui| {
                let mut table = TableBuilder::new(ui)
                    .column(Column::auto())
                    .sense(egui::Sense::click())
                    .cell_layout(
                        egui::Layout::left_to_right(egui::Align::LEFT)
                            .with_cross_align(egui::Align::Center),
                    );
                let row_height = ui_state.last_frame_size.unwrap().1 * 0.04;
                let image_height = row_height * 0.8;
                table.body(|mut body| {
                    body.rows(row_height, edit.masked_edits.len(), |mut row| {
                        let i = row.index();
                        //row.set_selected(ui_state.selected_mask_index == i);
                        row.col(|ui| {
                            if ui.radio(ui_state.selected_mask_index == i, "").clicked() {
                                ui_state.selected_mask_index = i;
                            }
                            if let Some(ref result) = session.editor.current_result {
                                let mask_img = result.masked_edit_results[i].mask.clone();
                                let aspect_ratio = mask_img.aspect_ratio();
                                let image_width = image_height / aspect_ratio;
                                let size = egui::Vec2 {
                                    x: image_width,
                                    y: image_height,
                                };
                                let (rect, response) =
                                    ui.allocate_exact_size(size, egui::Sense::click_and_drag());
                                ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                                    rect,
                                    MaskIndicatorCallback {
                                        image: mask_img.clone(),
                                    },
                                ));
                            }
                            ui.label(&edit.masked_edits[i].name);
                        });
                        if row.response().clicked() {
                            ui_state.selected_mask_index = i;
                        }
                    });
                });
            });
            ui.menu_button("Create New Mask", |ui| {
                if ui.button("Radial Gradient").clicked() {
                    add_masked_edit(
                        edit,
                        MaskPrimitive::RadialGradient(RadialGradientMask::default()),
                    );
                    ui.close_menu();
                }
                if ui.button("Linear Gradient").clicked() {
                    ui.close_menu();
                }
            });
        });
}

fn add_masked_edit(edit: &mut Edit, primitive: MaskPrimitive) {
    let added_index = edit.masked_edits.len();
    let name = "Custom Mask ".to_string() + added_index.to_string().as_str();
    edit.masked_edits.push(MaskedEdit {
        mask: Mask {
            terms: vec![MaskTerm {
                primitive,
                inverted: false,
                subtracted: false,
            }],
        },
        edit: GlobalEdit::new(),
        name,
    });
}
