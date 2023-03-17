use err_as_you_go::err_as_you_go;

#[err_as_you_go]
fn foo() -> Result<(), FooError> {
    Err(err!(Structy {
        bars: usize = 1,
        chars: char = 'a'
    }))?;
    Err(err!(Tuply(usize = 1, char = 'a')))?;
    Err(err!(Unit))?;
    Err(FooError::Unit)?;
    Ok(())
}

fn assert_usize(_: usize) {}
fn assert_char(_: char) {}

fn assert_foo_error(e: FooError) {
    match e {
        FooError::Structy { bars, chars } => {
            assert_usize(bars);
            assert_char(chars)
        }
        FooError::Tuply(_, _) => (),
        FooError::Unit => (),
    }
}

fn main() {}
