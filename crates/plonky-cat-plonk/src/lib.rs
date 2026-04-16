#![forbid(unsafe_code)]

mod error;
pub use self::error::Error;

use plonky_cat_field::Field;
use plonky_cat_poly::MultilinearPoly;

/// A custom gate: evaluates a constraint over a row's wire values.
/// Produces zero when the constraint is satisfied.
/// Type-level (no self), matching the ReductionFunctor convention.
pub trait Gate<F: Field> {
    fn evaluate_row(wires: &[F]) -> F;
    fn num_wires() -> usize;
}

/// Addition gate: a + b - c = 0.
pub struct AddGate;

impl<F: Field> Gate<F> for AddGate {
    fn evaluate_row(wires: &[F]) -> F {
        wires[0] + wires[1] - wires[2]
    }
    fn num_wires() -> usize { 3 }
}

/// Multiplication gate: a * b - c = 0.
/// Produces degree-2 constraint; requires extended sumcheck (v0.2).
pub struct MulGate;

impl<F: Field> Gate<F> for MulGate {
    fn evaluate_row(wires: &[F]) -> F {
        wires[0] * wires[1] - wires[2]
    }
    fn num_wires() -> usize { 3 }
}

/// Boolean gate: a * (1 - a) = 0 (constrains wire to {0, 1}).
pub struct BoolGate;

impl<F: Field> Gate<F> for BoolGate {
    fn evaluate_row(wires: &[F]) -> F {
        wires[0] * (F::one() - wires[0])
    }
    fn num_wires() -> usize { 1 }
}

/// Circuit trace: wire assignments arranged as columns.
/// `columns[wire_idx][row_idx]` gives the value of wire `wire_idx` at row `row_idx`.
/// Row count must be a power of two.
#[derive(Debug, Clone)]
pub struct PlonkTrace<F> {
    num_rows_log2: usize,
    columns: Vec<Vec<F>>,
}

impl<F: Field> PlonkTrace<F> {
    pub fn new(columns: Vec<Vec<F>>) -> Result<Self, Error> {
        let num_rows = columns.first().map_or(0, Vec::len);

        match () {
            () if num_rows == 0 => Err(Error::EmptyTrace),
            () if !num_rows.is_power_of_two() =>
                Err(Error::RowCountNotPowerOfTwo { len: num_rows }),
            () => {
                columns.iter().skip(1).try_fold((), |(), col| {
                    if col.len() == num_rows {
                        Ok(())
                    } else {
                        Err(Error::ColumnLengthMismatch {
                            expected: num_rows,
                            got: col.len(),
                        })
                    }
                })?;

                let num_rows_log2 = num_rows.trailing_zeros();

                num_rows_log2.try_into()
                    .map_err(|_| Error::EmptyTrace)
                    .map(|log2| Self { num_rows_log2: log2, columns })
            }
        }
    }

    #[must_use]
    pub fn num_rows(&self) -> usize {
        1 << self.num_rows_log2
    }

    #[must_use]
    pub fn num_rows_log2(&self) -> usize {
        self.num_rows_log2
    }

    #[must_use]
    pub fn num_columns(&self) -> usize {
        self.columns.len()
    }

    #[must_use]
    pub fn columns(&self) -> &[Vec<F>] {
        &self.columns
    }
}

/// Evaluate the gate constraint at every row and produce the constraint polynomial
/// in evaluation form.  If the circuit is satisfied, all evals are zero.
pub fn constraint_poly<G, F>(trace: &PlonkTrace<F>) -> Result<MultilinearPoly<F>, Error>
where
    G: Gate<F>,
    F: Field,
{
    if trace.num_columns() < G::num_wires() {
        Err(Error::InsufficientWires {
            gate_needs: G::num_wires(),
            trace_has: trace.num_columns(),
        })
    } else {
        let evals: Vec<F> = (0..trace.num_rows())
            .map(|row| {
                let wires: Vec<F> = trace.columns().iter()
                    .take(G::num_wires())
                    .map(|col| col[row])
                    .collect();
                G::evaluate_row(&wires)
            })
            .collect();

        MultilinearPoly::from_evals(trace.num_rows_log2(), evals).map_err(Error::from)
    }
}
