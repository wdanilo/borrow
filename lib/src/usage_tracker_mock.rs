#![cfg(not(usage_tracking_enabled))]

use borrow::Label;

#[derive(Copy, Debug)]
#[repr(transparent)]
pub struct UsageTracker;

impl UsageTracker {
    #[inline(always)]
    pub fn new() -> Self {
        UsageTracker
    }
}

impl Clone for UsageTracker {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}
