[![Worflows](https://github.com/jasonrhansen/cut-optimizer-2d/actions/workflows/rust.yml/badge.svg)](https://github.com/jasonrhansen/cut-optimizer-2d/actions)
[![Crates.io](https://img.shields.io/crates/v/cut-optimizer-2d.svg)](https://crates.io/crates/cut-optimizer-2d)
[![Documentation](https://docs.rs/cut-optimizer-2d/badge.svg)](https://docs.rs/cut-optimizer-2d/)
[![Dependency status](https://deps.rs/repo/github/jasonrhansen/cut-optimizer-2d/status.svg)](https://deps.rs/repo/github/jasonrhansen/cut-optimizer-2d)

# Cut Optimizer 2D

## Description

Cut Optimizer 2D is a cut optimizer library for optimizing rectangular cut pieces
from sheet goods.

Given desired cut pieces and stock sheets, it will attempt to layout the cut
pieces in way that gives the least waste.
It can't guarantee the most optimizal solution possible, since this would be too
inefficient. Instead it uses genetic
algorithms and multiple heuristics to solve the problem. This usually results in
a satisfactory solution.

## License

Duel-license under MIT license ([LICENSE-MIT](LICENSE-MIT)), or Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
