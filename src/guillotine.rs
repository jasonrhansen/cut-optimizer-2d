/// Implementation of the Guillotine Algorithms for bin packing.
/// [A Thousand Ways to Pack the Bin](http://pds25.egloos.com/pds/201504/21/98/RectangleBinPack.pdf)
use super::*;

use rand::distributions::{Distribution, Standard};
use rand::prelude::*;

use std::borrow::Borrow;
use std::cmp;

/// Heuristics for deciding which of the free rectangles to place the demand piece in.
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) enum FreeRectChoiceHeuristic {
    BestAreaFit,
    BestShortSideFit,
    BestLongSideFit,
    WorstAreaFit,
    WorstShortSideFit,
    WorstLongSideFit,
    SmallestY,
}

impl Distribution<FreeRectChoiceHeuristic> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> FreeRectChoiceHeuristic {
        // Don't include the "worst fit" heuristics here since they tend to give worse results.
        match rng.gen_range(0..3) {
            0 => FreeRectChoiceHeuristic::BestAreaFit,
            1 => FreeRectChoiceHeuristic::BestShortSideFit,
            _ => FreeRectChoiceHeuristic::BestLongSideFit,
        }
    }
}

/// Heuristic for determining how to subdivide the free space that remains after placing a demand piece.
#[derive(Copy, Clone)]
pub(crate) enum SplitHeuristic {
    ShorterLeftoverAxis,
    LongerLeftoverAxis,
    MinimizeArea,
    MaximizeArea,
    ShorterAxis,
    LongerAxis,
}

