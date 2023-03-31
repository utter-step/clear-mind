use itertools::Itertools;

/// Info about gap
#[derive(Debug, Clone, Copy)]
pub struct GapInfo {
    /// Start of a gap (sample index)
    pub start: usize,
    /// Length of a gap (in samples)
    pub length: usize,
    /// Sample rate per second
    pub sample_rate: usize,
}

impl GapInfo {
    /// Find two gaps with the biggest gap between them
    ///
    /// # Assumptions
    /// Assumes that gaps are ordered by their start time!
    pub fn find_boundary_gaps(gaps: &[GapInfo]) -> Option<(GapInfo, GapInfo)> {
        gaps.iter()
            .tuple_windows()
            .max_by_key(|(&left, &right)| right.start - (left.start + left.length))
            .map(|(&left, &right)| (left, right))
    }
}
