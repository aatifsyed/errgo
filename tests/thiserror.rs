#[err_as_you_go::err_as_you_go(derive(Debug, thiserror::Error))]
fn foo() -> Result<(), FooError> {
    Err(err!(
        #[error("no bars :(")]
        Bar
    ))
}

#[test]
fn thiserror_message() {
    assert_eq!(foo().unwrap_err().to_string(), "no bars :(")
}
