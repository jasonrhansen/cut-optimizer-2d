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

#[derive(Default, Clone)]
struct MockUnit {
    fitness: f64,
}

impl Unit for MockUnit {
    fn fitness(&self) -> f64 {
        self.fitness
    }

    fn breed_with<R>(&self, _other: &Self, _rng: &mut R) -> Self
    where
        R: Rng + ?Sized,
    {
        MockUnit { fitness: 1.0 }
    }
}

#[derive(Default, Clone)]
struct FloatyUnit {
    x: f64,
    y: f64,
}

impl Unit for FloatyUnit {
    fn fitness(&self) -> f64 {
        (self.x + self.y) / 2.0
    }

    fn breed_with<R>(&self, other: &Self, _rng: &mut R) -> Self
    where
        R: Rng + ?Sized,
    {
        FloatyUnit {
            x: self.x * 1.01,
            y: other.y * 1.01,
        }
    }
}

#[derive(Default, Clone)]
struct TendUnit {
    towards: f64,
    x: f64,
}

impl Unit for TendUnit {
    fn fitness(&self) -> f64 {
        -(self.towards - self.x).abs()
    }

    fn breed_with<R>(&self, other: &Self, rng: &mut R) -> Self
    where
        R: Rng + ?Sized,
    {
        TendUnit {
            x: ((self.x + other.x) / 2.0) + rng.gen_range(-0.1..0.1),
            towards: self.towards,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::population::Population;
    use super::{FloatyUnit, MockUnit, TendUnit};

    #[test]
    fn simple_compilation_test() {
        // Add one strong unit and one weak unit.
        let best_units =
            Population::new(vec![MockUnit { fitness: 0.2 }, MockUnit { fitness: 0.1 }])
                .set_size(10)
                .set_breed_factor(1.0)
                .epochs(100, &|_| {})
                .finish();

        assert_eq!(best_units.len(), 10);
        assert_eq!(best_units[0].fitness, 1.0);
    }

    #[test]
    fn basic_algorithm_test() {
        let towards = 10.0;
        let test_vec = vec![
            TendUnit {
                x: 0.3,
                towards: towards,
            },
            TendUnit {
                x: 0.1,
                towards: towards,
            },
            TendUnit {
                x: 0.7,
                towards: towards,
            },
            TendUnit {
                x: 2.3,
                towards: towards,
            },
            TendUnit {
                x: 4.3,
                towards: towards,
            },
        ];

        let best_unit = Population::new(test_vec.clone())
            .set_size(100)
            .set_breed_factor(0.25)
            .epochs(100, &|_| {})
            .finish()
            .get(0)
            .unwrap()
            .clone();

        assert_eq!(best_unit.x.round(), towards);
    }

    #[test]
    fn no_survivors_test() {
        let towards = 10.0;
        let test_vec = vec![
            TendUnit {
                x: 0.3,
                towards: towards,
            },
            TendUnit {
                x: 0.7,
                towards: towards,
            },
        ];

        let best_unit = Population::new(test_vec.clone())
            .set_size(100)
            .set_breed_factor(0.5)
            .set_survival_factor(0.0)
            .epochs(500, &|_| {})
            .finish()
            .get(0)
            .unwrap()
            .clone();

        assert_eq!(best_unit.x.round(), towards);
    }

    #[test]
    fn seeding_test() {
        let test_vec = vec![
            FloatyUnit { x: 0.23, y: 0.12 },
            FloatyUnit { x: 0.1, y: 1.45 },
            FloatyUnit { x: 0.14, y: 2.56 },
            FloatyUnit { x: 3.7, y: 0.1 },
            FloatyUnit { x: 2.6, y: 1.3 },
        ];

        let best_unit_one = Population::new(test_vec.clone())
            .set_size(200)
            .set_rand_seed(10)
            .set_breed_factor(0.3)
            .epochs(200, &|_| {})
            .finish()
            .get(0)
            .unwrap()
            .clone();

        let best_unit_two = Population::new(test_vec.clone())
            .set_size(200)
            .set_rand_seed(10)
            .set_breed_factor(0.3)
            .epochs(200, &|_| {})
            .finish()
            .get(0)
            .unwrap()
            .clone();

        assert_eq!(best_unit_one.x, best_unit_two.x);
        assert_eq!(best_unit_one.y, best_unit_two.y);
    }
}
