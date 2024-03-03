/// Returns whether the sequence `needle` is a part of `haystack`, regardless of its position
///
/// # Arguments
///
/// * `haystack` - The sequence to look for `needle`
/// * `needle` - The sequence that may be a part of `haystack`
///
/// # Examples
///
/// ```
/// // [1, 2, 3] is included in [1, 2, 3, 4, 5] -> returns true
/// assert_eq!(contains_sequence(&vec![1, 2, 3, 4, 5], &vec![1, 2, 3]), true);
/// // [3, 3, 3] is *NOT* included in [1, 2, 3, 4, 5] -> returns false
/// assert_eq!(contains_sequence(&vec![1, 2, 3, 4, 5], &vec![3, 3, 3]), false);
/// ```
pub fn contains_sequence<T: Eq>(haystack: &[T], needle: &[T]) -> bool {
    let haystack_len = haystack.len();
    let needle_len = needle.len();

    if needle_len > haystack_len {
        return false;
    }

    if haystack_len == 0 {
        /* this would be an incorrect edge case otherwise */
        return false;
    }

    let size_diff = haystack_len - needle_len;

    'outer: for haystack_start in 0..=size_diff {
        for needle_index in 0..needle_len {
            if haystack[needle_index + haystack_start] != needle[needle_index] {
                continue 'outer;
            }
        }

        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_match_start() {
        assert_eq!(
            contains_sequence(&vec![1, 2, 3, 4, 5], &vec![1, 2, 3]),
            true
        );
    }

    #[test]
    fn finds_match_middle() {
        assert_eq!(
            contains_sequence(&vec![1, 2, 3, 4, 5], &vec![2, 3, 4]),
            true
        );
    }

    #[test]
    fn finds_match_end() {
        assert_eq!(
            contains_sequence(&vec![1, 2, 3, 4, 5], &vec![3, 4, 5]),
            true
        );
    }

    #[test]
    fn finds_no_match() {
        assert_eq!(
            contains_sequence(&vec![1, 2, 3, 4, 5], &vec![3, 3, 3]),
            false
        );
    }

    #[test]
    fn finds_match_empty() {
        assert_eq!(contains_sequence(&vec![1, 2, 3, 4, 5], &vec![]), true);
    }

    #[test]
    fn finds_no_match_on_empty_haystack() {
        assert_eq!(contains_sequence(&vec![], &vec![1]), false);
    }

    #[test]
    fn finds_no_match_on_empty_haystack_and_needle() {
        assert_eq!(contains_sequence::<u8>(&vec![], &vec![]), false);
    }
}
