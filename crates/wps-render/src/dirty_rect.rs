use wps_core::Rect;

#[derive(Debug, Clone, Default)]
pub struct DirtyRectTracker {
    dirty_rects: Vec<Rect>,
}

impl DirtyRectTracker {
    pub fn new() -> Self {
        Self {
            dirty_rects: Vec::new(),
        }
    }

    pub fn mark_dirty(&mut self, rect: Rect) {
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }

        // Try to merge with an existing overlapping or near rect
        for existing in &mut self.dirty_rects {
            if existing.intersects(&rect) {
                *existing = existing.union(&rect);
                return;
            }
        }

        self.dirty_rects.push(rect);
    }

    pub fn is_dirty(&self) -> bool {
        !self.dirty_rects.is_empty()
    }

    pub fn clear(&mut self) {
        self.dirty_rects.clear();
    }

    pub fn get_combined_bounds(&self) -> Option<Rect> {
        if self.dirty_rects.is_empty() {
            return None;
        }

        let mut combined = self.dirty_rects[0];
        for r in &self.dirty_rects[1..] {
            combined = combined.union(r);
        }
        Some(combined)
    }

    pub fn rects(&self) -> &[Rect] {
        &self.dirty_rects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirty_rect_merging() {
        let mut tracker = DirtyRectTracker::new();
        assert!(!tracker.is_dirty());

        tracker.mark_dirty(Rect::new(0.0, 0.0, 50.0, 50.0));
        tracker.mark_dirty(Rect::new(25.0, 25.0, 50.0, 50.0));

        assert_eq!(tracker.rects().len(), 1);
        assert_eq!(tracker.rects()[0], Rect::new(0.0, 0.0, 75.0, 75.0));

        tracker.clear();
        assert!(!tracker.is_dirty());
    }
}
