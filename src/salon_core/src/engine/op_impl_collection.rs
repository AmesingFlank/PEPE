use super::ops::{exposure::AdjustExposureImpl, saturation::AdjustSaturationImpl, histogram::ComputeHistogramImpl, collect_statistics::CollectStatisticsImpl, contrast::AdjustContrastImpl, basic_statistics::ComputeBasicStatisticsImpl};

#[derive(Default)]
pub struct OpImplCollection {
    pub exposure: Option<AdjustExposureImpl>,
    pub contrast: Option<AdjustContrastImpl>,
    pub saturation: Option<AdjustSaturationImpl>,
    pub basic_statistics: Option<ComputeBasicStatisticsImpl>,
    pub histogram: Option<ComputeHistogramImpl>,
    pub collect_statistics: Option<CollectStatisticsImpl>,
}

impl OpImplCollection {
    pub fn new() -> Self {
        OpImplCollection {
            ..Default::default()
        }
    }
}