impl Distribution<SplitHeuristic> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> SplitHeuristic {
        match rng.gen_range(0..6) {
            0 => SplitHeuristic::ShorterLeftoverAxis,
            1 => SplitHeuristic::LongerLeftoverAxis,
            2 => SplitHeuristic::MinimizeArea,
            3 => SplitHeuristic::MaximizeArea,
            4 => SplitHeuristic::ShorterAxis,
            _ => SplitHeuristic::LongerAxis,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct GuillotineBin {
    width: usize,
    length: usize,
    blade_width: usize,
    pattern_direction: PatternDirection,
    cut_pieces: Vec<UsedCutPiece>,
    free_rects: Vec<Rect>,
}

impl Bin for GuillotineBin {
    type Heuristic = (FreeRectChoiceHeuristic, SplitHeuristic);

    fn new(
        width: usize,
        length: usize,
        blade_width: usize,
        pattern_direction: PatternDirection,
    ) -> Self {
        // We start with a single big free rectangle that spans the whole bin.
        let free_rect = Rect {
            x: 0,
            y: 0,
            width,
            length,
        };

        let free_rects = vec![free_rect];

        GuillotineBin {
            width,
            length,
            free_rects,
            blade_width,
            pattern_direction,
            cut_pieces: Default::default(),
        }
    }

    fn fitness(&self) -> f64 {
        let used_area = self
            .cut_pieces
            .iter()
            .fold(0, |acc, p| acc + p.rect.width as u64 * p.rect.length as u64)
            as f64;

        let free_area =
            self.free_rects
                .iter()
                .fold(0, |acc, fr| acc + fr.width as u64 * fr.length as u64) as f64;

        (used_area / (used_area + free_area) as f64).powf(2.0 + self.free_rects.len() as f64 * 0.01)
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
        self.merge_free_rects();
        old_len - self.cut_pieces.len()
    }

    fn cut_pieces(&self) -> std::slice::Iter<'_, UsedCutPiece> {
        self.cut_pieces.iter()
    }

    fn possible_heuristics() -> Vec<Self::Heuristic> {
        vec![
            (
                FreeRectChoiceHeuristic::BestAreaFit,
                SplitHeuristic::ShorterLeftoverAxis,
            ),
            (
                FreeRectChoiceHeuristic::BestAreaFit,
                SplitHeuristic::LongerLeftoverAxis,
            ),
            (
                FreeRectChoiceHeuristic::BestAreaFit,
                SplitHeuristic::MinimizeArea,
            ),
            (
                FreeRectChoiceHeuristic::BestAreaFit,
                SplitHeuristic::MaximizeArea,
            ),
            (
                FreeRectChoiceHeuristic::BestAreaFit,
                SplitHeuristic::ShorterAxis,
            ),
            (
                FreeRectChoiceHeuristic::BestAreaFit,
                SplitHeuristic::LongerAxis,
            ),
            (
                FreeRectChoiceHeuristic::BestShortSideFit,
                SplitHeuristic::ShorterLeftoverAxis,
            ),
            (
                FreeRectChoiceHeuristic::BestShortSideFit,
                SplitHeuristic::LongerLeftoverAxis,
            ),
            (
                FreeRectChoiceHeuristic::BestShortSideFit,
                SplitHeuristic::MinimizeArea,
            ),
            (
                FreeRectChoiceHeuristic::BestShortSideFit,
                SplitHeuristic::MaximizeArea,
            ),
            (
                FreeRectChoiceHeuristic::BestShortSideFit,
                SplitHeuristic::ShorterAxis,
            ),
            (
                FreeRectChoiceHeuristic::BestShortSideFit,
                SplitHeuristic::LongerAxis,
            ),
            (
                FreeRectChoiceHeuristic::BestLongSideFit,
                SplitHeuristic::ShorterLeftoverAxis,
            ),
            (
                FreeRectChoiceHeuristic::BestLongSideFit,
                SplitHeuristic::LongerLeftoverAxis,
            ),
            (
                FreeRectChoiceHeuristic::BestLongSideFit,
                SplitHeuristic::MinimizeArea,
            ),
            (
                FreeRectChoiceHeuristic::BestLongSideFit,
                SplitHeuristic::MaximizeArea,
            ),
            (
                FreeRectChoiceHeuristic::BestLongSideFit,
                SplitHeuristic::ShorterAxis,
            ),
            (
                FreeRectChoiceHeuristic::BestLongSideFit,
                SplitHeuristic::LongerAxis,
            ),
        ]
    }

    fn insert_cut_piece_with_heuristic(
        &mut self,
        cut_piece: &CutPieceWithId,
        heuristic: &Self::Heuristic,
    ) -> bool {
        self.insert_with_heuristics(cut_piece, true, heuristic.0, heuristic.1)
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
}

impl GuillotineBin {
    /// Insert demand piece in bin if it fits.
    fn insert_with_heuristics(
        &mut self,
        cut_piece: &CutPieceWithId,
        merge: bool,
        rect_choice: FreeRectChoiceHeuristic,
        split_method: SplitHeuristic,
    ) -> bool {
        if let Some((used_piece, free_index)) =
            self.find_placement_for_cut_piece(cut_piece, rect_choice)
        {
            let free_rect = self.free_rects.swap_remove(free_index);
            self.split_free_rect_by_heuristic(&free_rect, &used_piece.rect, split_method);

            if merge {
                self.merge_free_rects();
            }

            self.cut_pieces.push(used_piece);

            true
        } else {
            false
        }
    }

    fn find_placement_for_cut_piece(
        &self,
        cut_piece: &CutPieceWithId,
        rect_choice: FreeRectChoiceHeuristic,
    ) -> Option<(UsedCutPiece, usize)> {
        let mut best_rect = Rect::default();
        let mut best_score = std::isize::MAX;
        let mut best_fit = Fit::None;
        let mut free_index = None;

        for (i, free_rect) in self.free_rects.iter().enumerate() {
            let fit = free_rect.fit_cut_piece(self.pattern_direction, cut_piece);
            match fit {
                Fit::UprightExact => {
                    best_rect.x = free_rect.x;
                    best_rect.y = free_rect.y;
                    best_rect.width = cut_piece.width;
                    best_rect.length = cut_piece.length;
                    best_fit = fit;
                    free_index = Some(i);
                    break;
                }
                Fit::RotatedExact => {
                    best_rect.x = free_rect.x;
                    best_rect.y = free_rect.y;
                    best_rect.width = cut_piece.length;
                    best_rect.length = cut_piece.width;
                    best_fit = fit;
                    free_index = Some(i);
                    break;
                }
                Fit::Upright => {
                    let score = score_by_heuristic(
                        cut_piece.width,
                        cut_piece.length,
                        free_rect,
                        rect_choice,
                    );
                    if score < best_score {
                        best_rect.x = free_rect.x;
                        best_rect.y = free_rect.y;
                        best_rect.width = cut_piece.width;
                        best_rect.length = cut_piece.length;
                        best_score = score;
                        best_fit = fit;
                        free_index = Some(i);
                    }
                }
                Fit::Rotated => {
                    let score = score_by_heuristic(
                        cut_piece.length,
                        cut_piece.width,
                        free_rect,
                        rect_choice,
                    );
                    if score < best_score {
                        best_rect.x = free_rect.x;
                        best_rect.y = free_rect.y;
                        best_rect.width = cut_piece.length;
                        best_rect.length = cut_piece.width;
                        best_score = score;
                        best_fit = fit;
                        free_index = Some(i);
                    }
                }
                Fit::None => (),
            }
        }

        if let Some(index) = free_index {
            let is_rotated = best_fit == Fit::Rotated || best_fit == Fit::RotatedExact;
            let pattern_direction = if is_rotated {
                cut_piece.pattern_direction.rotated()
            } else {
                cut_piece.pattern_direction
            };
            Some((
                UsedCutPiece {
                    id: cut_piece.id,
                    external_id: cut_piece.external_id,
                    rect: best_rect,
                    can_rotate: cut_piece.can_rotate,
                    pattern_direction,
                    is_rotated,
                },
                index,
            ))
        } else {
            None
        }
    }

    fn split_free_rect_by_heuristic(
        &mut self,
        free_rect: &Rect,
        rect: &Rect,
        method: SplitHeuristic,
    ) {
        // Compute leftover dimensions.
        let w = (free_rect.width - rect.width) as u64;
        let h = (free_rect.length - rect.length) as u64;

        // Placing `cut_piece` into `free_rect` results in an L-shaped free area, which must be split into
        // two disjoint rectangles. This can be achieved with by splitting the L-shape using a single line.
        // We have two choices: horizontal or vertical.

        // Use the given heuristic to decide which choice to make.
        let split_horizontal = match method {
            SplitHeuristic::ShorterLeftoverAxis => (w <= h),
            SplitHeuristic::LongerLeftoverAxis => (w > h),
            SplitHeuristic::MinimizeArea => (rect.width as u64 * h > w * rect.length as u64),
            SplitHeuristic::MaximizeArea => (rect.width as u64 * h <= w * rect.length as u64),
            SplitHeuristic::ShorterAxis => (free_rect.width as u64 <= free_rect.length as u64),
            SplitHeuristic::LongerAxis => (free_rect.width as u64 > free_rect.length as u64),
        };

        let split_axis = if split_horizontal {
            SplitAxis::Horizontal
        } else {
            SplitAxis::Vertical
        };
        self.split_free_rect_along_axis(free_rect, rect, split_axis);
    }

    fn split_free_rect_along_axis(&mut self, free_rect: &Rect, rect: &Rect, split_axis: SplitAxis) {
        let (bottom_width, right_length) = match split_axis {
            SplitAxis::Horizontal => (free_rect.width, rect.length),
            SplitAxis::Vertical => (rect.width, free_rect.length),
        };

        let bottom_length = match free_rect.length - rect.length {
            h if h > self.blade_width => h - self.blade_width,
            _ => 0,
        };

        let right_width = match free_rect.width - rect.width {
            w if w > self.blade_width => w - self.blade_width,
            _ => 0,
        };

        // Add the new rectangles into the free rectangle pool if they weren't degenerate.
        if bottom_width > 0 && bottom_length > 0 {
            let bottom = Rect {
                x: free_rect.x,
                y: free_rect.y + rect.length + self.blade_width,
                width: bottom_width,
                length: bottom_length,
            };
            self.free_rects.push(bottom);
        }
        if right_width > 0 && right_length > 0 {
            let right = Rect {
                x: free_rect.x + rect.width + self.blade_width,
                y: free_rect.y,
                width: right_width,
                length: right_length,
            };
            self.free_rects.push(right);
        }
    }

    /// Merge adjacent free rectangles
    fn merge_free_rects(&mut self) {
        for i in (0..self.free_rects.len()).rev() {
            for j in (i + 1..self.free_rects.len()).rev() {
                if self.free_rects[i].width == self.free_rects[j].width
                    && self.free_rects[i].x == self.free_rects[j].x
                {
                    if self.free_rects[i].y
                        == self.free_rects[j].y + self.free_rects[j].length + self.blade_width
                    {
                        self.free_rects[i].y -= self.free_rects[j].length + self.blade_width;
                        self.free_rects[i].length += self.free_rects[j].length + self.blade_width;
                        self.free_rects.swap_remove(j);
                    } else if self.free_rects[i].y + self.free_rects[i].length + self.blade_width
                        == self.free_rects[j].y
                    {
                        self.free_rects[i].length += self.free_rects[j].length + self.blade_width;
                        self.free_rects.swap_remove(j);
                    }
                } else if self.free_rects[i].length == self.free_rects[j].length
                    && self.free_rects[i].y == self.free_rects[j].y
                {
                    if self.free_rects[i].x
                        == self.free_rects[j].x + self.free_rects[j].width + self.blade_width
                    {
                        self.free_rects[i].x -= self.free_rects[j].width + self.blade_width;
                        self.free_rects[i].width += self.free_rects[j].width + self.blade_width;
                        self.free_rects.swap_remove(j);
                    } else if self.free_rects[i].x + self.free_rects[i].width + self.blade_width
                        == self.free_rects[j].x
                    {
                        self.free_rects[i].width += self.free_rects[j].width + self.blade_width;
                        self.free_rects.swap_remove(j);
                    }
                }
            }
        }
    }
}

impl Into<ResultStockPiece> for GuillotineBin {
    fn into(self) -> ResultStockPiece {
        ResultStockPiece {
            width: self.width,
            length: self.length,
            pattern_direction: self.pattern_direction,
            cut_pieces: self.cut_pieces.into_iter().map(Into::into).collect(),
            waste_pieces: self.free_rects,
        }
    }
}

#[derive(Copy, Clone)]
enum SplitAxis {
    Horizontal,
    Vertical,
}

fn score_by_heuristic(
    width: usize,
    length: usize,
    free_rect: &Rect,
    rect_choice: FreeRectChoiceHeuristic,
) -> isize {
    match rect_choice {
        FreeRectChoiceHeuristic::BestAreaFit => score_best_area_fit(width, length, free_rect),
        FreeRectChoiceHeuristic::BestShortSideFit => {
            score_best_short_side_fit(width, length, free_rect)
        }
        FreeRectChoiceHeuristic::BestLongSideFit => {
            score_best_long_side_fit(width, length, free_rect)
        }
        FreeRectChoiceHeuristic::WorstAreaFit => score_worst_area_fit(width, length, free_rect),
        FreeRectChoiceHeuristic::WorstShortSideFit => {
            score_worst_short_side_fit(width, length, free_rect)
        }
        FreeRectChoiceHeuristic::WorstLongSideFit => {
            score_worst_long_side_fit(width, length, free_rect)
        }
        FreeRectChoiceHeuristic::SmallestY => free_rect.y as isize,
    }
}

fn score_best_area_fit(width: usize, length: usize, free_rect: &Rect) -> isize {
    ((free_rect.width as i64 * free_rect.length as i64) - (width as i64 * length as i64)) as isize
}

fn score_best_short_side_fit(width: usize, length: usize, free_rect: &Rect) -> isize {
    let leftover_horiz = (free_rect.width as i64 - width as i64).abs();
    let leftover_vert = (free_rect.length as i64 - length as i64).abs();
    cmp::min(leftover_horiz, leftover_vert) as isize
}

fn score_best_long_side_fit(width: usize, length: usize, free_rect: &Rect) -> isize {
    let leftover_horiz = (free_rect.width as i64 - width as i64).abs();
    let leftover_vert = (free_rect.length as i64 - length as i64).abs();
    cmp::max(leftover_horiz, leftover_vert) as isize
}

fn score_worst_area_fit(width: usize, length: usize, free_rect: &Rect) -> isize {
    -score_best_area_fit(width, length, free_rect)
}

fn score_worst_short_side_fit(width: usize, length: usize, free_rect: &Rect) -> isize {
    -score_best_short_side_fit(width, length, free_rect)
}

fn score_worst_long_side_fit(width: usize, length: usize, free_rect: &Rect) -> isize {
    -score_best_long_side_fit(width, length, free_rect)
}
