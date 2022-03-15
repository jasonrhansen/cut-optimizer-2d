/// Implementation of the Maximal Rectangles Algorithms for bin packing.
/// [A Thousand Ways to Pack the Bin](http://pds25.egloos.com/pds/201504/21/98/RectangleBinPack.pdf)
use super::*;
use crate::guillotine::RotateCutPieceHeuristic;

use rand::distributions::{Distribution, Standard};
use rand::prelude::*;

use std::borrow::Borrow;
use std::cmp;

/// Heuristics for deciding which of the free rectangles to place the demand piece in.
#[derive(Copy, Clone)]
pub(crate) enum FreeRectChoiceHeuristic {
    BestShortSideFit,
    BestLongSideFit,
    BestAreaFit,
    BottomLeftRule,
    ContactPointRule,
}

impl Distribution<FreeRectChoiceHeuristic> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> FreeRectChoiceHeuristic {
        match rng.gen_range(0..5) {
            0 => FreeRectChoiceHeuristic::BestShortSideFit,
            1 => FreeRectChoiceHeuristic::BestLongSideFit,
            2 => FreeRectChoiceHeuristic::BestAreaFit,
            3 => FreeRectChoiceHeuristic::BottomLeftRule,
            _ => FreeRectChoiceHeuristic::ContactPointRule,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct MaxRectsBin {
    width: usize,
    length: usize,
    blade_width: usize,
    pattern_direction: PatternDirection,
    cut_pieces: Vec<UsedCutPiece>,
    free_rects: Vec<Rect>,
    price: usize,
}

impl Bin for MaxRectsBin {
    type Heuristic = (FreeRectChoiceHeuristic, RotateCutPieceHeuristic);

    fn new(
        width: usize,
        length: usize,
        blade_width: usize,
        pattern_direction: PatternDirection,
        price: usize,
    ) -> Self {
        // We start with a single big free rectangle that spans the whole bin.
        let free_rect = Rect {
            x: 0,
            y: 0,
            width,
            length,
        };

        let free_rects = vec![free_rect];

        MaxRectsBin {
            width,
            length,
            free_rects,
            blade_width,
            pattern_direction,
            cut_pieces: Default::default(),
            price,
        }
    }

    fn fitness(&self) -> f64 {
        // We don't want cut loss from the blade width to penalize the fitness
        // so we calculate the used area including the cut loss.
        let half_blade_width = self.blade_width as f64 / 2.0;
        let used_area = self.cut_pieces.iter().fold(0.0, |acc, p| {
            let rect = &p.rect;
            let width: f64 = rect.width as f64
                + f64::min(rect.x as f64, half_blade_width)
                + f64::min(
                    self.width as f64 - rect.width as f64 - rect.x as f64,
                    half_blade_width,
                );

            let length: f64 = rect.length as f64
                + f64::min(rect.y as f64, half_blade_width)
                + f64::min(
                    self.length as f64 - rect.length as f64 - rect.y as f64,
                    half_blade_width,
                );

            acc + width * length
        });

        (used_area / (self.width as f64 * self.length as f64))
            .powf(2.0 + self.free_rects.len() as f64 * 0.01)
    }

    fn price(&self) -> usize {
        self.price
    }

    fn remove_cut_pieces<I>(&mut self, cut_pieces: I) -> usize
    where
        I: Iterator,
        I::Item: Borrow<UsedCutPiece>,
    {
        let old_len = self.cut_pieces.len();
        for cut_piece_to_remove in cut_pieces {
            for i in (0..self.cut_pieces.len()).rev() {
                if &self.cut_pieces[i] == cut_piece_to_remove.borrow() {
                    let removed_piece = self.cut_pieces.remove(i);
                    self.free_rects.push(removed_piece.rect);
                }
            }
        }
        old_len - self.cut_pieces.len()
    }

    fn cut_pieces(&self) -> std::slice::Iter<'_, UsedCutPiece> {
        self.cut_pieces.iter()
    }

    fn possible_heuristics() -> Vec<Self::Heuristic> {
        vec![
            (
                FreeRectChoiceHeuristic::BestShortSideFit,
                RotateCutPieceHeuristic::PreferUpright,
            ),
            (
                FreeRectChoiceHeuristic::BestLongSideFit,
                RotateCutPieceHeuristic::PreferUpright,
            ),
            (
                FreeRectChoiceHeuristic::BestAreaFit,
                RotateCutPieceHeuristic::PreferUpright,
            ),
            (
                FreeRectChoiceHeuristic::BottomLeftRule,
                RotateCutPieceHeuristic::PreferUpright,
            ),
            (
                FreeRectChoiceHeuristic::ContactPointRule,
                RotateCutPieceHeuristic::PreferUpright,
            ),
            (
                FreeRectChoiceHeuristic::BestShortSideFit,
                RotateCutPieceHeuristic::PreferRotated,
            ),
            (
                FreeRectChoiceHeuristic::BestLongSideFit,
                RotateCutPieceHeuristic::PreferRotated,
            ),
            (
                FreeRectChoiceHeuristic::BestAreaFit,
                RotateCutPieceHeuristic::PreferRotated,
            ),
            (
                FreeRectChoiceHeuristic::BottomLeftRule,
                RotateCutPieceHeuristic::PreferRotated,
            ),
            (
                FreeRectChoiceHeuristic::ContactPointRule,
                RotateCutPieceHeuristic::PreferRotated,
            ),
        ]
    }

    fn insert_cut_piece_with_heuristic(
        &mut self,
        cut_piece: &CutPieceWithId,
        heuristic: &Self::Heuristic,
    ) -> bool {
        self.insert_with_heuristics(cut_piece, heuristic.0, heuristic.1)
    }

    fn insert_cut_piece_random_heuristic<R>(
        &mut self,
        cut_piece: &CutPieceWithId,
        rng: &mut R,
    ) -> bool
    where
        R: Rng + ?Sized,
    {
        self.insert_cut_piece_with_heuristic(cut_piece, &rng.gen())
    }

    fn matches_stock_piece(&self, stock_piece: &StockPiece) -> bool {
        self.width == stock_piece.width
            && self.length == stock_piece.length
            && self.pattern_direction == stock_piece.pattern_direction
            && self.price == stock_piece.price
    }
}

impl MaxRectsBin {
    /// Insert demand piece in bin if it fits.
    fn insert_with_heuristics(
        &mut self,
        cut_piece: &CutPieceWithId,
        rect_choice: FreeRectChoiceHeuristic,
        rotate_preference: RotateCutPieceHeuristic,
    ) -> bool {
        let prefer_rotated = rotate_preference == RotateCutPieceHeuristic::PreferRotated;

        if let Some((best_rect, is_rotated)) =
            self.find_placement_for_cut_piece(cut_piece, rect_choice, prefer_rotated)
        {
            for i in (0..self.free_rects.len()).rev() {
                self.split_free_rect(i, &best_rect);
            }

            self.prune_free_rects();

            let pattern_direction = if is_rotated {
                cut_piece.pattern_direction.rotated()
            } else {
                cut_piece.pattern_direction
            };
            self.cut_pieces.push(UsedCutPiece {
                id: cut_piece.id,
                external_id: cut_piece.external_id,
                rect: best_rect,
                can_rotate: cut_piece.can_rotate,
                pattern_direction,
                is_rotated,
            });

            true
        } else {
            false
        }
    }

