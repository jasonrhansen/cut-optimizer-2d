use super::*;

static STOCK_PIECES: &[StockPiece] = &[
    StockPiece {
        width: 48,
        length: 96,
        pattern_direction: PatternDirection::None,
        price: 0,
        quantity: None,
    },
    StockPiece {
        width: 48,
        length: 120,
        pattern_direction: PatternDirection::None,
        price: 0,
        quantity: None,
    },
];

static CUT_PIECES: &[CutPiece] = &[
    CutPiece {
        quantity: 1,
        external_id: Some(1),
        width: 10,
        length: 30,
        pattern_direction: PatternDirection::None,
        can_rotate: true,
    },
    CutPiece {
        quantity: 1,
        external_id: Some(2),
        width: 20,
        length: 30,
        pattern_direction: PatternDirection::None,
        can_rotate: true,
    },
    CutPiece {
        quantity: 1,
        external_id: Some(3),
        width: 30,
        length: 30,
        pattern_direction: PatternDirection::None,
        can_rotate: true,
    },
    CutPiece {
        quantity: 1,
        external_id: Some(4),
        width: 40,
        length: 30,
        pattern_direction: PatternDirection::None,
        can_rotate: true,
    },
];

fn sanity_check_solution(solution: &Solution, num_cut_pieces: usize) {
    let stock_pieces = &solution.stock_pieces;

    assert!(solution.fitness <= 1.0);

    // The number of result cut pieces should match the number of input cut pieces.
    assert_eq!(
        stock_pieces
            .iter()
            .map(|sp| sp.cut_pieces.len())
            .sum::<usize>(),
        num_cut_pieces
    );

    for stock_piece in stock_pieces {
        for cut_piece in &stock_piece.cut_pieces {
            assert_eq!(stock_piece.pattern_direction, cut_piece.pattern_direction);
            let stock_piece_area = stock_piece.width * stock_piece.length;
            let cut_piece_area = stock_piece
                .cut_pieces
                .iter()
                .map(|cp| cp.width * cp.length)
                .sum::<usize>();
            let waste_piece_area = stock_piece
                .waste_pieces
                .iter()
                .map(|wp| wp.width * wp.length)
                .sum::<usize>();

            // Make sure the stock piece is big enough for the cut pieces and waste pieces.
            assert!(stock_piece_area >= cut_piece_area + waste_piece_area);
        }

        let rects: Vec<Rect> = stock_piece
            .cut_pieces
            .iter()
            .map(|cp| cp.into())
            .chain(stock_piece.waste_pieces.iter().cloned())
            .collect();

        // Assert that all cut pieces and waste pieces are disjoint.
        for i in (0..rects.len()).rev() {
            for j in (i + 1..rects.len()).rev() {
                assert!(!rects[j].contains(&rects[i]));
                assert!(!rects[i].contains(&rects[j]));
            }
        }
    }
}

#[test]
fn guillotine() {
    let solution = Optimizer::new()
        .add_stock_pieces(STOCK_PIECES.iter().cloned().collect::<Vec<_>>())
        .add_cut_pieces(CUT_PIECES.iter().cloned().collect::<Vec<_>>())
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_guillotine(|_| {})
        .unwrap();

    sanity_check_solution(&solution, CUT_PIECES.len());
}

#[test]
fn guillotine_rotate() {
    let solution = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 10,
            length: 11,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: None,
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: Some(1),
            width: 11,
            length: 10,
            pattern_direction: PatternDirection::None,
            can_rotate: true,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_guillotine(|_| {})
        .unwrap();

    sanity_check_solution(&solution, 1);

    let stock_pieces = solution.stock_pieces;
    assert_eq!(stock_pieces.len(), 1);
    let cut_pieces = &stock_pieces[0].cut_pieces;
    assert_eq!(cut_pieces.len(), 1);
    assert_eq!(
        cut_pieces[0],
        ResultCutPiece {
            external_id: Some(1),
            x: 0,
            y: 0,
            width: 10,
            length: 11,
            pattern_direction: PatternDirection::None,
            is_rotated: true,
        }
    );
}

