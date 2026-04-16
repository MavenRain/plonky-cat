use plonky_cat_field::BabyBear;
use plonky_cat_transcript::Transcript;
use plonky_cat_poly::MultilinearPoly;
use plonky_cat_sumcheck::{Sumcheck, SumcheckClaim, SumcheckWitness};
use plonky_cat_transcript::AlgebraicTranscript;
use plonky_cat_prover::prove;
use plonky_cat_verifier::{verify, Verdict};

fn main() -> Result<(), String> {
    println!("=== plonky-cat sumcheck end-to-end test ===");

    let evals: Vec<BabyBear> = vec![
        BabyBear::from(1u32),
        BabyBear::from(2u32),
        BabyBear::from(3u32),
        BabyBear::from(4u32),
    ];

    let poly = MultilinearPoly::from_evals(2, evals)
        .map_err(|e| format!("poly: {e}"))?;

    let claimed_sum = poly.sum_over_hypercube();
    println!("polynomial: 2 variables, 4 evals");
    println!("claimed sum: {claimed_sum}");

    let claim = SumcheckClaim::new(claimed_sum, 2);
    let witness = SumcheckWitness::new(poly);
    let transcript = AlgebraicTranscript::<BabyBear>::new();

    let (proof, _) = prove::<Sumcheck<SumcheckWitness<BabyBear>>, _>(claim.clone(), witness, transcript)
        .map_err(|e| format!("prove: {e}"))?;

    println!("proof: {} round messages", proof.messages().len());
    println!("opening: {}", proof.opening().value());

    let v_transcript = AlgebraicTranscript::<BabyBear>::new();
    let v_claim = SumcheckClaim::new(claimed_sum, 2);

    let (verdict, _) = verify::<Sumcheck<SumcheckWitness<BabyBear>>, _>(v_claim, proof.messages(), v_transcript)
        .map_err(|e| format!("verify: {e}"))?;

    match verdict {
        Verdict::Accept(opening) => {
            println!("verifier accepted, opening: {}", opening.value());
            if opening == *proof.opening() {
                println!("PASS: openings match");
            } else {
                println!("FAIL: opening mismatch");
            }
        }
    }

    Ok(())
}
