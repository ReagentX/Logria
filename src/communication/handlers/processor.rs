use std::io::Result;

use crate::communication::reader::MainWindow;

pub trait ProcessorMethods {
    fn return_to_normal(&mut self, window: &mut MainWindow) -> Result<()>;
    fn clear_matches(&mut self, window: &mut MainWindow) -> Result<()>;
    fn process_matches(&mut self, window: &mut MainWindow) -> Result<()>;
}

/// The step size for progress indicator updates.
const STEP: usize = 999;
/// Threshold for when to print progress updates in the aggregator.
const THRESHOLD: usize = 25_000;

#[inline(always)]
pub fn update_progress(
    window: &mut MainWindow,
    start: usize,
    end: usize,
    index: usize,
) -> Result<bool> {
    // Update the user interface with the current state
    if end - start > THRESHOLD && (index.is_multiple_of(STEP) || index == end - 1) {
        let word = if index == end - 1 {
            "Processed"
        } else {
            "Processing"
        };
        window.write_to_command_line(&format!(
            "{word} messages: {}/{} ({}%)",
            (index + 1) - start,
            end - start,
            ((index + 1 - start) * 100) / (end - start)
        ))?;
        return Ok(true);
    }
    Ok(false)
}
