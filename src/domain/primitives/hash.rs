#[derive(
    Debug, Copy, Clone,
    derive_more::Constructor, derive_more::From
)]
// kept for completeness of the decoded inline_message_id data, even though nothing reads it yet
pub struct AccessHash(#[allow(dead_code)] i64);
