#![forbid(unsafe_code)]

mod error;
pub use self::error::Error;

use plonky_cat_reduce::{ProverStep, ReductionFunctor, TranscriptSerialize};
use plonky_cat_transcript::Transcript;

#[derive(Debug, Clone)]
pub struct Proof<M, O> {
    messages: Vec<M>,
    opening: O,
}

impl<M, O> Proof<M, O> {
    #[must_use]
    pub fn new(messages: Vec<M>, opening: O) -> Self {
        Self { messages, opening }
    }

    #[must_use]
    pub fn messages(&self) -> &[M] {
        &self.messages
    }

    #[must_use]
    pub fn opening(&self) -> &O {
        &self.opening
    }

    pub fn into_parts(self) -> (Vec<M>, O) {
        (self.messages, self.opening)
    }
}

const MAX_ROUNDS: usize = 1024;

/// Prove driver: anamorphism over a ReductionFunctor.
/// Squeezes challenges, calls prover_step, absorbs round messages into the
/// transcript for Fiat-Shamir soundness, collects messages until Done.
pub fn prove<R, T>(
    claim: R::Claim,
    witness: R::Witness,
    transcript: T,
) -> Result<(Proof<R::RoundMsg, R::BaseOpening>, T), Error<R::Error>>
where
    R: ReductionFunctor<Challenge = T::F>,
    R::RoundMsg: TranscriptSerialize<T::F>,
    T: Transcript,
    R::Error: std::fmt::Debug,
{
    prove_rec::<R, T>(claim, witness, transcript, Vec::new(), MAX_ROUNDS)
}

fn prove_rec<R, T>(
    claim: R::Claim,
    witness: R::Witness,
    transcript: T,
    collected: Vec<R::RoundMsg>,
    rounds_left: usize,
) -> Result<(Proof<R::RoundMsg, R::BaseOpening>, T), Error<R::Error>>
where
    R: ReductionFunctor<Challenge = T::F>,
    R::RoundMsg: TranscriptSerialize<T::F>,
    T: Transcript,
    R::Error: std::fmt::Debug,
{
    if rounds_left == 0 {
        Err(Error::MaxRoundsExceeded { limit: MAX_ROUNDS })
    } else {
        let (t2, challenge) = transcript.squeeze();

        R::prover_step(claim, witness, challenge)
            .map_err(Error::Reduction)
            .and_then(|step| match step {
                ProverStep::Continue(c) => {
                    let (c_next, w_next, msg) = c.into_parts();
                    let t3 = absorb_msg::<T>(t2, &msg);
                    let new_collected: Vec<R::RoundMsg> = collected.into_iter()
                        .chain(std::iter::once(msg))
                        .collect();
                    prove_rec::<R, T>(c_next, w_next, t3, new_collected, rounds_left - 1)
                }
                ProverStep::Done(d) => {
                    let (_c_final, _w_final, opening) = d.into_parts();
                    Ok((Proof::new(collected, opening), t2))
                }
            })
    }
}

fn absorb_msg<T: Transcript>(transcript: T, msg: &impl TranscriptSerialize<T::F>) -> T {
    msg.to_field_elements()
        .into_iter()
        .fold(transcript, |t, f| t.absorb(f))
}
