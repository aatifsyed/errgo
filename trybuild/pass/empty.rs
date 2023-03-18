use errgo::errgo;

#[errgo]
fn foo() -> Result<(), FooError> {
    Ok(())
}

fn assert_foo_error(_: FooError) {}

fn main() {}
