/// Describes an optional action that is only executed if an Ok result
/// was returned (used in collect_or_pass_error);
pub enum MaybeAction<T> {
    // optional action should not be performed
    Skip,
    // action should be performed with the provided value
    Do(T),
}

pub trait ErrorCollector<E> {
    fn record_error(&mut self, error: E);
}

/// Handles a generic Result with an SpannedCompilerError error.
/// If the result is Ok(), an Ok(MaybeAction::Do) with the result is returned
/// If result is Error() and collected_errors is Some, the error is appended to the collected_errors
/// and an Ok(MaybeAction::Skip) is returned
/// If result is Error() and collected_errors is None, the error is directly returned
pub fn collect_or_pass_error<T, E, Collector: ErrorCollector<E>>(
    collected_errors: &mut Option<Collector>,
    result: Result<T, E>,
) -> Result<MaybeAction<T>, E> {
    if let Err(error) = result {
        if let Some(collected_errors) = collected_errors {
            collected_errors.record_error(error);
            Ok(MaybeAction::Skip)
        } else {
            Err(error)
        }
    } else {
        result.map(MaybeAction::Do)
    }
}
