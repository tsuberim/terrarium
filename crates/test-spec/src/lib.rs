//! Declarative Creature Studio test DSL — parse, validate, evaluate.

mod parse;

pub use parse::{
    evaluate_assertions, parse_tests, Assertion, AssertionOp, AssertionResult, Diagnostic,
    FrameSnapshot, ParsedTests, TestSpec, TileKind, TilePlacement,
};
