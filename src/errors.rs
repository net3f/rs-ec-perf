
// TODO: Inclide CodeParams struct, not n and k ??
// TODO: Remove the word validator
#[non_exhaustive]
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum Error {
	#[error("Valdiator number {want} exceeds max of {max}")]
	ValidatorCountTooHigh { want: usize, max: usize },

	#[error("At least 3 validators required, but have {0}")]
	ValidatorCountTooLow(usize),

	#[error("Size of the payload is zero")]
	PayloadSizeIsZero,

	#[error("Needs at least {min} shards of {all} to recover, have {have}")]
	NeedMoreShards { have: usize, min: usize, all: usize },

	#[error("Parameters: n (= {n}) and k (= {k}) both must be a power of 2")]
	ParamterMustBePowerOf2 { n: usize, k: usize },
}

pub type Result<T> = std::result::Result<T, Error>;
