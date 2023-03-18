use errgo::errgo;

#[errgo]
fn foo() -> Result<(), FooError> {
    Err(err!(Bar {
        bars: usize = 1,
        chars: char = 'a'
    }))?;
    Ok(())
}

fn assert_usize(_: usize) {}
fn assert_char(_: char) {}

fn assert_foo_error(e: FooError) {
    match e {
        FooError::Bar { bars, chars } => {
            assert_usize(bars);
            assert_char(chars)
        }
    }
}

fn main() {}
