//! cut-optimizer-2d is an optimizer library that attempts layout rectangular cut pieces from stock pieces in a
//! way that gives the least waste. It uses genetic algorithms and multiple heuristics to solve the problem.

#![deny(missing_docs)]

mod genetic;
mod guillotine;
mod maxrects;

use fnv::FnvHashSet;
use genetic::population::Population;
use genetic::unit::Unit;
use guillotine::GuillotineBin;
use maxrects::MaxRectsBin;
use rand::prelude::*;
use rand::seq::SliceRandom;
use std::borrow::Borrow;
use std::cmp;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

/// Indicates the linear direction of a pattern, grain, etc.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "camelCase"))]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum PatternDirection {
    /// No pattern
    None,

    /// Linear pattern that runs parallel to the width
    ParallelToWidth,

    /// Linear pattern that runs parallel to the length
    ParallelToLength,
}

impl PatternDirection {
    /// Returns the opposite orientation of this `PatternDirection`.
    fn rotated(self) -> PatternDirection {
        match self {
            PatternDirection::None => PatternDirection::None,
            PatternDirection::ParallelToWidth => PatternDirection::ParallelToLength,
            PatternDirection::ParallelToLength => PatternDirection::ParallelToWidth,
        }
    }
}

impl Default for PatternDirection {
    fn default() -> Self {
        PatternDirection::None
    }
}

/// A rectangular piece that needs to be cut from a stock piece.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "camelCase"))]
#[derive(Clone, Debug)]
pub struct CutPiece {
    /// ID to be used by the caller to match up result cut pieces
    /// with the original cut piece. This ID has no meaning to the
    /// optimizer so it can be set to `None` if not needed.
    pub external_id: Option<usize>,

    /// Width of this rectangular cut piece.
    pub width: usize,

    /// Length of this rectangular cut piece.
    pub length: usize,

    /// Pattern direction of this cut piece.
    pub pattern_direction: PatternDirection,

    /// Whether or not the optimizer is allowed to rotate this piece to make it fit.
    pub can_rotate: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct CutPieceWithId {
    pub(crate) id: usize,
    pub(crate) external_id: Option<usize>,
    pub(crate) width: usize,
    pub(crate) length: usize,
    pub(crate) pattern_direction: PatternDirection,
    pub(crate) can_rotate: bool,
}

impl Hash for CutPieceWithId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialEq for CutPieceWithId {
    fn eq(&self, other: &CutPieceWithId) -> bool {
        self.id == other.id
    }
}
impl Eq for CutPieceWithId {}

#[derive(Clone, Debug)]
pub(crate) struct UsedCutPiece {
    pub(crate) id: usize,
    pub(crate) external_id: Option<usize>,
    pub(crate) rect: Rect,
    pub(crate) pattern_direction: PatternDirection,
    pub(crate) is_rotated: bool,
    pub(crate) can_rotate: bool,
}

impl PartialEq for UsedCutPiece {
    fn eq(&self, other: &UsedCutPiece) -> bool {
        self.id == other.id
    }
}
impl Eq for UsedCutPiece {}

impl Into<CutPiece> for CutPieceWithId {
    fn into(self) -> CutPiece {
        CutPiece {
            external_id: self.external_id,
            width: self.width,
            length: self.length,
            can_rotate: self.can_rotate,
            pattern_direction: self.pattern_direction,
        }
    }
}

impl Into<CutPieceWithId> for UsedCutPiece {
    fn into(self) -> CutPieceWithId {
        let (width, length, pattern_direction) = if self.is_rotated {
            (
                self.rect.length,
                self.rect.width,
                self.pattern_direction.rotated(),
            )
        } else {
            (self.rect.width, self.rect.length, self.pattern_direction)
        };

        CutPieceWithId {
            id: self.id,
            external_id: self.external_id,
            width,
            length,
            can_rotate: self.can_rotate,
            pattern_direction,
        }
    }
}

impl Into<ResultCutPiece> for UsedCutPiece {
    fn into(self) -> ResultCutPiece {
        ResultCutPiece {
            external_id: self.external_id,
            x: self.rect.x,
            y: self.rect.y,
            width: self.rect.width,
            length: self.rect.length,
            pattern_direction: self.pattern_direction,
            is_rotated: self.is_rotated,
        }
    }
}

/// A cut piece that has been placed in a solution by the optimizer.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "camelCase"))]
#[derive(Clone, Debug)]
pub struct ResultCutPiece {
    /// ID that matches the one on the cut piece that was passed to the optimizer.
    pub external_id: Option<usize>,

