// don't add extra code in the file.
// it will be included in test cases in native code.

pub fn get_intersection(
    chunk_offset: usize,
    chunk_size: usize,
    target_offset: usize,
    target_size: usize,
) -> Option<(usize, usize)> {
    if target_offset >= chunk_offset {
        if target_offset < (chunk_offset + chunk_size) {
            let end = target_offset + target_size;
            if end >= (chunk_offset + chunk_size) {
                // case 1:
                // chunk_begin, signature_begin, chunk_end, signature_end
                return Some((target_offset - chunk_offset, chunk_size));
            } else {
                // case 2:
                // chunk_begin, signature_begin, signature_end, chunk_end
                return Some((target_offset - chunk_offset, end - chunk_offset));
            }
        }
    } else {
        let end = target_offset + target_size;
        if end > chunk_offset {
            if end >= (chunk_offset + chunk_size) {
                // case 3:
                // signature_begin, chunk_begin, chunk_end, signature_end
                return Some((0, chunk_size));
            } else {
                // case 4:
                // signature_begin, chunk_begin, signature_end, chunk_end
                return Some((0, end - chunk_offset));
            }
        }
    }
    None
}
