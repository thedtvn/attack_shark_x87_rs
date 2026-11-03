//! Progress reporting for long operations such as saving the macro bank
//! (about 10 s over 2.4G).

/// Progress of a macro bank save: 0..=100, reported in order, never decreasing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveProgress {
    pub percent: u8,
}

/// Wraps a callback so reported percentages only increase.
pub(crate) struct Reporter<'a> {
    callback: &'a mut dyn FnMut(SaveProgress),
    last: Option<u8>,
}

impl<'a> Reporter<'a> {
    pub(crate) fn new(callback: &'a mut dyn FnMut(SaveProgress)) -> Self {
        Self { callback, last: None }
    }

    /// Report `fraction` (0..=1) of the work done.
    pub(crate) fn report(&mut self, fraction: f32) {
        let percent = (fraction.clamp(0.0, 1.0) * 100.0).round() as u8;
        if self.last.is_some_and(|last| percent <= last) {
            return;
        }
        self.last = Some(percent);
        (self.callback)(SaveProgress { percent });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reporter_is_monotonic_and_bounded() {
        let mut seen = Vec::new();
        let mut cb = |p: SaveProgress| seen.push(p.percent);
        let mut r = Reporter::new(&mut cb);
        for f in [0.0, 0.5, 0.25, 0.5, 9.0, 1.0] {
            r.report(f);
        }
        drop(r);
        assert_eq!(seen, vec![0, 50, 100]);
    }
}