    /// X location of the left side of this cut piece within the stock piece.
    pub x: usize,

    /// Y location of the (bottom or top) side of this cut piece within the stock piece.
    pub y: usize,

    /// Width of this cut piece.
    pub width: usize,

    /// Length of this cut piece.
    pub length: usize,

    /// Pattern direction of this cut piece.
    pub pattern_direction: PatternDirection,

    /// Whether or not this cut piece was rotated 90 degrees by the optimizer from it's original
    /// oriorientation.
    pub is_rotated: bool,
}

/// A rectangular stock piece that is available to cut one or more
/// cut pieces from.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "camelCase"))]
#[derive(Hash, Copy, Clone, Debug, Eq, PartialEq)]
pub struct StockPiece {
    /// Width of rectangular stock piece.
    pub width: usize,

    /// Length of rectangular stock piece.
    pub length: usize,

    /// Pattern direction of stock piece.
    pub pattern_direction: PatternDirection,
}

impl StockPiece {
    fn fits_cut_piece(&self, cut_piece: &CutPieceWithId) -> bool {
        let rect = Rect {
            x: 0,
            y: 0,
            width: self.width,
            length: self.length,
        };

        rect.fit_cut_piece(self.pattern_direction, cut_piece) != Fit::None
    }
}

/// Stock piece that was used by the optimizer to get one or more cut pieces.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "camelCase"))]
#[derive(Clone, Debug)]
pub struct ResultStockPiece {
    /// Width of this stock piece.
    pub width: usize,

    /// Length of this stock piece.
    pub length: usize,

    /// Pattern direction of this stock piece.
    pub pattern_direction: PatternDirection,

    /// Cut pieces to cut from this stock piece.
    pub cut_pieces: Vec<ResultCutPiece>,

    /// Waste pieces that remain after cutting the cut pieces.
    pub waste_pieces: Vec<Rect>,
}

/// A rectangle
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "camelCase"))]
#[derive(Copy, Clone, Debug, Default)]
pub struct Rect {
    /// X location of this rectangle.
    x: usize,

    /// Y location of this rectangle.
    y: usize,

    /// Width of this rectangle.
    width: usize,

    /// Length of this rectangle.
    length: usize,
}

impl Rect {
    fn fit_cut_piece(
        &self,
        pattern_direction: PatternDirection,
        cut_piece: &CutPieceWithId,
    ) -> Fit {
        if cut_piece.pattern_direction == pattern_direction {
            if cut_piece.width == self.width && cut_piece.length == self.length {
                return Fit::UprightExact;
            } else if cut_piece.width <= self.width && cut_piece.length <= self.length {
                return Fit::Upright;
            }
        }

        if cut_piece.can_rotate && cut_piece.pattern_direction.rotated() == pattern_direction {
            if cut_piece.length == self.width && cut_piece.width == self.length {
                return Fit::RotatedExact;
            } else if cut_piece.length <= self.width && cut_piece.width <= self.length {
                return Fit::Rotated;
            }
        }

        Fit::None
    }

