use crate::default;
use crate::Label;
use crate::OptUsage;
use crate::Usage;
use crate::Bool;
use std::cell::Cell;
use std::marker::PhantomData;
use std::sync::Arc;
use std::rc::Rc;

// ===============
// === Logging ===
// ===============

macro_rules! warning {
    ($($ts:tt)*) => {
        warning(&format!($($ts)*));
    };
}

fn warning(msg: &str) {
    if inc_and_check_warning_count() {
        warning_no_count_check(msg)
    }
}

fn warning_no_count_check(msg: &str) {
    #[cfg(feature = "wasm")]
    web_sys::console::warn_1(&msg.into());
    #[cfg(not(feature = "wasm"))]
    eprintln!("{msg}");
}

/// We don't want to flood users with warnings, especially in interactive apps, where warnings can
/// be emitted per frame.
const MAX_WARNING_COUNT: usize = 100;

thread_local! {
    static WARNING_COUNT: Cell<usize> = const { Cell::new(0) };
}

fn inc_and_check_warning_count() -> bool {
    WARNING_COUNT.with(|count| {
        let new_count = count.get() + 1;
        count.set(new_count);
        let ok = new_count < MAX_WARNING_COUNT;
        if !ok && new_count == MAX_WARNING_COUNT {
            warning_no_count_check("Too many warnings, suppressing further ones.");
        }
        ok
    })
}

// ===================
// === UsageResult ===
// ===================

#[derive(Clone, Copy, Debug)]
struct UsageResult {
    requested: OptUsage,
    needed: OptUsage,
}

// ====================
// === UsageTracker ===
// ====================

#[doc(hidden)]
#[cfg(usage_tracking_enabled)]
#[derive(Clone, Debug)]
pub struct UsageTracker {
    data: Rc<std::cell::RefCell<UsageTrackerData>>,
}

#[cfg(usage_tracking_enabled)]
impl UsageTracker {
    #[track_caller]
    pub fn new() -> Self {
        Self { data: Rc::new(std::cell::RefCell::new(UsageTrackerData::new())) }
    }

    fn set_usage(&self, label: Label, usage: UsageResult) {
        self.data.borrow_mut().map.push((label, usage));
    }
}

impl Default for UsageTracker {
    #[track_caller]
    fn default() -> Self {
        Self::new()
    }
}

// ========================
// === UsageTrackerData ===
// ========================

#[derive(Debug, Default)]
struct UsageTrackerData {
    loc: String,
    map: Vec<(Label, UsageResult)>,
}

impl UsageTrackerData {
    #[track_caller]
    fn new() -> Self {
        let call_loc = std::panic::Location::caller();
        let loc = format!("{}:{}", call_loc.file(), call_loc.line());
        let map = default();
        Self { loc, map }
    }
}

#[cfg(not(feature = "wasm"))]
macro_rules! warning_body {
    ($s:ident, $($ts:tt)*) => {
        $s.push_str("\n    ");
        $s.push_str(&format!($($ts)*));
    };
}

#[cfg(feature = "wasm")]
macro_rules! warning_body {
    ($s:ident, $($ts:tt)*) => {
        $s.push_str("\n");
        $s.push_str(&format!($($ts)*));
    };
}

impl Drop for UsageTrackerData {
    fn drop(&mut self) {
        let mut not_used = vec![];
        let mut used_as_ref = vec![];
        for (label, usage) in &self.map {
            if usage.requested > usage.needed {
                if usage.needed.is_none() {
                    not_used.push(*label)
                } else {
                    used_as_ref.push(*label)
                }
            }
        }

        let mut msg = String::new();
        if !not_used.is_empty() {
            not_used.sort();
            warning_body!(msg, "Borrowed but not used: {}.", not_used.join(", "));
        }
        if !used_as_ref.is_empty() {
            used_as_ref.sort();
            warning_body!(msg, "Borrowed as mut but used as ref: {}.", used_as_ref.join(", "));
        }

        if !msg.is_empty() {
            let mut required = vec![];
            for (label, usage) in &self.map {
                if let Some(usage2) = usage.needed {
                    required.push((label, usage2));
                }
            }
            // If required is empty, we probably are in a conditional code, where the borrow was not
            // used. Otherwise, Clippy will complain about unused variable, so we don't need to
            // report it.
            if !required.is_empty() {
                required.sort_by(|a, b| a.0.cmp(b.0));
                let out = required.into_iter().map(|(label, usage)| {
                    match usage {
                        Usage::Ref => label.to_string(),
                        Usage::Mut => format!("mut {label}"),
                    }
                }).collect::<Vec<_>>();
                warning_body!(msg, "To fix the issue, use: &<{}>.", out.join(", "));
                warning!("Warning [{}]:{}", self.loc, msg);
            }
        }
    }
}

