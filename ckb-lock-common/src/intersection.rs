// don't add extra code in the file.
// it will be included in test cases in native code.
use core::ops::Range;

pub fn get_intersection(chunk: Range<usize>, target: Range<usize>) -> Option<Range<usize>> {
    let max_start = chunk.start.max(target.start);
    let min_end = chunk.end.min(target.end);

    if max_start < min_end {
        Some(max_start..min_end)
    } else {
        None
    }
}
