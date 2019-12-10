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

use rand::prelude::*;

/// Unit is an abstraction for representing a discrete set of variables to test
/// against a fitness function, and producing children by mutating those
/// variables.
pub trait Unit: Send {
    /// Calculate the relative fitness of this Unit by performing a task using
    /// its variables. Should return a value between 0 and 1, where 1 would be
    /// the maximum success and 0 is utter failure.
    fn fitness(&self) -> f64;

    /// Create a new unit by merging variable qualities from this and one other
    /// unit. The offspring should occasionally experience mutation in random
    /// dimensions.
    fn breed_with<R>(&self, other: &Self, rng: &mut R) -> Self
    where
        R: Rng + ?Sized;
}
