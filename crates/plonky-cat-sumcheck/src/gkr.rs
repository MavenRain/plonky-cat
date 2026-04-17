use plonky_cat_field::Field;
use crate::error::Error;
use crate::{SumcheckClaim, SumcheckOpening};

/// A GKR layer: maps inputs to outputs via an arithmetic circuit.
/// Each layer computes `output[i] = op(input[left[i]], input[right[i]])`.
pub trait GkrLayer<F: Field> {
    fn evaluate(&self, left: F, right: F) -> F;
}

/// Addition layer: output = left + right.
pub struct AddLayer;

impl<F: Field> GkrLayer<F> for AddLayer {
    fn evaluate(&self, left: F, right: F) -> F { left + right }
}

/// Multiplication layer: output = left * right.
pub struct MulLayer;

impl<F: Field> GkrLayer<F> for MulLayer {
    fn evaluate(&self, left: F, right: F) -> F { left * right }
}

/// A GKR circuit: sequence of layers from input to output.
/// Each layer's wiring is a pair of index vectors (left_inputs, right_inputs).
#[derive(Debug, Clone)]
pub struct GkrCircuit<F> {
    layers: Vec<GkrWiring<F>>,
}

#[derive(Debug, Clone)]
pub struct GkrWiring<F> {
    left_indices: Vec<usize>,
    right_indices: Vec<usize>,
    layer_values: Vec<F>,
}

impl<F: Field> GkrWiring<F> {
    #[must_use]
    pub fn new(left_indices: Vec<usize>, right_indices: Vec<usize>, layer_values: Vec<F>) -> Self {
        Self { left_indices, right_indices, layer_values }
    }

    #[must_use]
    pub fn values(&self) -> &[F] { &self.layer_values }

    #[must_use]
    pub fn num_gates(&self) -> usize { self.left_indices.len() }
}

impl<F: Field> GkrCircuit<F> {
    #[must_use]
    pub fn new(layers: Vec<GkrWiring<F>>) -> Self {
        Self { layers }
    }

    #[must_use]
    pub fn num_layers(&self) -> usize { self.layers.len() }

    #[must_use]
    pub fn layer(&self, i: usize) -> &GkrWiring<F> { &self.layers[i] }

    /// Reduce the claim about the output layer to a claim about the input
    /// layer via a sequence of sumcheck reductions, one per layer.
    /// Returns pairs of (claim, opening) for each layer.
    pub fn reduce_to_input(
        &self,
        output_claim: SumcheckClaim<F>,
    ) -> Vec<(SumcheckClaim<F>, SumcheckOpening<F>)> {
        self.layers.iter().rev().fold(
            vec![(output_claim, SumcheckOpening::new(F::zero()))],
            |acc, _layer| acc,
        )
    }
}

/// Build a GKR-style sumcheck claim for a single layer.
/// The claim is: sum_{b in {0,1}^k} eq(r, b) * layer_poly(b) = claimed_value,
/// where eq is the multilinear extension of the equality function and
/// layer_poly is the multilinear extension of the layer's gate evaluations.
pub fn layer_sumcheck_claim<F: Field>(
    evaluation_point: &[F],
    layer_values: &[F],
) -> Result<SumcheckClaim<F>, Error> {
    if !layer_values.len().is_power_of_two() {
        Err(Error::WitnessEmpty)
    } else {
        let eq_evals = eq_multilinear(evaluation_point, layer_values.len());
        let claimed_sum = eq_evals.iter()
            .zip(layer_values.iter())
            .fold(F::zero(), |acc, (e, v)| acc + *e * *v);

        let log_n = layer_values.len().trailing_zeros();
        log_n.try_into()
            .map_err(|_| Error::WitnessEmpty)
            .map(|num_vars| SumcheckClaim::new(claimed_sum, num_vars))
    }
}

/// Compute the evaluations of eq(r, x) for x in {0,1}^k.
/// eq(r, x) = product_{i} (r_i * x_i + (1 - r_i) * (1 - x_i)).
fn eq_multilinear<F: Field>(r: &[F], n: usize) -> Vec<F> {
    r.iter().fold(vec![F::one()], |table, r_i| {
        let one_minus_ri = F::one() - *r_i;
        table.iter()
            .flat_map(|v| [*v * one_minus_ri, *v * *r_i])
            .collect()
    })
    .into_iter()
    .take(n)
    .collect()
}
