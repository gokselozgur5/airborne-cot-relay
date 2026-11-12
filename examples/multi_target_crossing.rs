//! Multi-target crossing walkthrough.
//!
//! Two vehicles drive toward each other across the field of view. Each frame
//! is passed to the targeter, and every confirmed track is printed so readers
//! can see the tracker keep the two identities apart.

use airborne_cot_relay::{PlatformPose, RelayConfig, TargetDetection, Targeter};