    fn contains(&self, rect: &Rect) -> bool {
        rect.x >= self.x
            && rect.x + rect.width <= self.x + self.width
            && rect.y >= self.y
            && rect.y + rect.length <= self.y + self.length
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Fit {
    None,
    UprightExact,
    RotatedExact,
    Upright,
    Rotated,
}

impl Fit {
    fn is_none(self) -> bool {
        self == Fit::None
    }

    fn is_upright(self) -> bool {
        self == Fit::Upright || self == Fit::UprightExact
    }

    fn is_rotated(self) -> bool {
        self == Fit::Rotated || self == Fit::RotatedExact
    }
}

/// Represents a bin used for bin-packing.
trait Bin {
    /// Heuristic used for inserting `CutPiece`s.
    type Heuristic;

    /// Creates a new `Bin`.
    fn new(
        width: usize,
        length: usize,
        blade_width: usize,
        pattern_direction: PatternDirection,
    ) -> Self;

    /// Computes the fitness of this `Bin` on a scale of 0.0 to 1.0, with 1.0 being the most fit.
    fn fitness(&self) -> f64;

    /// Removes `UsedCutPiece`s from this `Bin` and returns how many were removed.
    fn remove_cut_pieces<I>(&mut self, cut_pieces: I) -> usize
    where
        I: Iterator,
        I::Item: Borrow<UsedCutPiece>;

    /// Returns an iterator over the `UsedCutPiece`s in this `Bin`.
    fn cut_pieces(&self) -> std::slice::Iter<'_, UsedCutPiece>;

    /// Returns the possible heuristics that can be passed to `insert_cut_piece_with_heuristic`.
    fn possible_heuristics() -> Vec<Self::Heuristic>;

    /// Inserts the `CutPieceWithId` into this `Bin` using the specified heuristic. Returns whether
    /// the insert succeeded.
    fn insert_cut_piece_with_heuristic(
        &mut self,
        cut_piece: &CutPieceWithId,
        heuristic: &Self::Heuristic,
    ) -> bool;

    /// Inserts the `CutPieceWithId` into this `Bin` using a random heuristic. Returns whether
    /// the insert succeeded.
    fn insert_cut_piece_random_heuristic<R>(
        &mut self,
        cut_piece: &CutPieceWithId,
        rng: &mut R,
    ) -> bool
    where
        R: Rng + ?Sized;
}

#[derive(Debug)]
struct OptimizerUnit<'a, B>
where
    B: Bin,
{
    bins: Vec<B>,
    possible_stock_pieces: &'a [StockPiece],
    blade_width: usize,
}

impl<'a, B> OptimizerUnit<'a, B>
where
    B: Bin,
{
    fn with_random_heuristics<R>(
        possible_stock_pieces: &'a [StockPiece],
        cut_pieces: &[&CutPieceWithId],
        blade_width: usize,
        rng: &mut R,
    ) -> Result<OptimizerUnit<'a, B>>
    where
        R: Rng + ?Sized,
    {
        let mut unit = OptimizerUnit {
            bins: Vec::new(),
            possible_stock_pieces,
            blade_width,
        };

        for cut_piece in cut_pieces {
            if !unit.first_fit_random_heuristics(cut_piece, rng) {
                return Err(no_fit_for_cut_piece_error(cut_piece));
            }
        }

        Ok(unit)
    }

    fn with_heuristic<R>(
        possible_stock_pieces: &'a [StockPiece],
        cut_pieces: &[&CutPieceWithId],
        blade_width: usize,
        heuristic: &B::Heuristic,
        rng: &mut R,
    ) -> Result<OptimizerUnit<'a, B>>
    where
        R: Rng + ?Sized,
    {
        let mut unit = OptimizerUnit {
            bins: Vec::new(),
            possible_stock_pieces,
            blade_width,
        };

        for cut_piece in cut_pieces {
            if !unit.first_fit_with_heuristic(cut_piece, heuristic, rng) {
                return Err(no_fit_for_cut_piece_error(cut_piece));
            }
        }

        Ok(unit)
    }

    pub(crate) fn generate_initial_units(
        possible_stock_pieces: &'a [StockPiece],
        mut cut_pieces: Vec<&CutPieceWithId>,
        blade_width: usize,
        random_seed: u64,
    ) -> Result<Vec<OptimizerUnit<'a, B>>> {
        let mut set = HashSet::new();
        for cut_piece in &cut_pieces {
            set.insert((
                cut_piece.width,
                cut_piece.length,
                cut_piece.can_rotate,
                cut_piece.pattern_direction,
            ));
        }
        let unique_cut_pieces = set.len();

        let possible_heuristics = B::possible_heuristics();

        let num_units = if cut_pieces.len() < 3 {
            possible_heuristics.len()
        } else {
            let denom = if cut_pieces.len() > 1 {
                (cut_pieces.len() as f64).log10()
            } else {
                1.0
            };

            cmp::max(
                possible_heuristics.len() * 3,
                (cut_pieces.len() as f64 / denom + ((unique_cut_pieces - 1) * 10) as f64) as usize,
            )
        };
        let mut units = Vec::with_capacity(num_units);
        let mut rng: StdRng = SeedableRng::seed_from_u64(random_seed);

        cut_pieces.sort_by_key(|p| cmp::Reverse((p.width, p.length)));
        for heuristic in &possible_heuristics {
            units.push(OptimizerUnit::with_heuristic(
                possible_stock_pieces,
                &cut_pieces,
                blade_width,
                heuristic,
                &mut rng,
            )?);
        }

        if cut_pieces.len() > 2 {
            for heuristic in &possible_heuristics {
                cut_pieces.shuffle(&mut rng);
                units.push(OptimizerUnit::with_heuristic(
                    possible_stock_pieces,
                    &cut_pieces,
                    blade_width,
                    heuristic,
                    &mut rng,
                )?);
            }

            for _ in 0..num_units - units.len() {
                cut_pieces.shuffle(&mut rng);
                units.push(OptimizerUnit::with_random_heuristics(
                    possible_stock_pieces,
                    &cut_pieces,
                    blade_width,
                    &mut rng,
                )?);
            }
        }
        Ok(units)
    }

    fn first_fit_random_heuristics<R>(&mut self, cut_piece: &CutPieceWithId, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        for bin in self.bins.iter_mut() {
            if bin.insert_cut_piece_random_heuristic(cut_piece, rng) {
                return true;
            }
        }

        self.add_to_new_bin(cut_piece, rng)
    }

    fn first_fit_with_heuristic<R>(
        &mut self,
        cut_piece: &CutPieceWithId,
        heuristic: &B::Heuristic,
        rng: &mut R,
    ) -> bool
    where
        R: Rng + ?Sized,
    {
        for bin in self.bins.iter_mut() {
            if bin.insert_cut_piece_with_heuristic(cut_piece, heuristic) {
                return true;
            }
        }

        self.add_to_new_bin(cut_piece, rng)
    }

    fn add_to_new_bin<R>(&mut self, cut_piece: &CutPieceWithId, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        let possible_stock_pieces: Vec<&StockPiece> = self
            .possible_stock_pieces
            .iter()
            .filter(|stock_piece| stock_piece.fits_cut_piece(cut_piece))
            .collect();

        if let Some(stock_piece) = possible_stock_pieces.choose(rng) {
            let mut bin = B::new(
                stock_piece.width,
                stock_piece.length,
                self.blade_width,
                stock_piece.pattern_direction,
            );
            if !bin.insert_cut_piece_random_heuristic(cut_piece, rng) {
                return false;
            }
            self.bins.push(bin);
            true
        } else {
            false
        }
    }

    fn crossover<R>(&self, other: &OptimizerUnit<'a, B>, rng: &mut R) -> OptimizerUnit<'a, B>
    where
        R: Rng + ?Sized,
        B: Clone,
    {
        let cross_dest = rng.gen_range(0..self.bins.len() + 1);
        let cross_src_start = rng.gen_range(0..other.bins.len());
        let cross_src_end = rng.gen_range(cross_src_start + 1..other.bins.len() + 1);

        let mut new_unit = OptimizerUnit {
            // Inject bins between crossing sites of other.
            bins: (&self.bins[..cross_dest])
                .iter()
                .chain((&other.bins[cross_src_start..cross_src_end]).iter())
                .chain((&self.bins[cross_dest..]).iter())
                .cloned()
                .collect(),
            possible_stock_pieces: self.possible_stock_pieces,
            blade_width: self.blade_width,
        };

        let mut removed_cut_pieces: Vec<UsedCutPiece> = Vec::new();
        for i in (0..cross_dest)
            .chain(cross_dest + cross_src_end - cross_src_start..new_unit.bins.len())
            .rev()
        {
            let bin = &mut new_unit.bins[i];
            let injected_cut_pieces = (&other.bins[cross_src_start..cross_src_end])
                .iter()
                .flat_map(Bin::cut_pieces)
                .cloned();
            if bin.remove_cut_pieces(injected_cut_pieces) > 0 {
                for cut_piece in bin.cut_pieces().cloned() {
                    removed_cut_pieces.push(cut_piece);
                }
                new_unit.bins.remove(i);
            }
        }

        for cut_piece in removed_cut_pieces {
            new_unit.first_fit_random_heuristics(&cut_piece.into(), rng);
        }

        // Only keep bins that have cut_pieces
        new_unit
            .bins
            .retain(|bin| bin.cut_pieces().next().is_some());

        new_unit
    }

    // Randomly apply a mutation to this unit.
    fn mutate<R>(&mut self, rng: &mut R)
    where
        R: Rng + ?Sized,
    {
        if let 1 = rng.gen_range(0..20) {
            self.inversion(rng)
        }
    }

    // Reverse order of a random range of bins.
    fn inversion<R>(&mut self, rng: &mut R)
    where
        R: Rng + ?Sized,
    {
        let start = rng.gen_range(0..self.bins.len());
        let end = rng.gen_range(start..self.bins.len());
        self.bins[start..end].reverse();
    }
}

impl<'a, B> Unit for OptimizerUnit<'a, B>
where
    B: Bin + Send + Clone,
{
    fn fitness(&self) -> f64 {
        self.bins.iter().fold(0.0, |acc, b| acc + b.fitness()) / self.bins.len() as f64
    }

    fn breed_with<R>(&self, other: &OptimizerUnit<'a, B>, rng: &mut R) -> OptimizerUnit<'a, B>
    where
        R: Rng + ?Sized,
    {
        let mut new_unit = self.crossover(other, rng);
        new_unit.mutate(rng);
        new_unit
    }
}

/// Error while optimizing.
pub enum Error {
    /// There was no stock piece that could contain this demand piece.
    NoFitForCutPiece(CutPiece),
}
fn no_fit_for_cut_piece_error(cut_piece: &CutPieceWithId) -> Error {
    Error::NoFitForCutPiece(CutPiece {
        external_id: cut_piece.external_id,
        width: cut_piece.width,
        length: cut_piece.length,
        can_rotate: cut_piece.can_rotate,
        pattern_direction: cut_piece.pattern_direction,
    })
}
type Result<T> = std::result::Result<T, Error>;

/// A valid solution to an optimization.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "camelCase"))]
pub struct Solution {
    /// Fitness score for this solution.
    /// Ranges between 0.0 and 1.0 inclusive, with 1.0 being a perfect solution with no waste.
    pub fitness: f64,

    /// The stock pieces that were used for this solution, each containing the demand piece layout.
    pub stock_pieces: Vec<ResultStockPiece>,
}

/// Optimizer for optimizing rectangular cut pieces from rectangular
/// stock pieces.
#[derive(Default)]
pub struct Optimizer {
    stock_pieces: FnvHashSet<StockPiece>,
    cut_pieces: Vec<CutPieceWithId>,
    cut_width: usize,
    random_seed: u64,
}

impl Optimizer {
    /// Create a new optimizer.
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a stock piece that the optimizer can use to optimize cut pieces. Each
    /// unique stock piece only needs to be added once.
    pub fn add_stock_piece(&mut self, stock_piece: StockPiece) -> &mut Self {
        self.stock_pieces.insert(stock_piece);
        self
    }

    /// Add stock pieces that the optimizer can use to optimize cut pieces. Each
    /// unique stock piece only needs to be added once.
    pub fn add_stock_pieces<I>(&mut self, stock_pieces: I) -> &mut Self
    where
        I: IntoIterator<Item = StockPiece>,
    {
        self.stock_pieces.extend(stock_pieces);
        self
    }

    /// Add a desired cut piece that you need cut from a stock piece.
    pub fn add_cut_piece(&mut self, cut_piece: CutPiece) -> &mut Self {
        let cut_piece = CutPieceWithId {
            id: self.cut_pieces.len(),
            external_id: cut_piece.external_id,
            width: cut_piece.width,
            length: cut_piece.length,
            pattern_direction: cut_piece.pattern_direction,
            can_rotate: cut_piece.can_rotate,
        };

        self.cut_pieces.push(cut_piece);
        self
    }

    /// Add desired cut pieces that you need cut from a stock piece.
    pub fn add_cut_pieces<I>(&mut self, cut_pieces: I) -> &mut Self
    where
        I: IntoIterator<Item = CutPiece>,
    {
        cut_pieces.into_iter().for_each(|dp| {
            self.add_cut_piece(dp);
        });
        self
    }

    /// Set the width of the cut to use between cut pieces. This could
    /// represent blade or kerf thickness.
    pub fn set_cut_width(&mut self, cut_width: usize) -> &mut Self {
        self.cut_width = cut_width;
        self
    }

    /// Set the random seed used by the genetic algorithms in the optimizer. Using
    /// the same random seed will give you the same result for the same input.
    pub fn set_random_seed(&mut self, seed: u64) -> &mut Self {
        self.random_seed = seed;
        self
    }

    /// Optimize in a way where each cut piece can be cut out using only guillotine cuts,
    /// where each cut extends from one side to the other.
    ///
    /// This method is suitable for cutting with a panel saw.
    pub fn optimize_guillotine<F>(&self, progress_callback: F) -> Result<Solution>
    where
        F: Fn(f64),
    {
        let (fitness, stock_pieces) = self.optimize::<GuillotineBin, F>(progress_callback)?;
        Ok(Solution {
            fitness,
            stock_pieces,
        })
    }

    /// Optimize without the requirement of guillotine cuts. Cuts can start and stop in the middle
    /// of the stock piece.
    ///
    /// This method is suitable for cutting on a CNC.
    pub fn optimize_nested<F>(&self, progress_callback: F) -> Result<Solution>
    where
        F: Fn(f64),
    {
        let (fitness, stock_pieces) = self.optimize::<MaxRectsBin, F>(progress_callback)?;
        Ok(Solution {
            fitness,
            stock_pieces,
        })
    }

    fn optimize<B, F>(&self, progress_callback: F) -> Result<(f64, Vec<ResultStockPiece>)>
    where
        B: Bin + Clone + Send + Into<ResultStockPiece>,
        F: Fn(f64),
    {
        let size_set: FnvHashSet<(usize, usize)> = self
            .stock_pieces
            .iter()
            .map(|sp| (sp.width, sp.length))
            .collect();

        let num_runs = size_set.len() + 1;
        let callback = |progress| {
            progress_callback(progress / num_runs as f64);
        };

        // Optimize with all stock sizes
        let mut best_result = self.optimize_with_stock_pieces::<B, _>(
            &self.stock_pieces.iter().cloned().collect::<Vec<_>>(),
            &callback,
        );

        // Optimize each stock size separately and see if any have better result than
        // when optimizing with all stock sizes.
        for (i, (width, length)) in size_set.iter().enumerate() {
            let stock_pieces: Vec<StockPiece> = self
                .stock_pieces
                .iter()
                .filter(|sp| sp.width == *width && sp.length == *length)
                .cloned()
                .collect();

            let completed_runs = i + 1;
            if let Ok((fitness, used_stock_pieces)) =
                self.optimize_with_stock_pieces::<B, _>(&stock_pieces, &|progress| {
                    progress_callback((completed_runs as f64 + progress) / num_runs as f64);
                })
            {
                match best_result {
                    Ok((best_fitness, _)) => {
                        if fitness > best_fitness {
                            best_result = Ok((fitness, used_stock_pieces));
                        }
                    }
                    Err(_) => best_result = Ok((fitness, used_stock_pieces)),
                }
            }
        }

        if let Ok((_, ref mut used_stock_pieces)) = &mut best_result {
            used_stock_pieces.sort_by_key(|p| cmp::Reverse((p.width, p.length)));
        };

        best_result
    }

    fn optimize_with_stock_pieces<B, F>(
        &self,
        stock_pieces: &[StockPiece],
        progress_callback: &F,
    ) -> Result<(f64, Vec<ResultStockPiece>)>
    where
        B: Bin + Clone + Send + Into<ResultStockPiece>,
        F: Fn(f64),
    {
        let cut_pieces: Vec<&CutPieceWithId> = self.cut_pieces.iter().collect();

        let units: Vec<OptimizerUnit<B>> = OptimizerUnit::generate_initial_units(
            stock_pieces,
            cut_pieces,
            self.cut_width,
            self.random_seed,
        )?;

        let population_size = units.len();
        let mut result_units = Population::new(units)
            .set_size(population_size)
            .set_rand_seed(self.random_seed)
            .set_breed_factor(0.5)
            .set_survival_factor(0.6)
            .epochs(100, progress_callback)
            .finish();

        let best_unit = &mut result_units[0];
        let fitness = best_unit.fitness();

        let used_stock_pieces: Vec<ResultStockPiece> =
            best_unit.bins.drain(..).map(Into::into).collect();

        Ok((fitness, used_stock_pieces))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    static STOCK_PIECES: &[StockPiece] = &[
        StockPiece {
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
        },
        StockPiece {
            width: 48,
            length: 120,
            pattern_direction: PatternDirection::None,
        },
    ];

    static CUT_PIECES: &[CutPiece] = &[
        CutPiece {
            external_id: Some(1),
            width: 10,
            length: 30,
            pattern_direction: PatternDirection::None,
            can_rotate: true,
        },
        CutPiece {
            external_id: Some(2),
            width: 20,
            length: 30,
            pattern_direction: PatternDirection::None,
            can_rotate: true,
        },
        CutPiece {
            external_id: Some(3),
            width: 30,
            length: 30,
            pattern_direction: PatternDirection::None,
            can_rotate: true,
        },
        CutPiece {
            external_id: Some(4),
            width: 40,
            length: 30,
            pattern_direction: PatternDirection::None,
            can_rotate: true,
        },
    ];

    #[test]
    fn test_guillotine() {
        let result = Optimizer::new()
            .add_stock_pieces(STOCK_PIECES.iter().cloned().collect::<Vec<_>>())
            .add_cut_pieces(CUT_PIECES.iter().cloned().collect::<Vec<_>>())
            .set_cut_width(1)
            .set_random_seed(1)
            .optimize_guillotine(|_| {});

        assert!(result.is_ok());
    }

    #[test]
    fn test_guillotine_rotate() {
        let result = Optimizer::new()
            .add_stock_piece(StockPiece {
                width: 10,
                length: 11,
                pattern_direction: PatternDirection::None,
            })
            .add_cut_piece(CutPiece {
                external_id: Some(1),
                width: 11,
                length: 10,
                pattern_direction: PatternDirection::None,
                can_rotate: true,
            })
            .set_cut_width(1)
            .set_random_seed(1)
            .optimize_guillotine(|_| {});

        assert!(result.is_ok());
    }

    #[test]
    fn test_guillotine_rotate_pattern() {
        let result = Optimizer::new()
            .add_stock_piece(StockPiece {
                width: 10,
                length: 11,
                pattern_direction: PatternDirection::ParallelToWidth,
            })
            .add_cut_piece(CutPiece {
                external_id: Some(1),
                width: 11,
                length: 10,
                pattern_direction: PatternDirection::ParallelToLength,
                can_rotate: true,
            })
            .set_cut_width(1)
            .set_random_seed(1)
            .optimize_guillotine(|_| {});

        assert!(result.is_ok());
    }

    #[test]
    fn test_non_fitting_cut_piece_guillotine_can_rotate() {
        let result = Optimizer::new()
            .add_stock_piece(StockPiece {
                width: 10,
                length: 10,
                pattern_direction: PatternDirection::None,
            })
            .add_cut_piece(CutPiece {
                external_id: Some(1),
                width: 11,
                length: 10,
                pattern_direction: PatternDirection::None,
                can_rotate: true,
            })
            .set_cut_width(1)
            .set_random_seed(1)
            .optimize_guillotine(|_| {});

        match result {
            Err(Error::NoFitForCutPiece(_)) => {}
            _ => {
                panic!("should have returned Error::NoFitForCutPiece");
            }
        }
    }

    #[test]
    fn test_non_fitting_cut_piece_guillotine_no_rotate() {
        let result = Optimizer::new()
            .add_stock_piece(StockPiece {
                width: 10,
                length: 11,
                pattern_direction: PatternDirection::None,
            })
            .add_cut_piece(CutPiece {
                external_id: Some(1),
                width: 11,
                length: 10,
                pattern_direction: PatternDirection::None,
                can_rotate: false,
            })
            .set_cut_width(1)
            .set_random_seed(1)
            .optimize_guillotine(|_| {});

        match result {
            Err(Error::NoFitForCutPiece(_)) => {}
            _ => {
                panic!("should have returned Error::NoFitForCutPiece");
            }
        }
    }

    #[test]
    fn test_non_fitting_cut_piece_guillotine_no_rotate_pattern() {
        let result = Optimizer::new()
            .add_stock_piece(StockPiece {
                width: 10,
                length: 11,
                pattern_direction: PatternDirection::ParallelToWidth,
            })
            .add_cut_piece(CutPiece {
                external_id: Some(1),
                width: 11,
                length: 10,
                pattern_direction: PatternDirection::ParallelToLength,
                can_rotate: false,
            })
            .set_cut_width(1)
            .set_random_seed(1)
            .optimize_guillotine(|_| {});

        match result {
            Err(Error::NoFitForCutPiece(_)) => {}
            _ => {
                panic!("should have returned Error::NoFitForCutPiece");
            }
        }
    }

    #[test]
    fn test_non_fitting_cut_piece_guillotine_mismatched_pattern() {
        let result = Optimizer::new()
            .add_stock_piece(StockPiece {
                width: 100,
                length: 100,
                pattern_direction: PatternDirection::None,
            })
            .add_cut_piece(CutPiece {
                external_id: Some(1),
                width: 11,
                length: 10,
                pattern_direction: PatternDirection::ParallelToWidth,
                can_rotate: true,
            })
            .set_cut_width(1)
            .set_random_seed(1)
            .optimize_guillotine(|_| {});

        match result {
            Err(Error::NoFitForCutPiece(_)) => {}
            _ => {
                panic!("should have returned Error::NoFitForCutPiece");
            }
        }
    }

    #[test]
    fn test_nested() {
        let result = Optimizer::new()
            .add_stock_pieces(STOCK_PIECES.iter().cloned().collect::<Vec<_>>())
            .add_cut_pieces(CUT_PIECES.iter().cloned().collect::<Vec<_>>())
            .set_cut_width(1)
            .set_random_seed(1)
            .optimize_nested(|_| {});

        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_rotate() {
        let result = Optimizer::new()
            .add_stock_piece(StockPiece {
                width: 10,
                length: 11,
                pattern_direction: PatternDirection::None,
            })
            .add_cut_piece(CutPiece {
                external_id: Some(1),
                width: 11,
                length: 10,
                pattern_direction: PatternDirection::None,
                can_rotate: true,
            })
            .set_cut_width(1)
            .set_random_seed(1)
            .optimize_nested(|_| {});

        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_rotate_pattern() {
        let result = Optimizer::new()
            .add_stock_piece(StockPiece {
                width: 10,
                length: 11,
                pattern_direction: PatternDirection::ParallelToWidth,
            })
            .add_cut_piece(CutPiece {
                external_id: Some(1),
                width: 11,
                length: 10,
                pattern_direction: PatternDirection::ParallelToLength,
                can_rotate: true,
            })
            .set_cut_width(1)
            .set_random_seed(1)
            .optimize_nested(|_| {});

        assert!(result.is_ok());
    }

    #[test]
    fn test_non_fitting_cut_piece_nested_can_rotate() {
        let result = Optimizer::new()
            .add_stock_piece(StockPiece {
                width: 10,
                length: 10,
                pattern_direction: PatternDirection::None,
            })
            .add_cut_piece(CutPiece {
                external_id: Some(1),
                width: 11,
                length: 10,
                pattern_direction: PatternDirection::None,
                can_rotate: true,
            })
            .set_cut_width(1)
            .set_random_seed(1)
            .optimize_nested(|_| {});

        match result {
            Err(Error::NoFitForCutPiece(_)) => {}
            _ => {
                panic!("should have returned Error::NoFitForCutPiece");
            }
        }
    }

    #[test]
    fn test_non_fitting_cut_piece_nested_no_rotate() {
        let result = Optimizer::new()
            .add_stock_piece(StockPiece {
                width: 10,
                length: 11,
                pattern_direction: PatternDirection::None,
            })
            .add_cut_piece(CutPiece {
                external_id: Some(1),
                width: 11,
                length: 10,
                pattern_direction: PatternDirection::None,
                can_rotate: false,
            })
            .set_cut_width(1)
            .set_random_seed(1)
            .optimize_nested(|_| {});

        match result {
            Err(Error::NoFitForCutPiece(_)) => {}
            _ => {
                panic!("should have returned Error::NoFitForCutPiece");
            }
        }
    }

    #[test]
    fn test_non_fitting_cut_piece_nested_no_rotate_pattern() {
        let result = Optimizer::new()
            .add_stock_piece(StockPiece {
                width: 10,
                length: 11,
                pattern_direction: PatternDirection::ParallelToWidth,
            })
            .add_cut_piece(CutPiece {
                external_id: Some(1),
                width: 11,
                length: 10,
                pattern_direction: PatternDirection::ParallelToLength,
                can_rotate: false,
            })
            .set_cut_width(1)
            .set_random_seed(1)
            .optimize_nested(|_| {});

        match result {
            Err(Error::NoFitForCutPiece(_)) => {}
            _ => {
                panic!("should have returned Error::NoFitForCutPiece");
            }
        }
    }

    #[test]
    fn test_non_fitting_cut_piece_nested_mismatched_pattern() {
        let result = Optimizer::new()
            .add_stock_piece(StockPiece {
                width: 100,
                length: 100,
                pattern_direction: PatternDirection::None,
            })
            .add_cut_piece(CutPiece {
                external_id: Some(1),
                width: 11,
                length: 10,
                pattern_direction: PatternDirection::ParallelToWidth,
                can_rotate: true,
            })
            .set_cut_width(1)
            .set_random_seed(1)
            .optimize_nested(|_| {});

        match result {
            Err(Error::NoFitForCutPiece(_)) => {}
            _ => {
                panic!("should have returned Error::NoFitForCutPiece");
            }
        }
    }
}
