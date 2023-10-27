use super::Id;

#[derive(Clone)]
pub enum Op {
    Input(InputOp),
    AdjustExposure(AdjustExposureOp),
    AdjustContrast(AdjustContrastOp),
    AdjustHighlightsAndShadows(AdjustHighlightsAndShadowsOp),
    AdjustTemperatureAndTint(AdjustTemperatureAndTintOp),
    AdjustVibrance(AdjustVibranceOp),
    AdjustSaturation(AdjustSaturationOp),
    ComputeBasicStatistics(ComputeBasicStatisticsOp),
    ComputeHistogram(ComputeHistogramOp),
    CollectDataForEditor(CollectDataForEditorOp),
}

#[derive(Clone)]
pub struct InputOp {
    pub result: Id,
}

#[derive(Clone)]
pub struct AdjustExposureOp {
    pub result: Id,
    pub arg: Id,
    pub exposure: f32,
}

#[derive(Clone)]
pub struct AdjustContrastOp {
    pub result: Id,
    pub arg: Id,
    pub basic_stats: Id,
    pub contrast: f32,
}

#[derive(Clone)]
pub struct AdjustHighlightsAndShadowsOp {
    pub result: Id,
    pub arg: Id,
    pub highlights: f32,
    pub shadows: f32,
}

// grouping temp and tint together, because they are heavy and shares a lot of common work
#[derive(Clone)]
pub struct AdjustTemperatureAndTintOp {
    pub result: Id,
    pub arg: Id,
    pub temperature: f32,
    pub tint: f32,
}

#[derive(Clone)]
pub struct AdjustVibranceOp {
    pub result: Id,
    pub arg: Id,
    pub vibrance: f32,
}


#[derive(Clone)]
pub struct AdjustSaturationOp {
    pub result: Id,
    pub arg: Id,
    pub saturation: f32,
}

#[derive(Clone)]
pub struct ComputeBasicStatisticsOp {
    pub result: Id,
    pub arg: Id,
}

#[derive(Clone)]
pub struct ComputeHistogramOp {
    pub result: Id,
    pub arg: Id,
}

#[derive(Clone)]
pub struct CollectDataForEditorOp {
    pub result: Id,
    pub histogram_final: Id,
}