    fn find_placement_for_cut_piece(
        &self,
        cut_piece: &CutPieceWithId,
        rect_choice: FreeRectChoiceHeuristic,
        prefer_rotated: bool,
    ) -> Option<(Rect, bool)> {
        match rect_choice {
            FreeRectChoiceHeuristic::BottomLeftRule => {
                self.find_placement_bottom_left(cut_piece, prefer_rotated)
            }
            FreeRectChoiceHeuristic::BestShortSideFit => {
                self.find_placement_best_short_side_fit(cut_piece, prefer_rotated)
            }
            FreeRectChoiceHeuristic::BestLongSideFit => {
                self.find_placement_best_long_side_fit(cut_piece, prefer_rotated)
            }
            FreeRectChoiceHeuristic::BestAreaFit => {
                self.find_placement_best_area_fit(cut_piece, prefer_rotated)
            }
            FreeRectChoiceHeuristic::ContactPointRule => {
                self.find_placement_contact_point(cut_piece, prefer_rotated)
            }
        }
    }

    fn find_placement_bottom_left(
        &self,
        cut_piece: &CutPieceWithId,
        prefer_rotated: bool,
    ) -> Option<(Rect, bool)> {
        let mut best_rect = Rect::default();
        let mut best_y = std::usize::MAX;
        let mut best_x = std::usize::MAX;
        let mut best_fit = Fit::None;

        for free_rect in &self.free_rects {
            let fit = free_rect.fit_cut_piece(self.pattern_direction, cut_piece, prefer_rotated);
            if fit.is_upright() {
                let top_side_y = free_rect.y + cut_piece.length;
                if top_side_y < best_y || (top_side_y == best_y && free_rect.x < best_x) {
                    best_rect.x = free_rect.x;
                    best_rect.y = free_rect.y;
                    best_rect.width = cut_piece.width;
                    best_rect.length = cut_piece.length;
                    best_y = top_side_y;
                    best_x = free_rect.x;
                    best_fit = fit;
                }
            } else if fit.is_rotated() {
                let top_side_y = free_rect.y + cut_piece.width;
                if top_side_y < best_y || (top_side_y == best_y && free_rect.x < best_x) {
                    best_rect.x = free_rect.x;
                    best_rect.y = free_rect.y;
                    best_rect.width = cut_piece.length;
                    best_rect.length = cut_piece.width;
                    best_y = top_side_y;
                    best_x = free_rect.x;
                    best_fit = fit;
                }
            }
        }

        if best_fit.is_none() {
            None
        } else {
            Some((best_rect, best_fit.is_rotated()))
        }
    }

