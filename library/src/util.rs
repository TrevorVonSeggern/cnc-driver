use core::{slice, str::from_utf8_unchecked};

#[allow(dead_code)]
pub fn join_slices<'a, T>(left: &'a [T], right: &'a [T]) -> &'a [T] {
    unsafe {
        let len = right.len() + right.as_ptr().offset_from(left.as_ptr()) as usize;
        return slice::from_raw_parts(left.as_ptr(), len);
    }
}
#[allow(dead_code)]
pub fn join_str<'a>(left: &'a str, right: &'a str) -> &'a str {
    return unsafe { from_utf8_unchecked(join_slices(left.as_bytes(), right.as_bytes())) };
}
#[allow(dead_code)]
pub fn starts_with<T, I>(left: I, right: I) -> bool  where I: Iterator<Item=T>, T: Eq {
    let mut result = true;
    for t in left.zip(right) {
        if t.0 != t.1 {
            result = false;
            break;
        }
    }
    return result;
}

pub fn search_pattern<T: PartialEq>(mut haystack: &[T], needle: &[T]) -> i32 {
    let mut i = 0;
    if needle.len() == 0 {
        return 0;
    }
    while !haystack.is_empty() {
        if haystack.starts_with(needle) {
            return i;
        }
        i += 1;
        haystack = &haystack[1..];
    }
    return -1;
}

#[allow(dead_code)]
pub fn count_iter<T>(iter: impl Iterator<Item = T>) -> usize {
    iter.fold(0, |total, _| total + 1)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_str_join() {
        let full = "0123456789";
        let (part1, part2) = full.split_at(5);
        let joined = join_str(part1, part2);
        assert_eq!(full, joined);
    }
}
