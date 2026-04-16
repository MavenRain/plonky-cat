use plonky_cat_field::{BabyBear, Field};
use plonky_cat_hash::AlgebraicHash;
use plonky_cat_plonk::{AddGate, PlonkTrace, constraint_poly};
use plonky_cat_fri::{FriClaim, FriWitness};
use plonky_cat_sumcheck::{SumcheckClaim, SumcheckWitness};
use plonky_cat_basefold::{BaseFold, BaseFoldShared, BaseFoldWitness};
use plonky_cat_sumcheck::LinearWitness;
use plonky_cat_transcript::{AlgebraicTranscript, Transcript};
use plonky_cat_prover::prove;
use plonky_cat_verifier::{verify, Verdict};

type H = AlgebraicHash<BabyBear>;
type F = BabyBear;

fn main() -> Result<(), String> {
    println!("=== fib-plonk-basefold: the v0.1 success criterion ===");
    println!("  PLONK (AddGate custom gate) + BaseFold (Interleave<FRI, Sumcheck>)");
    println!();

    // Fibonacci trace: 4 rows proving fib(0)+fib(1)=fib(2) through fib(3)+fib(4)=fib(5)
    //   row 0: a=0, b=1, c=1   (fib(0) + fib(1) = fib(2))
    //   row 1: a=1, b=1, c=2   (fib(1) + fib(2) = fib(3))
    //   row 2: a=1, b=2, c=3   (fib(2) + fib(3) = fib(4))
    //   row 3: a=2, b=3, c=5   (fib(3) + fib(4) = fib(5))
    let col_a: Vec<F> = [0, 1, 1, 2].iter().map(|v| F::from(*v as u32)).collect();
    let col_b: Vec<F> = [1, 1, 2, 3].iter().map(|v| F::from(*v as u32)).collect();
    let col_c: Vec<F> = [1, 2, 3, 5].iter().map(|v| F::from(*v as u32)).collect();

    let trace = PlonkTrace::new(vec![col_a, col_b, col_c])
        .map_err(|e| format!("trace: {e}"))?;

    println!("trace: {} rows, {} wires (a, b, c)", trace.num_rows(), trace.num_columns());
    println!("gate: AddGate (a + b - c = 0)");

    // Compute the constraint polynomial.  If the Fibonacci relation holds,
    // every evaluation is zero and the sum over the hypercube is zero.
    let constraint = constraint_poly::<AddGate, F>(&trace)
        .map_err(|e| format!("constraint: {e}"))?;

    let sum = constraint.sum_over_hypercube();
    println!("constraint sum over hypercube: {sum}");

    if sum != F::zero() {
        println!("FAIL: constraint is not satisfied");
        Err("constraint not satisfied".to_string())
    } else {
        println!("constraint satisfied (sum = 0)");
        println!();

        // Build BaseFold proof for "sum of constraint poly = 0"
        let evals = constraint.evals().to_vec();
        let fri_witness = FriWitness::<H>::build(evals)
            .map_err(|e| format!("fri witness: {e}"))?;
        let merkle_root = fri_witness.merkle_root()
            .map_err(|e| format!("merkle root: {e}"))?;

        let fri_claim = FriClaim::until_constant(merkle_root, 4);
        let sum_claim = SumcheckClaim::new(sum, 2);

        let shared = BaseFoldShared::new(fri_claim, sum_claim);
        let witness = BaseFoldWitness::new(fri_witness, SumcheckWitness::new(constraint));

        println!("--- proving with BaseFold = Interleave<FRI, Sumcheck> ---");
        let transcript = AlgebraicTranscript::<F>::new();

        let (proof, _) = prove::<BaseFold<H, LinearWitness<F>>, _>(shared.clone(), witness, transcript)
            .map_err(|e| format!("prove: {e:?}"))?;

        println!("proof: {} round messages", proof.messages().len());

        println!();
        println!("--- verifying ---");
        let v_transcript = AlgebraicTranscript::<F>::new();

        let (verdict, _) = verify::<BaseFold<H, LinearWitness<F>>, _>(shared, proof.messages(), v_transcript)
            .map_err(|e| format!("verify: {e:?}"))?;

        match verdict {
            Verdict::Accept(opening) => {
                println!("verifier ACCEPTED");
                if opening == *proof.opening() {
                    println!();
                    println!("PASS: fib-plonk-basefold end-to-end");
                    println!("  Fibonacci trace verified via:");
                    println!("    PLONK AddGate -> constraint polynomial");
                    println!("    BaseFold = Interleave<FRI, Sumcheck>");
                    println!("    zero handwritten protocol code");
                    Ok(())
                } else {
                    println!("FAIL: opening mismatch");
                    Err("opening mismatch".to_string())
                }
            }
        }
    }
}
