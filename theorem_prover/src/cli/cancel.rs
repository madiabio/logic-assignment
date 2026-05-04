//! Ctrl+C handling and exit-code policy shared by CLI subcommands.

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

pub(crate) const EXIT_FAILURE: i32 = 1;
pub(crate) const EXIT_CANCELLED: i32 = 130;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InterruptEvent {
    ExitNow(i32),
    FirstCancellation,
}

/// Shared cancellation state driven by the process `Ctrl+C` handler.
#[derive(Clone)]
pub(crate) struct CancellationState {
    requested: Arc<AtomicBool>,
    interrupt_count: Arc<AtomicUsize>,
    exit_immediately: Arc<AtomicBool>,
}

impl CancellationState {
    /// Installs a `Ctrl+C` handler and exposes its atomic cancellation flag.
    pub(crate) fn install() -> Self {
        let state = Self::new_uninstalled();
        let handler_state = state.clone();
        ctrlc::set_handler(move || {
            if let InterruptEvent::ExitNow(code) = handler_state.record_interrupt() {
                std::process::exit(code);
            }
        })
        .expect("failed to install Ctrl+C handler");
        state
    }

    /// Builds cancellation state without registering a process signal handler.
    fn new_uninstalled() -> Self {
        let requested = Arc::new(AtomicBool::new(false));
        let interrupt_count = Arc::new(AtomicUsize::new(0));
        let exit_immediately = Arc::new(AtomicBool::new(true));
        Self {
            requested,
            interrupt_count,
            exit_immediately,
        }
    }

    #[cfg(test)]
    pub(crate) fn uninstalled_for_test() -> Self {
        Self::new_uninstalled()
    }

    /// Switches cancellation into summary-owned mode before proof/rules work starts.
    pub(crate) fn defer_exit_until_summary(&self) {
        self.exit_immediately.store(false, Ordering::Relaxed);
    }

    /// Records an interrupt and reports whether the handler should terminate now.
    pub(crate) fn record_interrupt(&self) -> InterruptEvent {
        let count = self.interrupt_count.fetch_add(1, Ordering::Relaxed) + 1;
        self.requested.store(true, Ordering::Relaxed);

        if self.exit_immediately.load(Ordering::Relaxed) {
            eprintln!("Cancellation requested.");
            return InterruptEvent::ExitNow(EXIT_CANCELLED);
        }

        if count == 1 {
            eprintln!("Cancellation requested. Press Ctrl+C again to force cancellation.");
            return InterruptEvent::FirstCancellation;
        }

        eprintln!("Force cancellation requested.");
        InterruptEvent::ExitNow(EXIT_CANCELLED)
    }

    /// Returns whether cancellation has been requested.
    pub(crate) fn is_requested(&self) -> bool {
        self.requested.load(Ordering::Relaxed)
    }

    /// Returns the raw atomic flag for proof-engine cancellation checks.
    pub(crate) fn flag(&self) -> &AtomicBool {
        &self.requested
    }
}

/// Returns the exit code for a `prove` batch after it finishes reporting.
pub(crate) fn prove_batch_exit_code(
    cancelled_count: usize,
    failed_to_process_count: usize,
    cancellation: &CancellationState,
) -> Option<i32> {
    if cancelled_count > 0 || cancellation.is_requested() {
        Some(EXIT_CANCELLED)
    } else if failed_to_process_count > 0 {
        Some(EXIT_FAILURE)
    } else {
        None
    }
}

/// Returns the exit code for a `rules` batch after it finishes reporting.
pub(crate) fn rules_batch_exit_code(cancelled: bool, failed_count: usize) -> Option<i32> {
    if cancelled {
        Some(EXIT_CANCELLED)
    } else if failed_count > 0 {
        Some(EXIT_FAILURE)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_interrupt_exits_immediately_before_summary_phase() {
        let cancellation = CancellationState::uninstalled_for_test();

        assert_eq!(
            cancellation.record_interrupt(),
            InterruptEvent::ExitNow(EXIT_CANCELLED)
        );
        assert!(cancellation.is_requested());
    }

    #[test]
    fn second_interrupt_exits_immediately_during_summary_phase() {
        let cancellation = CancellationState::uninstalled_for_test();
        cancellation.defer_exit_until_summary();

        assert_eq!(
            cancellation.record_interrupt(),
            InterruptEvent::FirstCancellation
        );
        assert!(cancellation.is_requested());

        assert_eq!(
            cancellation.record_interrupt(),
            InterruptEvent::ExitNow(EXIT_CANCELLED)
        );
        assert!(cancellation.is_requested());
    }

    #[test]
    fn prove_batch_exit_code_prefers_ctrl_c_exit_code_after_summary() {
        let cancellation = CancellationState::uninstalled_for_test();
        cancellation.defer_exit_until_summary();
        cancellation.record_interrupt();

        assert_eq!(
            prove_batch_exit_code(0, 0, &cancellation),
            Some(EXIT_CANCELLED)
        );
    }

    #[test]
    fn rules_batch_exit_code_uses_failure_exit_code_for_failed_runs() {
        assert_eq!(rules_batch_exit_code(false, 1), Some(EXIT_FAILURE));
    }

    #[test]
    fn rules_batch_exit_code_uses_ctrl_c_exit_code_for_cancelled_runs() {
        assert_eq!(rules_batch_exit_code(true, 0), Some(EXIT_CANCELLED));
    }
}