// === FieldUsageTracker ===

#[derive(Debug)]
pub(crate) struct FieldUsageTracker<Enabled: Bool> {
    label: Label,
    requested_usage: OptUsage,
    needed_usage: Arc<Cell<OptUsage>>,
    parent_needed_usage: Option<Arc<Cell<OptUsage>>>,
    disabled: Cell<bool>,
    tracker: Option<UsageTracker>,
    enabled_marker: PhantomData<Enabled>,
}

impl<Enabled: Bool> Drop for FieldUsageTracker<Enabled> {
    fn drop(&mut self) {
        let needed = self.needed_usage.get();
        self.register_parent_needed_usage(needed);
        let enabled = !self.disabled.get() && Enabled::bool();
        if enabled {
            let requested = self.requested_usage;
            let usage = UsageResult { requested, needed };
            if let Some(t) = self.tracker.as_mut() { t.set_usage(self.label, usage) }
            if needed < requested {
                // We don't want to report error on parent unless children are fixed.
                self.register_parent_needed_usage(Some(Usage::Mut))
            }
        }
    }
}

impl<Enabled: Bool> FieldUsageTracker<Enabled> {
    pub(crate) fn new(label: Label, requested_usage: OptUsage, tracker: UsageTracker) -> Self {
        let needed_usage = default();
        let parent_needed_usage = None;
        let disabled = default();
        let tracker = Some(tracker);
        let enabled_marker = PhantomData;
        FieldUsageTracker { label, requested_usage, needed_usage, parent_needed_usage, disabled, tracker, enabled_marker }
    }

    pub(crate) fn new_child<E: Bool>(&self, requested_usage: Usage, tracker: UsageTracker) -> FieldUsageTracker<E> {
        let label = self.label;
        let needed_usage = default();
        let parent_needed_usage = Some(self.needed_usage.clone());
        let disabled = default();
        let requested_usage = Some(requested_usage);
        let enabled_marker = PhantomData;
        let tracker = Some(tracker);
        FieldUsageTracker { label, requested_usage, needed_usage, parent_needed_usage, disabled, tracker, enabled_marker }
    }

    pub(crate) fn new_child_disabled<E: Bool>(&self) -> FieldUsageTracker<E> {
        let label = self.label;
        let requested_usage = Some(Usage::Mut);
        let needed_usage = default();
        let parent_needed_usage = Some(self.needed_usage.clone());
        let disabled = Cell::new(true);
        let enabled_marker = PhantomData;
        let tracker = None;
        FieldUsageTracker { label, requested_usage, needed_usage, parent_needed_usage, disabled, tracker, enabled_marker }
    }

    pub(crate) fn clone_disabled<E: Bool>(&self) -> FieldUsageTracker<E> {
        let label = self.label;
        let requested_usage = self.requested_usage;
        let needed_usage = self.needed_usage.clone();
        let parent_needed_usage = self.parent_needed_usage.clone();
        let disabled = Cell::new(true);
        let enabled_marker = PhantomData;
        let tracker = None;
        FieldUsageTracker { label, requested_usage, needed_usage, parent_needed_usage, disabled, tracker, enabled_marker }
    }

    pub(crate) fn disable(&self) {
        self.disabled.set(true);
    }

    pub(crate) fn register_usage(&self, usage: OptUsage) {
        self.needed_usage.set(self.needed_usage.get().max(usage));
    }

    pub(crate) fn register_parent_needed_usage(&self, usage: OptUsage) {
        if let Some(parent) = self.parent_needed_usage.as_ref() {
            parent.set(parent.get().max(usage));
        }
    }
}