    fn find_placement_best_short_side_fit(
        &self,
        cut_piece: &CutPieceWithId,
        prefer_rotated: bool,
    ) -> Option<(Rect, bool)> {
        let mut best_rect = Rect::default();
        let mut best_short_side_fit = std::usize::MAX;
        let mut best_long_side_fit = std::usize::MAX;
        let mut best_fit = Fit::None;

        for free_rect in &self.free_rects {
            let fit = free_rect.fit_cut_piece(self.pattern_direction, cut_piece, prefer_rotated);
            if fit.is_upright() {
                let leftover_horiz =
                    (free_rect.width as isize - cut_piece.width as isize).abs() as usize;
                let leftover_vert =
                    (free_rect.length as isize - cut_piece.length as isize).abs() as usize;
                let short_side_fit = cmp::min(leftover_horiz, leftover_vert);
                let long_side_fit = cmp::max(leftover_horiz, leftover_vert);

                if short_side_fit < best_short_side_fit
                    || (short_side_fit == best_short_side_fit && long_side_fit < best_long_side_fit)
                {
                    best_rect.x = free_rect.x;
                    best_rect.y = free_rect.y;
                    best_rect.width = cut_piece.width;
                    best_rect.length = cut_piece.length;
                    best_short_side_fit = short_side_fit;
                    best_long_side_fit = long_side_fit;
                    best_fit = fit;
                }
            } else if fit.is_rotated() {
                let leftover_horiz =
                    (free_rect.width as isize - cut_piece.length as isize).abs() as usize;
                let leftover_vert =
                    (free_rect.length as isize - cut_piece.width as isize).abs() as usize;
                let short_side_fit = cmp::min(leftover_horiz, leftover_vert);
                let long_side_fit = cmp::max(leftover_horiz, leftover_vert);

                if short_side_fit < best_short_side_fit
                    || (short_side_fit == best_short_side_fit && long_side_fit < best_long_side_fit)
                {
                    best_rect.x = free_rect.x;
                    best_rect.y = free_rect.y;
                    best_rect.width = cut_piece.length;
                    best_rect.length = cut_piece.width;
                    best_short_side_fit = short_side_fit;
                    best_long_side_fit = long_side_fit;
                    best_fit = fit;
                }
            }
        }

        if best_fit.is_none() {
            None
        } else {
            Some((best_rect, best_fit.is_rotated()))
        }
    }

