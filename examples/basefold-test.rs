use plonky_cat_field::BabyBear;
use plonky_cat_hash::AlgebraicHash;
use plonky_cat_poly::MultilinearPoly;
use plonky_cat_fri::{FriClaim, FriWitness};
use plonky_cat_sumcheck::{SumcheckClaim, SumcheckWitness};
use plonky_cat_basefold::{BaseFold, BaseFoldShared, BaseFoldWitness};
use plonky_cat_sumcheck::LinearWitness;
use plonky_cat_transcript::{AlgebraicTranscript, Transcript};
use plonky_cat_prover::prove;
use plonky_cat_verifier::{verify, Verdict};

type H = AlgebraicHash<BabyBear>;

fn main() -> Result<(), String> {
    println!("=== plonky-cat BaseFold end-to-end test ===");
    println!("thesis: BaseFold = Interleave<BaseFoldAdapter>");
    println!();

    // 2-variable multilinear: P(x0, x1) with 4 evals
    let evals: Vec<BabyBear> = vec![
        BabyBear::from(1u32),
        BabyBear::from(2u32),
        BabyBear::from(3u32),
        BabyBear::from(4u32),
    ];

    let poly = MultilinearPoly::from_evals(2, evals.clone())
        .map_err(|e| format!("poly: {e}"))?;

    let claimed_sum = poly.sum_over_hypercube();
    println!("polynomial: 2 variables, 4 evals");
    println!("claimed sum: {claimed_sum}");

    // Build FRI side: codeword = evals (rate 1 for v0.1 lockstep)
    let fri_witness = FriWitness::<H>::build(evals)
        .map_err(|e| format!("fri witness: {e}"))?;
    let merkle_root = fri_witness.merkle_root()
        .map_err(|e| format!("merkle root: {e}"))?;
    let fri_claim = FriClaim::until_constant(merkle_root, 4);

    println!("FRI codeword length: 4");
    println!("merkle root: {merkle_root}");

    // Build sumcheck side
    let sum_claim = SumcheckClaim::new(claimed_sum, 2);
    let sum_witness = SumcheckWitness::new(poly);

    // Assemble shared state
    let shared = BaseFoldShared::new(fri_claim, sum_claim);
    let witness = BaseFoldWitness::new(fri_witness, sum_witness);

    // Prove
    println!();
    println!("--- proving ---");
    let transcript = AlgebraicTranscript::<BabyBear>::new();

    let (proof, _) = prove::<BaseFold<H, LinearWitness<BabyBear>>, _>(shared.clone(), witness, transcript)
        .map_err(|e| format!("prove: {e:?}"))?;

    println!("proof: {} round messages", proof.messages().len());
    println!("opening: {:?}", proof.opening());

    // Verify
    println!();
    println!("--- verifying ---");
    let v_transcript = AlgebraicTranscript::<BabyBear>::new();

    let (verdict, _) = verify::<BaseFold<H, LinearWitness<BabyBear>>, _>(shared, proof.messages(), v_transcript)
        .map_err(|e| format!("verify: {e:?}"))?;

    match verdict {
        Verdict::Accept(opening) => {
            println!("verifier ACCEPTED");
            println!("verifier opening: {:?}", opening);
            if opening == *proof.opening() {
                println!();
                println!("PASS: BaseFold = Interleave<BaseFoldAdapter> works end-to-end");
            } else {
                println!();
                println!("FAIL: opening mismatch");
            }
        }
    }

    Ok(())
}
