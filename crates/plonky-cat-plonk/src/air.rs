use plonky_cat_field::Field;
use plonky_cat_poly::MultilinearPoly;
use crate::error::Error;

/// Algebraic Intermediate Representation: a trace table where each row
/// satisfies polynomial constraints involving the current row and
/// optionally the next row.
///
/// An AIR constraint is a polynomial C(curr_row, next_row) that must
/// vanish for every consecutive pair of rows in the trace.
///
/// For the boundary: first-row and last-row constraints are separate.

/// A transition constraint evaluates over (current_row, next_row).
pub trait TransitionConstraint<F: Field> {
    fn evaluate(&self, curr: &[F], next: &[F]) -> F;
    fn degree(&self) -> usize;
}

/// Simple addition transition: curr[0] + curr[1] - next[0] = 0.
/// Models Fibonacci: fib(i) + fib(i+1) = fib(i+2).
pub struct FibTransition;

impl<F: Field> TransitionConstraint<F> for FibTransition {
    fn evaluate(&self, curr: &[F], next: &[F]) -> F {
        curr[0] + curr[1] - next[0]
    }
    fn degree(&self) -> usize { 1 }
}

/// Multiplication transition: curr[0] * curr[1] - next[0] = 0.
pub struct MulTransition;

impl<F: Field> TransitionConstraint<F> for MulTransition {
    fn evaluate(&self, curr: &[F], next: &[F]) -> F {
        curr[0] * curr[1] - next[0]
    }
    fn degree(&self) -> usize { 2 }
}

/// AIR trace: a table of field elements with `width` columns and `num_rows` rows.
/// num_rows must be a power of two.
#[derive(Debug, Clone)]
pub struct AirTrace<F> {
    width: usize,
    num_rows_log2: usize,
    data: Vec<F>,
}

impl<F: Field> AirTrace<F> {
    pub fn new(width: usize, data: Vec<F>) -> Result<Self, Error> {
        if data.is_empty() || width == 0 {
            Err(Error::EmptyTrace)
        } else if data.len() % width != 0 {
            Err(Error::ColumnLengthMismatch {
                expected: width,
                got: data.len() % width,
            })
        } else {
            let num_rows = data.len() / width;
            if !num_rows.is_power_of_two() {
                Err(Error::RowCountNotPowerOfTwo { len: num_rows })
            } else {
                num_rows.trailing_zeros()
                    .try_into()
                    .map_err(|_| Error::EmptyTrace)
                    .map(|log2| Self { width, num_rows_log2: log2, data })
            }
        }
    }

    #[must_use]
    pub fn num_rows(&self) -> usize { 1 << self.num_rows_log2 }

    #[must_use]
    pub fn width(&self) -> usize { self.width }

    #[must_use]
    pub fn row(&self, i: usize) -> &[F] {
        &self.data[i * self.width..(i + 1) * self.width]
    }

    /// Evaluate a transition constraint at every consecutive row pair
    /// and produce the constraint polynomial in evaluation form.
    /// The result has `num_rows - 1` evaluations, zero-padded to a power of two.
    pub fn transition_poly<C: TransitionConstraint<F>>(
        &self,
        constraint: &C,
    ) -> Result<MultilinearPoly<F>, Error> {
        let n = self.num_rows();
        let evals: Vec<F> = (0..n)
            .map(|i| {
                let next_i = (i + 1) % n;
                constraint.evaluate(self.row(i), self.row(next_i))
            })
            .collect();

        MultilinearPoly::from_evals(self.num_rows_log2, evals).map_err(Error::from)
    }
}