#[test]
fn guillotine_rotate_pattern() {
    let solution = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 10,
            length: 11,
            pattern_direction: PatternDirection::ParallelToWidth,
            price: 0,
            quantity: None,
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: Some(1),
            width: 11,
            length: 10,
            pattern_direction: PatternDirection::ParallelToLength,
            can_rotate: true,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_guillotine(|_| {})
        .unwrap();

    sanity_check_solution(&solution, 1);

    let stock_pieces = solution.stock_pieces;
    assert_eq!(stock_pieces.len(), 1);
    let cut_pieces = &stock_pieces[0].cut_pieces;
    assert_eq!(cut_pieces.len(), 1);
    assert_eq!(
        cut_pieces[0],
        ResultCutPiece {
            external_id: Some(1),
            x: 0,
            y: 0,
            width: 10,
            length: 11,
            pattern_direction: PatternDirection::ParallelToWidth,
            is_rotated: true,
        }
    );
}

#[test]
fn guillotine_non_fitting_cut_piece_can_rotate() {
    let result = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 10,
            length: 10,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: None,
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: Some(1),
            width: 11,
            length: 10,
            pattern_direction: PatternDirection::None,
            can_rotate: true,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_guillotine(|_| {});

    assert!(
        matches!(result, Err(Error::NoFitForCutPiece(_))),
        "should have returned Error::NoFitForCutPiece"
    )
}

#[test]
fn guillotine_non_fitting_cut_piece_no_rotate() {
    let result = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 10,
            length: 11,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: None,
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: Some(1),
            width: 11,
            length: 10,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_guillotine(|_| {});

    assert!(
        matches!(result, Err(Error::NoFitForCutPiece(_))),
        "should have returned Error::NoFitForCutPiece"
    )
}

#[test]
fn guillotine_non_fitting_cut_piece_no_rotate_pattern() {
    let result = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 10,
            length: 11,
            pattern_direction: PatternDirection::ParallelToWidth,
            price: 0,
            quantity: None,
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: Some(1),
            width: 11,
            length: 10,
            pattern_direction: PatternDirection::ParallelToLength,
            can_rotate: false,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_guillotine(|_| {});

    assert!(
        matches!(result, Err(Error::NoFitForCutPiece(_))),
        "should have returned Error::NoFitForCutPiece"
    )
}

#[test]
fn guillotine_non_fitting_cut_piece_mismatched_pattern() {
    let result = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 100,
            length: 100,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: None,
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: Some(1),
            width: 11,
            length: 10,
            pattern_direction: PatternDirection::ParallelToWidth,
            can_rotate: true,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_guillotine(|_| {});

    assert!(
        matches!(result, Err(Error::NoFitForCutPiece(_))),
        "should have returned Error::NoFitForCutPiece"
    )
}

#[test]
fn guillotine_no_allow_mixed_stock_sizes() {
    let solution = Optimizer::new()
        .add_stock_pieces(STOCK_PIECES.iter().cloned().collect::<Vec<_>>())
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: Some(1),
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: Some(2),
            width: 48,
            length: 120,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .allow_mixed_stock_sizes(false)
        .optimize_guillotine(|_| {})
        .unwrap();

    sanity_check_solution(&solution, 2);

    assert_eq!(solution.stock_pieces.len(), 2);
    for stock_piece in solution.stock_pieces {
        // Since we aren't allowing mixed sizes,
        // all stock pieces will need to be 120 long.
        assert_eq!(stock_piece.length, 120)
    }
}