    fn find_placement_best_long_side_fit(
        &self,
        cut_piece: &CutPieceWithId,
        prefer_rotated: bool,
    ) -> Option<(Rect, bool)> {
        let mut best_rect = Rect::default();
        let mut best_short_side_fit = std::usize::MAX;
        let mut best_long_side_fit = std::usize::MAX;
        let mut best_fit = Fit::None;

        for free_rect in &self.free_rects {
            let fit = free_rect.fit_cut_piece(self.pattern_direction, cut_piece, prefer_rotated);
            if fit.is_upright() {
                let leftover_horiz =
                    (free_rect.width as isize - cut_piece.width as isize).abs() as usize;
                let leftover_vert =
                    (free_rect.length as isize - cut_piece.length as isize).abs() as usize;
                let short_side_fit = cmp::min(leftover_horiz, leftover_vert);
                let long_side_fit = cmp::max(leftover_horiz, leftover_vert);

                if long_side_fit < best_long_side_fit
                    || (long_side_fit == best_long_side_fit && short_side_fit < best_short_side_fit)
                {
                    best_rect.x = free_rect.x;
                    best_rect.y = free_rect.y;
                    best_rect.width = cut_piece.width;
                    best_rect.length = cut_piece.length;
                    best_short_side_fit = short_side_fit;
                    best_long_side_fit = long_side_fit;
                    best_fit = fit;
                }
            } else if fit.is_rotated() {
                let leftover_horiz =
                    (free_rect.width as isize - cut_piece.length as isize).abs() as usize;
                let leftover_vert =
                    (free_rect.length as isize - cut_piece.width as isize).abs() as usize;
                let short_side_fit = cmp::min(leftover_horiz, leftover_vert);
                let long_side_fit = cmp::max(leftover_horiz, leftover_vert);

                if long_side_fit < best_long_side_fit
                    || (long_side_fit == best_long_side_fit && short_side_fit < best_short_side_fit)
                {
                    best_rect.x = free_rect.x;
                    best_rect.y = free_rect.y;
                    best_rect.width = cut_piece.length;
                    best_rect.length = cut_piece.width;
                    best_short_side_fit = short_side_fit;
                    best_long_side_fit = long_side_fit;
                    best_fit = fit;
                }
            }
        }

        if best_fit.is_none() {
            None
        } else {
            Some((best_rect, best_fit.is_rotated()))
        }
    }

