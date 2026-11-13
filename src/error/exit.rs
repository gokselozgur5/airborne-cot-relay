//! Process exit status for a run that ends with a `RelayError`.
//!
//! | Status | Meaning |
//! | ---: | :--- |
//! | 0 | Clean exit |
//! | 2 | `InvalidConfig` |
//! | 3 | `IoError` |
//! | 4 | `DecodeError` |
//! | 5 | `TargetingError` |

/// Exit status for an invalid configuration.
pub const EXIT_CONFIG: i32 = 2;
/// Exit status for a socket or file failure.
pub const EXIT_IO: i32 = 3;
/// Exit status for an undecodable input stream.
pub const EXIT_DECODE: i32 = 4;
/// Exit status for a targeting failure.
pub const EXIT_TARGETING: i32 = 5;
