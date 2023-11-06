use thiserror::Error;

/// An exercise in creating a custom error type that can be used with Result<>.

type Result<T> = std::result::Result<T, MyError>;

// Using thiserror::Error to avoid having to implement the Display trait for my
// custom error.
#[derive(Error, Debug, PartialEq, Clone)]
enum MyError {
    #[error("Bad length")]
    BadLength,

    #[error("Authentication failed")]
    AuthenticationFailed,
}

#[allow(dead_code)]
fn returns_bad_length() -> Result<()> {
    Err(MyError::BadLength)
}

#[allow(dead_code)]
fn returns_authentication_failed() -> Result<()> {
    Err(MyError::AuthenticationFailed)
}

fn string_checks(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(MyError::BadLength);
    }

    Ok(())
}

fn main() {
    let s1 = "";
    let rc = string_checks(s1);
    match rc {
        Ok(_) => println!("String Checks Passed!"),
        Err(error) => println!("Error: {error}"),
    }

    let s2 = "Hello";
    let rc = string_checks(s2);
    match rc {
        Ok(_) => println!("String Checks Passed!"),
        Err(error) => println!("Error: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_for_bad_length_error() {
        let rc = returns_bad_length();
        assert!(rc.is_err());
        assert_eq!(rc.unwrap_err(), MyError::BadLength);
    }

    #[test]
    fn check_for_authentication_failed() {
        let rc = returns_authentication_failed();
        assert!(rc.is_err());
        assert_eq!(rc.unwrap_err(), MyError::AuthenticationFailed);
    }

    #[test]
    fn test_string_checks() {
        let s = "";
        let rc = string_checks(s);
        assert!(rc.is_err());
        assert_eq!(rc.unwrap_err(), MyError::BadLength);
    }
}
