use plonky_cat_field::BabyBear;
use plonky_cat_hash::AlgebraicHash;
use plonky_cat_code::{rs_encode, RsParams};
use plonky_cat_fri::{Fri, FriClaim, FriWitness};
use plonky_cat_transcript::{AlgebraicTranscript, Transcript};
use plonky_cat_prover::prove;
use plonky_cat_verifier::{verify, Verdict};

type F = BabyBear;
type H = AlgebraicHash<F>;

fn main() -> Result<(), String> {
    println!("=== FRI with rate 1/2 RS encoding ===");
    println!();

    // Polynomial coefficients: [1, 2, 3, 4] (degree 3, 4 coefficients)
    // Rate 1/2: message length = 4, codeword length = 8
    // log_n = 3 (2^3 = 8), log_rate_inv = 1
    let message: Vec<F> = [1, 2, 3, 4].iter()
        .map(|v| F::from(*v as u32))
        .collect();

    let params = RsParams::new(3, 1)
        .map_err(|e| format!("rs params: {e}"))?;

    println!("message: 4 coefficients");
    println!("codeword length: {} (rate 1/2)", params.codeword_len());

    // 8th root of unity for NTT
    let omega = F::root_of_unity(3)
        .map_err(|e| format!("root of unity: {e}"))?;

    let codeword = rs_encode(&message, omega, params)
        .map_err(|e| format!("rs encode: {e}"))?;

    println!("encoded codeword: {} elements", codeword.len());

    // Build FRI witness and claim
    let fri_witness = FriWitness::<H>::build(codeword)
        .map_err(|e| format!("fri witness: {e}"))?;
    let merkle_root = fri_witness.merkle_root()
        .map_err(|e| format!("merkle root: {e}"))?;

    // Fold all the way to a single constant
    let fri_claim = FriClaim::until_constant(merkle_root, 8);

    println!("merkle root: {merkle_root}");
    println!("FRI: fold from 8 to 1 (3 rounds)");

    println!();
    println!("--- proving ---");
    let transcript = AlgebraicTranscript::<F>::new();

    let (proof, _) = prove::<Fri<H>, _>(fri_claim.clone(), fri_witness, transcript)
        .map_err(|e| format!("prove: {e:?}"))?;

    println!("proof: {} round messages", proof.messages().len());
    println!("opening: {:?}", proof.opening());

    println!();
    println!("--- verifying ---");
    let v_transcript = AlgebraicTranscript::<F>::new();

    let (verdict, _) = verify::<Fri<H>, _>(fri_claim, proof.messages(), v_transcript)
        .map_err(|e| format!("verify: {e:?}"))?;

    match verdict {
        Verdict::Accept(opening) => {
            println!("verifier ACCEPTED");
            if opening == *proof.opening() {
                println!();
                println!("PASS: rate-1/2 FRI with real RS encoding");
            } else {
                println!("FAIL: opening mismatch");
            }
        }
    }

    Ok(())
}
