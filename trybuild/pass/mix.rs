use errgo::errgo;

#[errgo(derive(Clone, Copy, Default))]
fn foo() -> Result<(), FooError> {
    Err(err!(Structy {
        bars: usize = 1,
        chars: char = 'a'
    }))?;
    Err(err!(Tuply(usize = 1, char = 'a')))?;
    Err(err!(
        #[default]
        Unit
    ))?;
    Err(FooError::Unit)?;
    Ok(())
}

fn assert_usize(_: usize) {}
fn assert_char(_: char) {}
fn assert_default(_: impl Default) {}

fn assert_foo_error(e: FooError) {
    assert_default(e);
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
