use errgo::errgo;

#[errgo]
fn foo() -> Result<(), FooError> {
    return Err(err!(1Bar));
    Ok(())
}

fn main() {}
