use std::io::{self, BufRead, BufReader, Read};

pub fn read_lines<R: Read>(reader: R) -> io::Result<Vec<String>> {
    BufReader::new(reader)
        .lines()
        .filter(|r| match r {
            Ok(s) => !s.is_empty(),
            Err(_) => true,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_lines_separated_by_newline() {
        let input = b"alpha\nbeta\ngamma\n" as &[u8];
        let lines = read_lines(input).unwrap();
        assert_eq!(lines, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn handles_input_without_trailing_newline() {
        let input = b"alpha\nbeta" as &[u8];
        let lines = read_lines(input).unwrap();
        assert_eq!(lines, vec!["alpha", "beta"]);
    }

    #[test]
    fn empty_input_returns_empty_vec() {
        let input = b"" as &[u8];
        assert_eq!(read_lines(input).unwrap(), Vec::<String>::new());
    }

    #[test]
    fn strips_carriage_return_from_crlf() {
        let input = b"alpha\r\nbeta\r\n" as &[u8];
        let lines = read_lines(input).unwrap();
        assert_eq!(lines, vec!["alpha", "beta"]);
    }

    #[test]
    fn skips_blank_lines() {
        let input = b"alpha\n\nbeta\n" as &[u8];
        let lines = read_lines(input).unwrap();
        assert_eq!(lines, vec!["alpha", "beta"]);
    }
}
