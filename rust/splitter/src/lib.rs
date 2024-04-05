/// An example of using iterators to process text lines.
/// Input text format: multiple lines, each of the format:
/// text: text
/// Blank lines are also tolerated.
pub fn splitter(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| line.trim().split_once(':'))
        .map(|(_, tbl_name)| tbl_name.trim().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_INPUT: &str = "
    0: a.tb1
    1: a.tb2
    2: a.tb3
    ";

    #[test]
    fn test_splitter() {
        let r = splitter(TEST_INPUT);
        assert_eq!(3, r.len());
        assert_eq!("a.tb1", r[0]);
        assert_eq!("a.tb3", r[2]);
    }
}
