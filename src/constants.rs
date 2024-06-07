
/// Number of nibbles (i.e. hex-digits) to display for intermediate hashes when
/// inspecting a [Trie].
const DIGEST_SUMMARY_LENGTH: usize = 12; // # of nibbles

/// Maximum number of nibbles (i.e. hex-digits) to display for prefixes before
/// adding an ellipsis
const PREFIX_CUTOFF: usize = 8; // # of nibbles

/// A special database key for storing and retrieving the root hash of the trie.
/// This is useful to ensure that the root hash doesn't get lost, and with it,
/// the entire trie.
const ROOT_KEY: &'static str = "__root__";

/// Size of the digest of the underlying hash algorithm.
pub const DIGEST_LENGTH: usize = 32; // # of bytes

pub static NULL_HASH: &[u8; DIGEST_LENGTH] = &[0x0; DIGEST_LENGTH];