#![allow(unused)]

use err_as_you_go::err_as_you_go;

#[err_as_you_go(derive(Debug, thiserror::Error))]
fn simple_string_error() -> Result<(), FooError> {
    Err(err!(
        #[error("no bars :(")]
        NoBars
    ))
}

#[err_as_you_go(derive(Debug, thiserror::Error))]
fn interpolated_string_error(u: usize) -> Result<(), BarError> {
    Err(err!(
        #[error("{0} foos is not enough!")]
        NotEnoughFoos(usize = u)
    ))
}

#[err_as_you_go(derive(Debug, thiserror::Error))]
fn error_implements_from() -> Result<(), BazError> {
    Err(err!(
        #[error("fuck")]
        IoErr(
            #[from]
            std::io::Error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "blah")
        )
    ))
}

#[test]
fn thiserror_message_simple() {
    assert_eq!(simple_string_error().unwrap_err().to_string(), "no bars :(")
}

#[test]
fn thiserror_message_interpolated() {
    assert_eq!(
        interpolated_string_error(1).unwrap_err().to_string(),
        "1 foos is not enough!"
    )
}

fn assert_error_implements_from(e: std::io::Error) -> BazError {
    e.into()
}