    fn find_placement_best_area_fit(
        &self,
        cut_piece: &CutPieceWithId,
        prefer_rotated: bool,
    ) -> Option<(Rect, bool)> {
        let mut best_rect = Rect::default();
        let mut best_area_fit = std::u64::MAX;
        let mut best_short_side_fit = std::u64::MAX;
        let mut best_fit = Fit::None;

        for free_rect in &self.free_rects {
            let free_rect_area = free_rect.width as u64 * free_rect.length as u64;
            let cut_piece_area = cut_piece.width as u64 * cut_piece.length as u64;

            if cut_piece_area > free_rect_area {
                continue;
            }

            let area_fit = free_rect_area - cut_piece_area;

            let fit = free_rect.fit_cut_piece(self.pattern_direction, cut_piece, prefer_rotated);
            if fit.is_upright() {
                let leftover_horiz = (free_rect.width as i64 - cut_piece.width as i64).abs() as u64;
                let leftover_vert =
                    (free_rect.length as i64 - cut_piece.length as i64).abs() as u64;
                let short_side_fit = cmp::min(leftover_horiz, leftover_vert);

                if area_fit < best_area_fit
                    || (area_fit == best_area_fit && short_side_fit < best_short_side_fit)
                {
                    best_rect.x = free_rect.x;
                    best_rect.y = free_rect.y;
                    best_rect.width = cut_piece.width;
                    best_rect.length = cut_piece.length;
                    best_area_fit = area_fit;
                    best_short_side_fit = short_side_fit;
                    best_fit = fit;
                }
            } else if fit.is_rotated() {
                let leftover_horiz =
                    (free_rect.width as i64 - cut_piece.length as i64).abs() as u64;
                let leftover_vert = (free_rect.length as i64 - cut_piece.width as i64).abs() as u64;
                let short_side_fit = cmp::min(leftover_horiz, leftover_vert);

                if area_fit < best_area_fit
                    || (area_fit == best_area_fit && short_side_fit < best_short_side_fit)
                {
                    best_rect.x = free_rect.x;
                    best_rect.y = free_rect.y;
                    best_rect.width = cut_piece.length;
                    best_rect.length = cut_piece.width;
                    best_area_fit = area_fit;
                    best_short_side_fit = short_side_fit;
                    best_fit = fit;
                }
            }
        }

        if best_fit.is_none() {
            None
        } else {
            Some((best_rect, best_fit.is_rotated()))
        }
    }

    fn find_placement_contact_point(
        &self,
        cut_piece: &CutPieceWithId,
        prefer_rotated: bool,
    ) -> Option<(Rect, bool)> {
        let mut best_rect = Rect::default();
        let mut best_contact_score = 0;
        let mut best_fit = Fit::None;

        for free_rect in &self.free_rects {
            let fit = free_rect.fit_cut_piece(self.pattern_direction, cut_piece, prefer_rotated);
            if fit.is_upright() {
                let score = self.contact_point_score(
                    free_rect.x,
                    free_rect.y,
                    cut_piece.width,
                    cut_piece.length,
                );
                if score > best_contact_score || best_fit.is_none() {
                    best_rect.x = free_rect.x;
                    best_rect.y = free_rect.y;
                    best_rect.width = cut_piece.width;
                    best_rect.length = cut_piece.length;
                    best_contact_score = score;
                    best_fit = fit;
                }
            } else if fit.is_rotated() {
                let score = self.contact_point_score(
                    free_rect.x,
                    free_rect.y,
                    cut_piece.length,
                    cut_piece.width,
                );
                if score > best_contact_score || best_fit.is_none() {
                    best_rect.x = free_rect.x;
                    best_rect.y = free_rect.y;
                    best_rect.width = cut_piece.length;
                    best_rect.length = cut_piece.width;
                    best_contact_score = score;
                    best_fit = fit;
                }
            }
        }

        if best_fit.is_none() {
            None
        } else {
            Some((best_rect, best_fit.is_rotated()))
        }
    }

    fn contact_point_score(&self, x: usize, y: usize, width: usize, length: usize) -> usize {
        let mut score = 0;

        if x == 0 || x + width == self.width {
            score += length;
        }

        if y == 0 || y + length == self.length {
            score += width;
        }

        for cut_piece in &self.cut_pieces {
            let rect = &cut_piece.rect;
            if rect.x == x + width || rect.x + rect.width == x {
                score += common_interval_length(rect.y, rect.y + rect.length, y, y + length);
            }

            if rect.y == y + length || rect.y + rect.length == y {
                score += common_interval_length(rect.x, rect.x + rect.width, x, x + width);
            }
        }

        score
    }

