use plonky_cat_field::BabyBear;
use plonky_cat_poly::MultilinearPoly;
use plonky_cat_sumcheck::{Sumcheck, SumcheckClaim, ProductWitness};
use plonky_cat_transcript::{AlgebraicTranscript, Transcript};
use plonky_cat_prover::prove;
use plonky_cat_verifier::{verify, Verdict};

type F = BabyBear;

fn main() -> Result<(), String> {
    println!("=== degree-2 sumcheck: product of two multilinears ===");
    println!();

    // Two 2-variable multilinears:
    //   a(x0, x1):  evals = [1, 2, 3, 4]
    //   b(x0, x1):  evals = [5, 6, 7, 8]
    //
    // Product a*b evaluated at {0,1}^2:
    //   (0,0): 1*5 = 5
    //   (1,0): 2*6 = 12
    //   (0,1): 3*7 = 21
    //   (1,1): 4*8 = 32
    // Sum = 5 + 12 + 21 + 32 = 70

    let a = MultilinearPoly::from_evals(2, vec![
        F::from(1u32), F::from(2u32), F::from(3u32), F::from(4u32),
    ]).map_err(|e| format!("a: {e}"))?;

    let b = MultilinearPoly::from_evals(2, vec![
        F::from(5u32), F::from(6u32), F::from(7u32), F::from(8u32),
    ]).map_err(|e| format!("b: {e}"))?;

    let expected_sum = F::from(70u32);
    println!("a(x) evals: [1, 2, 3, 4]");
    println!("b(x) evals: [5, 6, 7, 8]");
    println!("expected sum of a*b over hypercube: {expected_sum}");

    let witness = ProductWitness::new(a, b);
    let claim = SumcheckClaim::new(expected_sum, 2);

    println!();
    println!("--- proving (degree-2 round polynomials) ---");
    let transcript = AlgebraicTranscript::<F>::new();

    let (proof, _) = prove::<Sumcheck<ProductWitness<F>>, _>(claim.clone(), witness, transcript)
        .map_err(|e| format!("prove: {e:?}"))?;

    println!("proof: {} round messages", proof.messages().len());
    proof.messages().iter().enumerate().for_each(|(i, msg)| {
        println!("  round {}: {} evals (degree {})", i, msg.evals().len(), msg.degree());
    });
    println!("opening: {}", proof.opening().value());

    println!();
    println!("--- verifying ---");
    let v_transcript = AlgebraicTranscript::<F>::new();

    let (verdict, _) = verify::<Sumcheck<ProductWitness<F>>, _>(claim, proof.messages(), v_transcript)
        .map_err(|e| format!("verify: {e:?}"))?;

    match verdict {
        Verdict::Accept(opening) => {
            println!("verifier ACCEPTED, opening: {}", opening.value());
            if opening == *proof.opening() {
                println!();
                println!("PASS: degree-2 sumcheck (MulGate-compatible)");
            } else {
                println!("FAIL: opening mismatch");
            }
        }
    }

    Ok(())
}
