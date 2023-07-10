// don't add extra code in the file.
// it will be included in test cases in native code.
use core::ops::Range;

pub fn get_intersection(chunk: Range<usize>, target: Range<usize>) -> Option<Range<usize>> {
    let chunk_size = chunk.end - chunk.start;

    if target.start >= chunk.start {
        if target.start < (chunk.start + chunk_size) {
            if target.end >= (chunk.start + chunk_size) {
                // case 1:
                // chunk_begin, signature_begin, chunk_end, signature_end
                return Some(target.start - chunk.start..chunk_size);
            } else {
                // case 2:
                // chunk_begin, signature_begin, signature_end, chunk_end
                return Some(target.start - chunk.start..target.end - chunk.start);
            }
        }
    } else {
        if target.end > chunk.start {
            if target.end >= chunk.end {
                // case 3:
                // signature_begin, chunk_begin, chunk_end, signature_end
                return Some(0..chunk_size);
            } else {
                // case 4:
                // signature_begin, chunk_begin, signature_end, chunk_end
                return Some(0..target.end - chunk.start);
            }
        }
    }
    None
}