    fn split_free_rect(&mut self, free_rect_index: usize, rect: &Rect) {
        let free_rect = self.free_rects[free_rect_index];

        // Account for blade width.
        let rect = {
            let x = if rect.x >= self.blade_width {
                rect.x - self.blade_width
            } else {
                0
            };
            let y = if rect.y >= self.blade_width {
                rect.y - self.blade_width
            } else {
                0
            };
            let mut width = rect.width + rect.x - x + self.blade_width;
            if x + width > self.width {
                width -= x + width - self.width;
            }
            let mut length = rect.length + rect.y - y + self.blade_width;
            if y + length > self.length {
                length -= y + length - self.length;
            }

            Rect {
                x,
                y,
                width,
                length,
            }
        };

        // Check if rects intersect
        if rect.x >= free_rect.x + free_rect.width
            || rect.x + rect.width <= free_rect.x
            || rect.y >= free_rect.y + free_rect.length
            || rect.y + rect.length <= free_rect.y
        {
            return;
        }

        if rect.x < free_rect.x + free_rect.width && rect.x + rect.width > free_rect.x {
            // New rect above
            if rect.y > free_rect.y && rect.y < free_rect.y + free_rect.length {
                let mut new_rect = free_rect;
                new_rect.length = rect.y - new_rect.y;
                self.free_rects.push(new_rect);
            }

            // New rect below
            if rect.y + rect.length < free_rect.y + free_rect.length {
                let mut new_rect = free_rect;
                new_rect.y = rect.y + rect.length;
                new_rect.length = free_rect.y + free_rect.length - rect.y - rect.length;
                self.free_rects.push(new_rect);
            }
        }

        if rect.y < free_rect.y + free_rect.length && rect.y + rect.length > free_rect.y {
            // New rect to the left
            if rect.x > free_rect.x && rect.x < free_rect.x + free_rect.width {
                let mut new_rect = free_rect;
                new_rect.width = rect.x - new_rect.x;
                self.free_rects.push(new_rect);
            }

            // New rect to the right
            if rect.x + rect.width < free_rect.x + free_rect.width {
                let mut new_rect = free_rect;
                new_rect.x = rect.x + rect.width;
                new_rect.width = free_rect.x + free_rect.width - rect.x - rect.width;
                self.free_rects.push(new_rect);
            }
        }

        // Remove original free rect that was split.
        self.free_rects.swap_remove(free_rect_index);
    }

    // Remove free rects that are contained by other free rects.
    fn prune_free_rects(&mut self) {
        for i in (0..self.free_rects.len()).rev() {
            for j in (i + 1..self.free_rects.len()).rev() {
                if self.free_rects[j].contains(&self.free_rects[i]) {
                    self.free_rects.swap_remove(i);
                } else if self.free_rects[i].contains(&self.free_rects[j]) {
                    self.free_rects.swap_remove(j);
                }
            }
        }
    }

    fn make_free_rects_disjoint(&mut self) {
        let length = self.free_rects.len();
        'outer: for i in (0..length).rev() {
            for j in (i + 1..length).rev() {
                // It's possible that self.free_rects gets smaller
                // so we must check we haven't iterated too far.
                if j >= self.free_rects.len() {
                    break;
                }
                if i >= self.free_rects.len() {
                    break 'outer;
                }

                if self.free_rects[i].width as u64 * self.free_rects[i].length as u64
                    > self.free_rects[j].width as u64 * self.free_rects[j].length as u64
                {
                    let rect = self.free_rects[i];
                    self.split_free_rect(j, &rect);
                } else {
                    let rect = self.free_rects[j];
                    self.split_free_rect(i, &rect);
                }
            }
        }
    }
}

impl From<MaxRectsBin> for ResultStockPiece {
    fn from(mut bin: MaxRectsBin) -> Self {
        bin.make_free_rects_disjoint();
        Self {
            width: bin.width,
            length: bin.length,
            pattern_direction: bin.pattern_direction,
            cut_pieces: bin.cut_pieces.iter().map(Into::into).collect(),
            waste_pieces: bin.free_rects,
            price: bin.price,
        }
    }
}

