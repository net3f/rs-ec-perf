
pub const FIELD_NAME: &'static str = "f256";

pub type Elt = u8;
pub type Wide = u16;

pub const FIELD_BITS: usize = 8;
pub const FIELD_SIZE: usize = 1_usize << FIELD_BITS;

include!("f_log_mul.rs");

#[cfg(table_bootstrap_complete)]
include!("afft.rs");

// must be placed in a separate file, such that the preproc never tries to eval OUT_DIR
// in env which does not exist in the build.rs case
#[cfg(table_bootstrap_complete)]
include!(concat!(env!("OUT_DIR"), "/table_f256.rs"));


/* Needs Cleanup  */

pub type GFSymbol = Elt;
pub const ONEMASK: Elt = (FIELD_SIZE - 1) as Elt;

/// Quotient ideal generator given by tail of irreducible polynomial
pub const GENERATOR: Elt = 0x1D; //GF(2^8): x^8 + x^4 + x^3 + x^2 + 1
// Chen's suggested tower
// pub const GENERATOR: Elt = 0x79; //GF(2^8): z^8 + z^6 + z^5 + z^4 + z^3 + 1
// Select for GFNI compatability, but lacks an embedded GF(16).
// pub const GENERATOR: Elt = 0x1B; //GF(2^8): x^8 + x^4 + x^3 + x + 1

// impl Additive {
//     pub const ONE: Additive = Additive(???);
// }

/// Cantor basis' final element
pub const BASE_FINAL: Elt = 230;
// pub const BASE_FINAL: Elt = 238;


// Cantor basis
// pub const BASE: [Elt; FIELD_BITS] = [1, 214, 152, 146, 86, 200, 88, 230];


