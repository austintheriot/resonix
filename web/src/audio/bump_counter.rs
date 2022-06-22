/// Increments an interanl counter so that Yew knows interanal state has changed,
/// even when the internal state that is behind an `Arc` pointer still points to the same memory.
/// 
/// When an audio Handle struct is `Clone`d on global state updates, its clone is the
/// identical object to the original, because its internal `Arc` is simply cloned,
/// and the outer struct frame (with the `counter`) is identical.
///
/// This makes it impossible for Yew to know when the struct's inernal data has actually been
/// udpated internally, because it will always be compared to itself accross state updates.
///
/// For this reason, an internal counter is udated to "force" a difference in state
/// while keeping the underlying `buffer_selection` reference identical in memory.
///
/// This allows Yew to know that internal state has changed, while also keeping the state's
/// location in memory unchanged, so that it can be safely accessed from the audio thread.
pub trait BumpCounter {
    fn bump_counter(&mut self);
}
