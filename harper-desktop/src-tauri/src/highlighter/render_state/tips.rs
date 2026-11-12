//! Occasional, local-only teaching tips for the native suggestion popup.
//!
//! Keep tips specific to supported Desktop features. Adding a tip requires no persistence or IPC;
//! the drawer measures and wraps each tip to fit its text, with scrolling for tall content.

pub const TIPS: &[&str] = &[
    "Add names and technical terms in Settings → Dictionary.",
    "Use Dismiss to ignore a suggestion without disabling its rule.",
    "Disable an unwanted rule from the suggestion popup. Re-enable it in Settings → Rules.",
    "Choose your English dialect in Settings → General.",
    "Want fewer interruptions? Try a 300 ms checking delay in Settings → General.",
    "Choose which apps Harper checks in Settings → Integrations.",
    "Use the settings button in this popup to open Harper’s preferences.",
    "Harper checks your writing locally. Your text stays on your device.",
    "Read a sentence aloud to catch awkward phrasing.",
    "Use consistent terminology so readers can follow your ideas.",
];

/// Chooses a uniformly random tip with a 10% probability, matching the browser extension.
///
/// Call only when a popup opens or changes lints, and retain `None` as well as selected tips until
/// that session ends.
pub fn choose_tip(rng: &mut fastrand::Rng) -> Option<&'static str> {
    (rng.u8(0..10) == 0).then(|| TIPS[rng.usize(0..TIPS.len())])
}