/// Returns 0 if the two intervals i1 and i2 are disjoint, or the length of their overlap otherwise.
fn common_interval_length(start1: usize, end1: usize, start2: usize, end2: usize) -> usize {
    if end1 < start2 || end2 < start1 {
        0
    } else {
        cmp::min(end1, end2) - cmp::max(start1, start2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_cut_pieces() {
        let cut_pieces = &[
            CutPieceWithId {
                id: 0,
                external_id: None,
                width: 10,
                length: 10,
                pattern_direction: PatternDirection::None,
                can_rotate: false,
            },
            CutPieceWithId {
                id: 1,
                external_id: None,
                width: 10,
                length: 10,
                pattern_direction: PatternDirection::None,
                can_rotate: false,
            },
            CutPieceWithId {
                id: 2,
                external_id: None,
                width: 10,
                length: 10,
                pattern_direction: PatternDirection::None,
                can_rotate: false,
            },
            CutPieceWithId {
                id: 3,
                external_id: None,
                width: 10,
                length: 10,
                pattern_direction: PatternDirection::None,
                can_rotate: false,
            },
        ];

        let heuristic = MaxRectsBin::possible_heuristics()[0];

        let mut bin = MaxRectsBin::new(48, 96, 1, PatternDirection::None, 0);
        cut_pieces.iter().for_each(|cut_piece| {
            bin.insert_cut_piece_with_heuristic(cut_piece, &heuristic);
        });

        assert_eq!(bin.cut_pieces().len(), 4);

        let cut_pieces_to_remove = [
            UsedCutPiece {
                id: 1,
                external_id: None,
                rect: Default::default(),
                pattern_direction: PatternDirection::None,
                is_rotated: false,
                can_rotate: false,
            },
            UsedCutPiece {
                id: 3,
                external_id: None,
                rect: Default::default(),
                pattern_direction: PatternDirection::None,
                is_rotated: false,
                can_rotate: false,
            },
        ];

        bin.remove_cut_pieces(cut_pieces_to_remove.iter());

        assert_eq!(bin.cut_pieces().len(), 2);
        assert_eq!(bin.cut_pieces().next().unwrap().id, 0);
        assert_eq!(bin.cut_pieces().nth(1).unwrap().id, 2);
    }

    #[test]
    fn bin_matches_stock_piece() {
        let bin = MaxRectsBin {
            width: 48,
            length: 96,
            blade_width: 1,
            pattern_direction: PatternDirection::None,
            cut_pieces: Default::default(),
            free_rects: Default::default(),
            price: 0,
        };

        let stock_piece = StockPiece {
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: Some(20),
        };

        assert!(bin.matches_stock_piece(&stock_piece));
    }

    #[test]
    fn bin_does_not_match_stock_pieces() {
        let bin = MaxRectsBin {
            width: 48,
            length: 96,
            blade_width: 1,
            pattern_direction: PatternDirection::None,
            cut_pieces: Default::default(),
            free_rects: Default::default(),
            price: 0,
        };

        let stock_pieces = &[
            StockPiece {
                width: 10,
                length: 96,
                pattern_direction: PatternDirection::None,
                price: 0,
                quantity: Some(20),
            },
            StockPiece {
                width: 48,
                length: 10,
                pattern_direction: PatternDirection::None,
                price: 0,
                quantity: Some(20),
            },
            StockPiece {
                width: 48,
                length: 96,
                pattern_direction: PatternDirection::ParallelToLength,
                price: 0,
                quantity: Some(20),
            },
            StockPiece {
                width: 48,
                length: 96,
                pattern_direction: PatternDirection::None,
                price: 10,
                quantity: Some(20),
            },
        ];

        stock_pieces
            .iter()
            .for_each(|stock_piece| assert!(!bin.matches_stock_piece(&stock_piece)))
    }
}
