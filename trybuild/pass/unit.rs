use err_as_you_go::err_as_you_go;

#[err_as_you_go]
fn foo() -> Result<(), FooError> {
    Err(err!(Bar))?;
    Ok(())
}

fn assert_foo_error(e: FooError) {
    match e {
        FooError::Bar => (),
    }
}

fn main() {}
