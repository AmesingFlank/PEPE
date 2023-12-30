use crate::ir::{ColorMixGroup, GlobalMask, Mask};

use crate::utils::rectangle::Rectangle;

#[derive(Clone, PartialEq)]
pub struct Edit {
    pub crop: Option<Rectangle>,
    pub masked_edits: Vec<MaskedEdit>,
}

impl Edit {
    pub fn new() -> Self {
        Self {
            crop: None,
            masked_edits: vec![MaskedEdit::new(
                Mask::Global(GlobalMask {}),
                GlobalEdit::new(),
            )],
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct MaskedEdit {
    pub mask: Mask,
    pub edit: GlobalEdit,
}

impl MaskedEdit {
    pub fn new(mask: Mask, edit: GlobalEdit) -> Self {
        Self { mask, edit }
    }
}

#[derive(Clone, PartialEq)]
pub struct GlobalEdit {
    pub exposure: f32,
    pub contrast: f32,
    pub highlights: f32,
    pub shadows: f32,

    pub curve_control_points_all: Vec<(f32, f32)>,
    pub curve_control_points_r: Vec<(f32, f32)>,
    pub curve_control_points_g: Vec<(f32, f32)>,
    pub curve_control_points_b: Vec<(f32, f32)>,

    pub temperature: f32,
    pub tint: f32,
    pub vibrance: f32,
    pub saturation: f32,

    pub color_mixer_edits: [ColorMixGroup; 8],

    pub dehaze: f32,
    pub vignette: f32,
}

impl GlobalEdit {
    pub fn new() -> Self {
        GlobalEdit {
            exposure: 0.0,
            contrast: 0.0,
            highlights: 0.0,
            shadows: 0.0,

            curve_control_points_all: GlobalEdit::initial_control_points(),
            curve_control_points_r: GlobalEdit::initial_control_points(),
            curve_control_points_g: GlobalEdit::initial_control_points(),
            curve_control_points_b: GlobalEdit::initial_control_points(),

            temperature: 0.0,
            tint: 0.0,
            vibrance: 0.0,
            saturation: 0.0,

            color_mixer_edits: [ColorMixGroup::new(); 8],

            dehaze: 0.0,
            vignette: 0.0,
        }
    }

    pub fn initial_control_points() -> Vec<(f32, f32)> {
        vec![(0.0, 0.0), (1.0, 1.0)]
    }
}