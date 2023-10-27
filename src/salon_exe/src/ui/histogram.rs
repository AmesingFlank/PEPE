use eframe::{egui::{Ui, CollapsingHeader}, epaint::Color32};
use egui_plot::{Line, Plot};
use salon_core::session::Session;

use super::AppUiState;

pub fn histogram(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    CollapsingHeader::new("Histogram")
        .default_open(true)
        .show(ui, |ui| {
            if let Some(ref result) = session.current_process_result {
                if let Some(ref stats) = result.data_for_editor {
                    let hist = &stats.histogram_final;

                    let get_line_data = |v: &Vec<u32>| {
                        let line_data: Vec<[f64; 2]> = (0..hist.num_bins)
                            .map(|i| [i as f64 / hist.num_bins as f64, v[i as usize] as f64])
                            .collect();
                        line_data
                    };

                    let r_line_data = get_line_data(&hist.r);
                    let g_line_data = get_line_data(&hist.g);
                    let b_line_data = get_line_data(&hist.b);
                    let luma_line_data = get_line_data(&hist.luma);

                    let r_line = Line::new(r_line_data)
                        .color(Color32::from_rgb(205, 25, 25))
                        .fill(0.0);
                    let g_line = Line::new(g_line_data)
                        .color(Color32::from_rgb(25, 205, 25))
                        .fill(0.0);
                    let b_line = Line::new(b_line_data)
                        .color(Color32::from_rgb(25, 25, 205))
                        .fill(0.0);
                    let luma_line = Line::new(luma_line_data)
                        .color(Color32::from_rgb(200, 200, 200))
                        .fill(0.0);

                    let img_dim = result.final_image.as_ref().unwrap().properties.dimensions;
                    let num_pixels = img_dim.0 * img_dim.1;
                    let mut y_top = hist.max_value() as f32;
                    y_top = y_top.min(10.0 * num_pixels as f32 / hist.num_bins as f32);

                    let plot = Plot::new("histogram")
                        .height(ui_state.last_frame_size.unwrap().1 * 0.1)
                        .show_x(false)
                        .show_y(false)
                        .allow_zoom(false)
                        .allow_scroll(false)
                        .allow_double_click_reset(false)
                        .allow_drag(false)
                        .include_x(0.0)
                        .include_x(1.0)
                        .include_y(0.0)
                        .include_y(y_top)
                        .show_axes([false, false])
                        .show_grid([false, false]);
                    plot.show(ui, |plot_ui| {
                        plot_ui.line(r_line);
                        plot_ui.line(g_line);
                        plot_ui.line(b_line);
                        plot_ui.line(luma_line);
                    });
                }
            }
        });
}