#[test]
fn guillotine_different_stock_piece_prices() {
    let solution = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            price: 1,
            quantity: None,
        })
        .add_stock_piece(StockPiece {
            width: 48,
            length: 120,
            pattern_direction: PatternDirection::None,
            // Maker the 48x120 stock piece more expensive than (2) 48x96 pieces.
            price: 3,
            quantity: None,
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: Some(1),
            width: 48,
            length: 50,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: Some(2),
            width: 48,
            length: 50,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .allow_mixed_stock_sizes(false)
        .optimize_guillotine(|_| {})
        .unwrap();

    sanity_check_solution(&solution, 2);

    // A single 48x120 stock piece could be used, but since we've set (2) 48x96 pieces to
    // be a lower price than (1) 48x120, it should use (2) 48x96 pieces instead.
    assert_eq!(solution.stock_pieces.len(), 2);
    for stock_piece in solution.stock_pieces {
        assert_eq!(stock_piece.length, 96)
    }
}

#[test]
fn guillotine_same_stock_piece_prices() {
    let solution = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: None,
        })
        .add_stock_piece(StockPiece {
            width: 48,
            length: 120,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: None,
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: Some(1),
            width: 48,
            length: 50,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: Some(2),
            width: 48,
            length: 50,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .allow_mixed_stock_sizes(false)
        .optimize_guillotine(|_| {})
        .unwrap();

    sanity_check_solution(&solution, 2);

    assert_eq!(solution.stock_pieces.len(), 1);
    assert_eq!(solution.stock_pieces[0].length, 120)
}

#[test]
fn guillotine_stock_quantity_too_low() {
    let result = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: Some(1),
        })
        .add_cut_piece(CutPiece {
            quantity: 2,
            external_id: None,
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_guillotine(|_| {});

    assert!(
        result.is_err(),
        "should fail because stock quantity is too low"
    );
}

#[test]
fn guillotine_stock_quantity_1() {
    let solution = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: Some(1),
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: None,
            width: 10,
            length: 96,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_guillotine(|_| {})
        .unwrap();

    sanity_check_solution(&solution, 1);
}

#[test]
fn guillotine_stock_quantity_2() {
    let solution = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: Some(2),
        })
        .add_cut_piece(CutPiece {
            quantity: 2,
            external_id: None,
            width: 10,
            length: 96,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_guillotine(|_| {})
        .unwrap();

    sanity_check_solution(&solution, 2);
}

#[test]
fn guillotine_stock_quantity_multiple() {
    let solution = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: Some(2),
        })
        .add_stock_piece(StockPiece {
            width: 64,
            length: 192,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: Some(1),
        })
        .add_cut_piece(CutPiece {
            quantity: 2,
            external_id: None,
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: None,
            width: 64,
            length: 192,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .set_cut_width(0)
        .set_random_seed(1)
        .optimize_guillotine(|_| {})
        .unwrap();

    sanity_check_solution(&solution, 3);
}

#[test]
fn guillotine_one_stock_piece_several_cut_pieces() {
    let solution = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: Some(1),
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: None,
            width: 8,
            length: 96,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .add_cut_piece(CutPiece {
            quantity: 2,
            external_id: None,
            width: 40,
            length: 10,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .add_cut_piece(CutPiece {
            quantity: 4,
            external_id: None,
            width: 20,
            length: 20,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: None,
            width: 40,
            length: 36,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .set_cut_width(0)
        .set_random_seed(1)
        .optimize_guillotine(|_| {})
        .unwrap();

    sanity_check_solution(&solution, 8);
}

#[test]
fn guillotine_stock_duplicate_cut_piece() {
    let solution = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: Some(1),
        })
        .add_stock_piece(StockPiece {
            width: 64,
            length: 192,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: Some(1),
        })
        .add_cut_piece(CutPiece {
            quantity: 2,
            external_id: None,
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_guillotine(|_| {})
        .unwrap();

    sanity_check_solution(&solution, 2);
}

#[test]
fn guillotine_32_cut_pieces_on_1_stock_piece() {
    let mut optimizer = Optimizer::new();
    optimizer.add_stock_piece(StockPiece {
        width: 48,
        length: 96,
        pattern_direction: PatternDirection::None,
        price: 0,
        quantity: None,
    });

    let num_cut_pieces = 32;

    optimizer.add_cut_piece(CutPiece {
        quantity: num_cut_pieces,
        external_id: Some(1),
        width: 10,
        length: 10,
        pattern_direction: PatternDirection::None,
        can_rotate: false,
    });

    let solution = optimizer
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_guillotine(|_| {})
        .unwrap();

    sanity_check_solution(&solution, num_cut_pieces);

    let stock_pieces = solution.stock_pieces;
    assert_eq!(stock_pieces.len(), 1);
    let cut_pieces = &stock_pieces[0].cut_pieces;
    assert_eq!(cut_pieces.len(), 32);
}

#[test]
fn guillotine_32_cut_pieces_on_2_stock_piece_zero_cut_width() {
    let mut optimizer = Optimizer::new();
    optimizer.add_stock_piece(StockPiece {
        width: 48,
        length: 96,
        pattern_direction: PatternDirection::None,
        price: 0,
        quantity: None,
    });

    let num_cut_pieces = 32;

    optimizer.add_cut_piece(CutPiece {
        quantity: num_cut_pieces,
        external_id: Some(1),
        width: 12,
        length: 12,
        pattern_direction: PatternDirection::None,
        can_rotate: false,
    });

    let solution = optimizer
        .set_cut_width(0)
        .set_random_seed(1)
        .optimize_guillotine(|_| {})
        .unwrap();

    sanity_check_solution(&solution, num_cut_pieces);

    let stock_pieces = solution.stock_pieces;
    assert_eq!(stock_pieces.len(), 1);
    let cut_pieces = &stock_pieces[0].cut_pieces;
    assert_eq!(cut_pieces.len(), 32);
}

#[test]
fn guillotine_32_cut_pieces_on_2_stock_piece() {
    let mut optimizer = Optimizer::new();
    optimizer.add_stock_piece(StockPiece {
        width: 48,
        length: 96,
        pattern_direction: PatternDirection::None,
        price: 0,
        quantity: None,
    });

    let num_cut_pieces = 32;

    optimizer.add_cut_piece(CutPiece {
        quantity: num_cut_pieces,
        external_id: Some(1),
        width: 12,
        length: 12,
        pattern_direction: PatternDirection::None,
        can_rotate: false,
    });

    let solution = optimizer
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_guillotine(|_| {})
        .unwrap();

    sanity_check_solution(&solution, num_cut_pieces);

    let stock_pieces = solution.stock_pieces;
    assert_eq!(stock_pieces.len(), 2);
}

#[test]
fn guillotine_64_cut_pieces_on_2_stock_pieces() {
    let mut optimizer = Optimizer::new();
    optimizer.add_stock_piece(StockPiece {
        width: 48,
        length: 96,
        pattern_direction: PatternDirection::None,
        price: 0,
        quantity: None,
    });

    let num_cut_pieces = 64;

    optimizer.add_cut_piece(CutPiece {
        quantity: num_cut_pieces,
        external_id: Some(1),
        width: 10,
        length: 10,
        pattern_direction: PatternDirection::None,
        can_rotate: false,
    });

    let solution = optimizer
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_guillotine(|_| {})
        .unwrap();

    sanity_check_solution(&solution, num_cut_pieces);

    let stock_pieces = solution.stock_pieces;
    assert_eq!(stock_pieces.len(), 2);
    assert_eq!(stock_pieces[0].cut_pieces.len(), 32);
    assert_eq!(stock_pieces[1].cut_pieces.len(), 32);
}

#[test]
fn guillotine_random_cut_pieces() {
    let mut optimizer = Optimizer::new();
    optimizer.add_stock_piece(StockPiece {
        width: 48,
        length: 96,
        pattern_direction: PatternDirection::ParallelToWidth,
        price: 0,
        quantity: None,
    });
    optimizer.add_stock_piece(StockPiece {
        width: 48,
        length: 96,
        pattern_direction: PatternDirection::ParallelToLength,
        price: 0,
        quantity: None,
    });
    optimizer.add_stock_piece(StockPiece {
        width: 48,
        length: 120,
        pattern_direction: PatternDirection::ParallelToWidth,
        price: 0,
        quantity: None,
    });
    optimizer.add_stock_piece(StockPiece {
        width: 48,
        length: 120,
        pattern_direction: PatternDirection::ParallelToLength,
        price: 0,
        quantity: None,
    });

    let mut rng: StdRng = SeedableRng::seed_from_u64(1);

    let num_cut_pieces = 30;

    optimizer.add_cut_piece(CutPiece {
        quantity: num_cut_pieces,
        external_id: Some(1),
        width: rng.gen_range(1..=48),
        length: rng.gen_range(1..=120),
        pattern_direction: if rng.gen_bool(0.5) {
            PatternDirection::ParallelToWidth
        } else {
            PatternDirection::ParallelToLength
        },
        can_rotate: true,
    });

    let solution = optimizer
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_guillotine(|_| {})
        .unwrap();

    sanity_check_solution(&solution, num_cut_pieces);
}

#[test]
fn nested() {
    let solution = Optimizer::new()
        .add_stock_pieces(STOCK_PIECES.iter().cloned().collect::<Vec<_>>())
        .add_cut_pieces(CUT_PIECES.iter().cloned().collect::<Vec<_>>())
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_nested(|_| {})
        .unwrap();

    sanity_check_solution(&solution, CUT_PIECES.len());

    let stock_pieces = solution.stock_pieces;
    assert_eq!(stock_pieces.len(), 1);
    let cut_pieces = &stock_pieces[0].cut_pieces;
    assert_eq!(cut_pieces.len(), CUT_PIECES.len());
}

#[test]
fn nested_rotate() {
    let solution = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 10,
            length: 11,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: None,
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: Some(1),
            width: 11,
            length: 10,
            pattern_direction: PatternDirection::None,
            can_rotate: true,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_nested(|_| {})
        .unwrap();

    sanity_check_solution(&solution, 1);

    let stock_pieces = solution.stock_pieces;
    assert_eq!(stock_pieces.len(), 1);
    let cut_pieces = &stock_pieces[0].cut_pieces;
    assert_eq!(cut_pieces.len(), 1);
    assert_eq!(
        cut_pieces[0],
        ResultCutPiece {
            external_id: Some(1),
            x: 0,
            y: 0,
            width: 10,
            length: 11,
            pattern_direction: PatternDirection::None,
            is_rotated: true,
        }
    );
}

#[test]
fn nested_rotate_pattern() {
    let solution = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 10,
            length: 11,
            pattern_direction: PatternDirection::ParallelToWidth,
            price: 0,
            quantity: None,
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: Some(1),
            width: 11,
            length: 10,
            pattern_direction: PatternDirection::ParallelToLength,
            can_rotate: true,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_nested(|_| {})
        .unwrap();

    sanity_check_solution(&solution, 1);

    let stock_pieces = solution.stock_pieces;
    assert_eq!(stock_pieces.len(), 1);
    let cut_pieces = &stock_pieces[0].cut_pieces;
    assert_eq!(cut_pieces.len(), 1);
    assert_eq!(
        cut_pieces[0],
        ResultCutPiece {
            external_id: Some(1),
            x: 0,
            y: 0,
            width: 10,
            length: 11,
            pattern_direction: PatternDirection::ParallelToWidth,
            is_rotated: true,
        }
    );
}

#[test]
fn nested_non_fitting_cut_piece_can_rotate() {
    let result = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 10,
            length: 10,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: None,
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: Some(1),
            width: 11,
            length: 10,
            pattern_direction: PatternDirection::None,
            can_rotate: true,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_nested(|_| {});

    assert!(
        matches!(result, Err(Error::NoFitForCutPiece(_))),
        "should have returned Error::NoFitForCutPiece"
    )
}

#[test]
fn nested_non_fitting_cut_piece_no_rotate() {
    let result = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 10,
            length: 11,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: None,
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: Some(1),
            width: 11,
            length: 10,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_nested(|_| {});

    assert!(
        matches!(result, Err(Error::NoFitForCutPiece(_))),
        "should have returned Error::NoFitForCutPiece"
    )
}

#[test]
fn nested_non_fitting_cut_piece_no_rotate_pattern() {
    let result = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 10,
            length: 11,
            pattern_direction: PatternDirection::ParallelToWidth,
            price: 0,
            quantity: None,
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: Some(1),
            width: 11,
            length: 10,
            pattern_direction: PatternDirection::ParallelToLength,
            can_rotate: false,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_nested(|_| {});

    assert!(
        matches!(result, Err(Error::NoFitForCutPiece(_))),
        "should have returned Error::NoFitForCutPiece"
    )
}

#[test]
fn nested_non_fitting_cut_piece_mismatched_pattern() {
    let result = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 100,
            length: 100,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: None,
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: Some(1),
            width: 11,
            length: 10,
            pattern_direction: PatternDirection::ParallelToWidth,
            can_rotate: true,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_nested(|_| {});

    assert!(
        matches!(result, Err(Error::NoFitForCutPiece(_))),
        "should have returned Error::NoFitForCutPiece"
    )
}

#[test]
fn nested_no_allow_mixed_stock_sizes() {
    let solution = Optimizer::new()
        .add_stock_pieces(STOCK_PIECES.iter().cloned().collect::<Vec<_>>())
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: Some(1),
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: Some(2),
            width: 48,
            length: 120,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .allow_mixed_stock_sizes(false)
        .optimize_nested(|_| {})
        .unwrap();

    sanity_check_solution(&solution, 2);

    assert_eq!(solution.stock_pieces.len(), 2);
    for stock_piece in solution.stock_pieces {
        // Since we aren't allowing mixed sizes,
        // all stock pieces will need to be 120 long.
        assert_eq!(stock_piece.length, 120)
    }
}

#[test]
fn nested_different_stock_piece_prices() {
    let solution = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            price: 1,
            quantity: None,
        })
        .add_stock_piece(StockPiece {
            width: 48,
            length: 120,
            pattern_direction: PatternDirection::None,
            // Maker the 48x120 stock piece more expensive than (2) 48x96 pieces.
            price: 3,
            quantity: None,
        })
        .add_cut_piece(CutPiece {
            quantity: 2,
            external_id: Some(1),
            width: 48,
            length: 50,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .allow_mixed_stock_sizes(false)
        .optimize_nested(|_| {})
        .unwrap();

    sanity_check_solution(&solution, 2);

    // A single 48x120 stock piece could be used, but since we've set (2) 48x96 pieces to
    // be a lower price than (1) 48x120, it should use (2) 48x96 pieces instead.
    assert_eq!(solution.stock_pieces.len(), 2);
    for stock_piece in solution.stock_pieces {
        assert_eq!(stock_piece.length, 96)
    }
}

#[test]
fn nested_same_stock_piece_prices() {
    let solution = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: None,
        })
        .add_stock_piece(StockPiece {
            width: 48,
            length: 120,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: None,
        })
        .add_cut_piece(CutPiece {
            quantity: 2,
            external_id: Some(1),
            width: 48,
            length: 50,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .allow_mixed_stock_sizes(false)
        .optimize_nested(|_| {})
        .unwrap();

    sanity_check_solution(&solution, 2);

    assert_eq!(solution.stock_pieces.len(), 1);
    assert_eq!(solution.stock_pieces[0].length, 120)
}

#[test]
fn nested_stock_quantity_too_low() {
    let result = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: Some(1),
        })
        .add_cut_piece(CutPiece {
            quantity: 2,
            external_id: None,
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_nested(|_| {});

    assert!(
        result.is_err(),
        "should fail because stock quantity is too low"
    );
}

#[test]
fn nested_stock_quantity_1() {
    let solution = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: Some(1),
        })
        .add_cut_piece(CutPiece {
            quantity: 1,
            external_id: None,
            width: 10,
            length: 96,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_nested(|_| {})
        .unwrap();

    sanity_check_solution(&solution, 1);
}

#[test]
fn nested_stock_quantity_2() {
    let solution = Optimizer::new()
        .add_stock_piece(StockPiece {
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: Some(2),
        })
        .add_cut_piece(CutPiece {
            quantity: 2,
            external_id: None,
            width: 10,
            length: 96,
            pattern_direction: PatternDirection::None,
            can_rotate: false,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_nested(|_| {})
        .unwrap();

    sanity_check_solution(&solution, 2);
}

#[test]
fn nested_32_cut_pieces_on_1_stock_piece() {
    let mut optimizer = Optimizer::new();
    optimizer.add_stock_piece(StockPiece {
        width: 48,
        length: 96,
        pattern_direction: PatternDirection::None,
        price: 0,
        quantity: None,
    });

    let num_cut_pieces = 32;

    optimizer.add_cut_piece(CutPiece {
        quantity: num_cut_pieces,
        external_id: Some(1),
        width: 10,
        length: 10,
        pattern_direction: PatternDirection::None,
        can_rotate: false,
    });

    let solution = optimizer
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_nested(|_| {})
        .unwrap();

    sanity_check_solution(&solution, num_cut_pieces);

    let stock_pieces = solution.stock_pieces;
    assert_eq!(stock_pieces.len(), 1);
    let cut_pieces = &stock_pieces[0].cut_pieces;
    assert_eq!(cut_pieces.len(), 32);
}

#[test]
fn nested_32_cut_pieces_on_2_stock_piece_zero_cut_width() {
    let mut optimizer = Optimizer::new();
    optimizer.add_stock_piece(StockPiece {
        width: 48,
        length: 96,
        pattern_direction: PatternDirection::None,
        price: 0,
        quantity: None,
    });

    let num_cut_pieces = 32;

    optimizer.add_cut_piece(CutPiece {
        quantity: num_cut_pieces,
        external_id: Some(1),
        width: 12,
        length: 12,
        pattern_direction: PatternDirection::None,
        can_rotate: false,
    });

    let solution = optimizer
        .set_cut_width(0)
        .set_random_seed(1)
        .optimize_nested(|_| {})
        .unwrap();

    sanity_check_solution(&solution, num_cut_pieces);

    let stock_pieces = solution.stock_pieces;
    assert_eq!(stock_pieces.len(), 1);
    let cut_pieces = &stock_pieces[0].cut_pieces;
    assert_eq!(cut_pieces.len(), 32);
}

#[test]
fn nested_32_cut_pieces_on_2_stock_piece() {
    let mut optimizer = Optimizer::new();
    optimizer.add_stock_piece(StockPiece {
        width: 48,
        length: 96,
        pattern_direction: PatternDirection::None,
        price: 0,
        quantity: None,
    });

    let num_cut_pieces = 32;

    optimizer.add_cut_piece(CutPiece {
        quantity: num_cut_pieces,
        external_id: Some(1),
        width: 12,
        length: 12,
        pattern_direction: PatternDirection::None,
        can_rotate: false,
    });

    let solution = optimizer
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_nested(|_| {})
        .unwrap();

    sanity_check_solution(&solution, num_cut_pieces);

    let stock_pieces = solution.stock_pieces;
    assert_eq!(stock_pieces.len(), 2);
}

#[test]
fn nested_64_cut_pieces_on_2_stock_pieces() {
    let mut optimizer = Optimizer::new();
    optimizer.add_stock_piece(StockPiece {
        width: 48,
        length: 96,
        pattern_direction: PatternDirection::None,
        price: 0,
        quantity: None,
    });

    let num_cut_pieces = 64;

    optimizer.add_cut_piece(CutPiece {
        quantity: num_cut_pieces,
        external_id: Some(1),
        width: 10,
        length: 10,
        pattern_direction: PatternDirection::None,
        can_rotate: false,
    });

    let solution = optimizer
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_nested(|_| {})
        .unwrap();

    sanity_check_solution(&solution, num_cut_pieces);

    let stock_pieces = solution.stock_pieces;
    assert_eq!(stock_pieces.len(), 2);
    assert_eq!(stock_pieces[0].cut_pieces.len(), 32);
    assert_eq!(stock_pieces[1].cut_pieces.len(), 32);
}

#[test]
fn nested_random_cut_pieces() {
    let mut optimizer = Optimizer::new();
    optimizer.add_stock_piece(StockPiece {
        width: 48,
        length: 96,
        pattern_direction: PatternDirection::ParallelToWidth,
        price: 0,
        quantity: None,
    });
    optimizer.add_stock_piece(StockPiece {
        width: 48,
        length: 96,
        pattern_direction: PatternDirection::ParallelToLength,
        price: 0,
        quantity: None,
    });
    optimizer.add_stock_piece(StockPiece {
        width: 48,
        length: 120,
        pattern_direction: PatternDirection::ParallelToWidth,
        price: 0,
        quantity: None,
    });
    optimizer.add_stock_piece(StockPiece {
        width: 48,
        length: 120,
        pattern_direction: PatternDirection::ParallelToLength,
        price: 0,
        quantity: None,
    });

    let mut rng: StdRng = SeedableRng::seed_from_u64(1);

    let num_cut_pieces = 30;

    optimizer.add_cut_piece(CutPiece {
        quantity: num_cut_pieces,
        external_id: Some(1),
        width: rng.gen_range(1..=48),
        length: rng.gen_range(1..=120),
        pattern_direction: if rng.gen_bool(0.5) {
            PatternDirection::ParallelToWidth
        } else {
            PatternDirection::ParallelToLength
        },
        can_rotate: true,
    });

    let solution = optimizer
        .set_cut_width(1)
        .set_random_seed(1)
        .optimize_nested(|_| {})
        .unwrap();

    sanity_check_solution(&solution, num_cut_pieces);
}

#[test]
fn add_equivalent_stock_pieces_sums_quantities() {
    let mut optimizer = Optimizer::new();
    optimizer
        .add_stock_piece(StockPiece {
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: Some(3),
        })
        .add_stock_piece(StockPiece {
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: Some(6),
        });

    assert_eq!(optimizer.stock_pieces.len(), 1);
    assert_eq!(optimizer.stock_pieces[0].quantity, Some(9));
}

#[test]
fn add_equivalent_stock_pieces_with_none() {
    let mut optimizer = Optimizer::new();
    optimizer
        .add_stock_piece(StockPiece {
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: None,
        })
        .add_stock_piece(StockPiece {
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: Some(6),
        });

    assert_eq!(optimizer.stock_pieces.len(), 1);
    assert_eq!(optimizer.stock_pieces[0].quantity, None);
}

#[test]
fn stock_pieces_dec_quantity() {
    let mut stock_piece = StockPiece {
        width: 48,
        length: 96,
        pattern_direction: PatternDirection::None,
        price: 0,
        quantity: Some(10),
    };

    stock_piece.dec_quantity();

    assert_eq!(stock_piece.quantity, Some(9));

    stock_piece.quantity = None;
    stock_piece.dec_quantity();

    assert_eq!(stock_piece.quantity, None);
}

#[test]
fn guillotine_rotate_cut_pieces() {
    let mut optimizer = Optimizer::new();
    optimizer
        .add_stock_piece(StockPiece {
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: None,
        })
        .add_stock_piece(StockPiece {
            width: 48,
            length: 120,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: None,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .allow_mixed_stock_sizes(false);

    optimizer.add_cut_piece(CutPiece {
        quantity: 16,
        external_id: Some(1),
        width: 18,
        length: 24,
        pattern_direction: PatternDirection::None,
        can_rotate: true,
    });

    let result = optimizer.optimize_guillotine(|_| {});

    assert!(result.is_ok());
    if let Ok(solution) = result {
        assert_eq!(solution.stock_pieces.len(), 2);
        assert_eq!(solution.stock_pieces[0].length, 96);
    }
}

#[test]
fn nested_rotate_cut_pieces() {
    let mut optimizer = Optimizer::new();
    optimizer
        .add_stock_piece(StockPiece {
            width: 48,
            length: 96,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: None,
        })
        .add_stock_piece(StockPiece {
            width: 48,
            length: 120,
            pattern_direction: PatternDirection::None,
            price: 0,
            quantity: None,
        })
        .set_cut_width(1)
        .set_random_seed(1)
        .allow_mixed_stock_sizes(false);

    optimizer.add_cut_piece(CutPiece {
        quantity: 16,
        external_id: Some(1),
        width: 18,
        length: 24,
        pattern_direction: PatternDirection::None,
        can_rotate: true,
    });

    let result = optimizer.optimize_guillotine(|_| {});

    assert!(result.is_ok());
    if let Ok(solution) = result {
        assert_eq!(solution.stock_pieces.len(), 2);
        assert_eq!(solution.stock_pieces[0].length, 96);
    }
}

#[test]
fn pighetti_github_issue_12() {
    let mut optimizer = Optimizer::new();

    let plywood = StockPiece {
        quantity: Some(1),
        length: 2440,
        width: 1220,
        pattern_direction: PatternDirection::ParallelToLength,
        price: 130,
    };

    let cut_piece_a = CutPiece {
        quantity: 12,
        external_id: Some(1),
        length: 775,
        width: 150,
        can_rotate: false,
        pattern_direction: PatternDirection::ParallelToLength,
    };

    let cut_piece_b = CutPiece {
        quantity: 25,
        external_id: Some(1),
        length: 450,
        width: 100,
        can_rotate: false,
        pattern_direction: PatternDirection::ParallelToLength,
    };

    optimizer.add_stock_piece(plywood);
    optimizer.add_cut_piece(cut_piece_a);
    optimizer.add_cut_piece(cut_piece_b);
    optimizer.set_cut_width(2);
    optimizer.set_random_seed(1);

    let result = optimizer.optimize_guillotine(|_| {});

    assert!(result.is_ok());
    if let Ok(solution) = result {
        assert_eq!(solution.stock_pieces.len(), 1);
        sanity_check_solution(&solution, 37);
    }
}

#[test]
fn pighetti_github_issue_16() {
    let plywood = StockPiece {
        quantity: Some(2),
        length: 2440,
        width: 1220,
        pattern_direction: PatternDirection::ParallelToLength,
        price: 130,
    };

    let cut_piece_a = CutPiece {
        quantity: 6,
        external_id: Some(1),
        length: 814,
        width: 465,
        can_rotate: false,
        pattern_direction: PatternDirection::ParallelToLength,
    };

    let mut optimizer = Optimizer::new();
    optimizer.add_stock_piece(plywood);
    optimizer.add_cut_piece(cut_piece_a);
    optimizer.set_cut_width(2);
    optimizer.set_random_seed(1);

    let result = optimizer.optimize_guillotine(|_| {});

    assert!(result.is_ok());
    if let Ok(solution) = result {
        assert_eq!(solution.stock_pieces.len(), 2);
        sanity_check_solution(&solution, 6);
    }
}
