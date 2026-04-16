#![forbid(unsafe_code)]

mod error;
pub use self::error::Error;

use plonky_cat_reduce::{ReductionFunctor, TranscriptSerialize, VerifierStep};
use plonky_cat_transcript::Transcript;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict<O> {
    Accept(O),
}

/// Verify driver: catamorphism over a ReductionFunctor.
/// Processes proof messages via verifier_step, absorbing each into the transcript
/// for Fiat-Shamir consistency.  Expects Done exactly on the last message.
pub fn verify<R, T>(
    claim: R::Claim,
    messages: &[R::RoundMsg],
    transcript: T,
) -> Result<(Verdict<R::BaseOpening>, T), Error<R::Error>>
where
    R: ReductionFunctor<Challenge = T::F>,
    R::RoundMsg: Clone + TranscriptSerialize<T::F>,
    T: Transcript,
    R::Error: std::fmt::Debug,
{
    verify_rec::<R, T>(claim, messages, transcript, 0)
}

fn verify_rec<R, T>(
    claim: R::Claim,
    remaining: &[R::RoundMsg],
    transcript: T,
    round: usize,
) -> Result<(Verdict<R::BaseOpening>, T), Error<R::Error>>
where
    R: ReductionFunctor<Challenge = T::F>,
    R::RoundMsg: Clone + TranscriptSerialize<T::F>,
    T: Transcript,
    R::Error: std::fmt::Debug,
{
    remaining.split_first()
        .ok_or(Error::ProtocolNotDone { messages_consumed: round })
        .and_then(|(msg, rest)| {
            let (t2, challenge) = transcript.squeeze();
            let t3 = absorb_msg::<T>(t2, msg);

            R::verifier_step(claim, msg.clone(), challenge)
                .map_err(Error::Reduction)
                .and_then(|step| match step {
                    VerifierStep::Continue(c) => {
                        verify_rec::<R, T>(c.into_inner(), rest, t3, round + 1)
                    }
                    VerifierStep::Done(d) => {
                        if rest.is_empty() {
                            let (_c_final, opening) = d.into_parts();
                            Ok((Verdict::Accept(opening), t3))
                        } else {
                            Err(Error::UnexpectedDone { round })
                        }
                    }
                })
        })
}

fn absorb_msg<T: Transcript>(transcript: T, msg: &impl TranscriptSerialize<T::F>) -> T {
    msg.to_field_elements()
        .into_iter()
        .fold(transcript, |t, f| t.absorb(f))
}
