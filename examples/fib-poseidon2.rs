use plonky_cat_field::{BabyBear, Field};
use plonky_cat_hash::Poseidon2;
use plonky_cat_plonk::{AddGate, PlonkTrace, constraint_poly};
use plonky_cat_fri::{FriClaim, FriWitness};
use plonky_cat_sumcheck::{SumcheckClaim, LinearWitness};
use plonky_cat_basefold::{BaseFold, BaseFoldShared, BaseFoldWitness};
use plonky_cat_transcript::{AlgebraicTranscript, Transcript};
use plonky_cat_prover::prove;
use plonky_cat_verifier::{verify, Verdict};

type F = BabyBear;

fn main() -> Result<(), String> {
    println!("=== fib-plonk-basefold with Poseidon2 ===");
    println!();

    let col_a: Vec<F> = [0, 1, 1, 2].iter().map(|v| F::from(*v as u32)).collect();
    let col_b: Vec<F> = [1, 1, 2, 3].iter().map(|v| F::from(*v as u32)).collect();
    let col_c: Vec<F> = [1, 2, 3, 5].iter().map(|v| F::from(*v as u32)).collect();

    let trace = PlonkTrace::new(vec![col_a, col_b, col_c])
        .map_err(|e| format!("trace: {e}"))?;

    let constraint = constraint_poly::<AddGate, F>(&trace)
        .map_err(|e| format!("constraint: {e}"))?;

    let sum = constraint.sum_over_hypercube();
    println!("constraint sum: {sum} (should be 0)");

    if sum != F::zero() {
        Err("constraint not satisfied".to_string())
    } else {
        let evals = constraint.evals().to_vec();
        let fri_witness = FriWitness::<Poseidon2>::build(evals)
            .map_err(|e| format!("fri: {e}"))?;
        let root = fri_witness.merkle_root().map_err(|e| format!("root: {e}"))?;

        let shared = BaseFoldShared::new(
            FriClaim::until_constant(root, 4),
            SumcheckClaim::new(sum, 2),
        );
        let witness = BaseFoldWitness::new(
            fri_witness,
            LinearWitness::new(constraint),
        );

        println!("hash: Poseidon2 (BabyBear, x^7, 8+22 rounds)");
        println!("merkle root: {root}");
        println!();

        println!("--- proving ---");
        let (proof, _) = prove::<BaseFold<Poseidon2, LinearWitness<F>>, _>(
            shared.clone(), witness, AlgebraicTranscript::<F>::new(),
        ).map_err(|e| format!("prove: {e:?}"))?;

        println!("proof: {} round messages", proof.messages().len());

        println!();
        println!("--- verifying ---");
        let (verdict, _) = verify::<BaseFold<Poseidon2, LinearWitness<F>>, _>(
            shared, proof.messages(), AlgebraicTranscript::<F>::new(),
        ).map_err(|e| format!("verify: {e:?}"))?;

        match verdict {
            Verdict::Accept(opening) => {
                if opening == *proof.opening() {
                    println!("PASS: fib-plonk-basefold with Poseidon2");
                } else {
                    println!("FAIL: opening mismatch");
                }
            }
        }

        Ok(())
    }
}
