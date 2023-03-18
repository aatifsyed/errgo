use errgo::errgo;

#[errgo(attributes(
    #[repr(u8)] 
    #[must_use = "gotta use me"]
))]
fn foo() -> Result<(), FooError> {
    Err(err!(Bar))?;
    Ok(())
}

fn assert_foo_error(e: FooError) {
    e as u8;
}

fn main() {}
