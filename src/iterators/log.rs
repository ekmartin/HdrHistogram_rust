use Counter;
use Histogram;
use iterators::{HistogramIterator, PickyIterator};

/// An iterator that will yield at log-size steps through the histogram's value range.
pub struct Iter<'a, T: 'a + Counter> {
    hist: &'a Histogram<T>,

    nextValueReportingLevel: f64,
    logBase: f64,

    currentStepLowestValueReportingLevel: i64,
    currentStepHighestValueReportingLevel: i64,
}

impl<'a, T: 'a + Counter> Iter<'a, T> {
    /// Construct a new logarithmic iterator. See `Histogram::iter_log` for details.
    pub fn new(hist: &'a Histogram<T>,
               valueUnitsInFirstBucket: i64,
               logBase: f64)
               -> HistogramIterator<'a, T, Iter<'a, T>> {
        HistogramIterator::new(hist,
                               Iter {
                                   hist: hist,
                                   logBase: logBase,
                                   nextValueReportingLevel: valueUnitsInFirstBucket as f64,
                                   currentStepHighestValueReportingLevel: valueUnitsInFirstBucket -
                                                                          1,
                                   currentStepLowestValueReportingLevel:
                                       hist.lowest_equivalent(valueUnitsInFirstBucket - 1),
                               })
    }
}

impl<'a, T: 'a + Counter> PickyIterator<T> for Iter<'a, T> {
    fn pick(&mut self, index: usize, _: T) -> bool {
        let val = self.hist.value_for(index);
        if val >= self.currentStepLowestValueReportingLevel || index == self.hist.last() {
            self.nextValueReportingLevel *= self.logBase;
            self.currentStepHighestValueReportingLevel = self.nextValueReportingLevel as i64 - 1;
            self.currentStepLowestValueReportingLevel = self.hist
                .lowest_equivalent(self.currentStepHighestValueReportingLevel);
            true
        } else {
            false
        }
    }

    fn more(&mut self, next_index: usize) -> bool {
        // If the next iterate will not move to the next sub bucket index (which is empty if if we
        // reached this point), then we are not yet done iterating (we want to iterate until we are
        // no longer on a value that has a count, rather than util we first reach the last value
        // that has a count. The difference is subtle but important)...
        self.hist.lowest_equivalent(self.nextValueReportingLevel as i64) <
        self.hist.value_for(next_index)
    }
}
