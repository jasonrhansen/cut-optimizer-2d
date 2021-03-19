// Copyright (c) 2017 Ashley Jeffs
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use super::unit::Unit;

use rand::prelude::*;

use std::cmp::Ordering;
use std::mem;

/// Wraps a unit within a struct that lazily evaluates its fitness to avoid
/// duplicate work.
struct LazyUnit<T: Unit> {
    unit: T,
    lazy_fitness: Option<f64>,
}

impl<T: Unit> LazyUnit<T> {
    fn from(unit: T) -> Self {
        LazyUnit {
            unit,
            lazy_fitness: None,
        }
    }

    fn fitness(&mut self) -> f64 {
        match self.lazy_fitness {
            Some(x) => x,
            None => {
                let fitness = self.unit.fitness();
                self.lazy_fitness = Some(fitness);
                fitness
            }
        }
    }
}

/// Population is an abstraction that represents a collection of units. Each
/// unit is a combination of variables, which produces an overall fitness. Units
/// mate with other units to produce mutated offspring combining traits from
/// both units.
///
/// The population is responsible for iterating new generations of units by
/// mating fit units and killing unfit units.
pub struct Population<T: Unit> {
    units: Vec<T>,

    seed: u64,
    breed_factor: f64,
    survival_factor: f64,
    max_size: usize,
}

impl<T: Unit> Population<T> {
    /// Creates a new population with the passed-in units.
    pub fn new(init_pop: Vec<T>) -> Self {
        Population {
            units: init_pop,
            seed: 1,
            breed_factor: 0.5,
            survival_factor: 0.5,
            max_size: 100,
        }
    }

    //--------------------------------------------------------------------------

    /// Sets the random seed of the population.
    pub fn set_rand_seed(&mut self, seed: u64) -> &mut Self {
        self.seed = seed;
        self
    }

    /// Sets the maximum size of the population. If already populated with more
    /// than this amount a random section of the population is killed.
    pub fn set_size(&mut self, size: usize) -> &mut Self {
        self.units.truncate(size);
        self.max_size = size;
        self
    }

    /// Sets the breed_factor (0 < b <= 1) of the genetic algorithm, which is
    /// the percentage of the population that will be able to breed per epoch.
    /// Units that are more fit are preferred for breeding, and so a high
    /// breed_factor results in more poorly performing units being able to
    /// breed, which will slow the algorithm down but allow it to escape local
    /// peaks.
    pub fn set_breed_factor(&mut self, breed_factor: f64) -> &mut Self {
        assert!(breed_factor > 0.0 && breed_factor <= 1.0);
        self.breed_factor = breed_factor;
        self
    }

    /// Sets the survival_factor (0 <= b <= 1) of the genetic algorithm, which
    /// is the percentage of the breeding population that will survive each
    /// epoch. Units that are more fit are preferred for survival, and so a high
    /// survival rate results in more poorly performing units being carried into
    /// the next epoch.
    ///
    /// Note that this value is a percentage of the breeding population. So if
    /// your breeding factor is 0.5, and your survival factor is 0.9, the
    /// percentage of units that will survive the next epoch is:
    ///
    /// 0.5 * 0.9 * 100 = 45%
    ///
    pub fn set_survival_factor(&mut self, survival_factor: f64) -> &mut Self {
        assert!((0.0..=1.0).contains(&survival_factor));
        self.survival_factor = survival_factor;
        self
    }

    //--------------------------------------------------------------------------

    /// An epoch that allows units to breed and mutate without harsh culling.
    /// It's important to sometimes allow 'weak' units to produce generations
    /// that might escape local peaks in certain dimensions.
    fn epoch(&self, units: &mut Vec<LazyUnit<T>>, mut rng: StdRng) -> StdRng {
        assert!(!units.is_empty());

        // breed_factor dicates how large a percentage of the population will be
        // able to breed.
        let breed_up_to = (self.breed_factor * (units.len() as f64)) as usize;
        let mut breeders: Vec<LazyUnit<T>> = Vec::new();

        while let Some(unit) = units.pop() {
            breeders.push(unit);
            if breeders.len() == breed_up_to {
                break;
            }
        }
        units.clear();

        // The strongest half of our breeders will survive each epoch. Always at
        // least one.
        let surviving_parents = (breeders.len() as f64 * self.survival_factor).ceil() as usize;

        for i in 0..self.max_size - surviving_parents {
            let rs = rng.gen_range(0..breeders.len());
            units.push(LazyUnit::from(
                breeders[i % breeders.len()]
                    .unit
                    .breed_with(&breeders[rs].unit, &mut rng),
            ));
        }

        // Move our survivors into the new generation.
        units.append(&mut breeders.drain(0..surviving_parents).collect());

        rng
    }

    /// Runs a number of epochs.
    pub fn epochs<F>(&mut self, n_epochs: u32, progress_callback: &F) -> &mut Self
    where
        F: Fn(f64),
    {
        let mut processed_stack = Vec::new();
        let mut active_stack = Vec::new();

        while let Some(unit) = self.units.pop() {
            active_stack.push(LazyUnit::from(unit));
        }

        let mut rng = SeedableRng::seed_from_u64(self.seed);

        for i in 0..=n_epochs {
            while let Some(mut unit) = active_stack.pop() {
                unit.fitness();
                processed_stack.push(unit);
            }

            // Swap the full processed_stack with the active stack.
            mem::swap(&mut active_stack, &mut processed_stack);

            // We want to sort such that highest fitness units are at the
            // end.
            active_stack.sort_by(|a, b| {
                a.lazy_fitness
                    .unwrap_or(0.0)
                    .partial_cmp(&b.lazy_fitness.unwrap_or(0.0))
                    .unwrap_or(Ordering::Equal)
            });

            // If we have the perfect solution then break early.
            if active_stack.last().unwrap().lazy_fitness.unwrap_or(0.0) >= 1.0 {
                break;
            }

            if i != n_epochs {
                rng = self.epoch(&mut active_stack, rng);
            }

            progress_callback(i as f64 / n_epochs as f64);
        }

        // Reverse the order of units such that the first unit is the
        // strongest candidate.
        while let Some(unit) = active_stack.pop() {
            self.units.push(unit.unit);
        }

        self
    }

    //--------------------------------------------------------------------------

    /// Returns the full population of units, ordered such that the first
    /// element is the strongest candidate. This collection can be used to
    /// create a new population.
    pub fn finish(&mut self) -> Vec<T> {
        let mut empty_units: Vec<T> = Vec::new();
        mem::swap(&mut empty_units, &mut self.units);
        empty_units
    }
}
