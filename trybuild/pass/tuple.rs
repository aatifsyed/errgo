use errgo::errgo;

#[errgo]
fn foo() -> Result<(), FooError> {
    Err(err!(Bar(usize = 1, char = 'a')))?;
    Ok(())
}

fn assert_usize(_: usize) {}
fn assert_char(_: char) {}

fn assert_foo_error(e: FooError) {
    match e {
        FooError::Bar(u, c) => {
            assert_usize(u);
            assert_char(c)
        }
    }
}

fn main() {}
