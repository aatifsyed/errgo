use err_as_you_go::err_as_you_go;

#[err_as_you_go]
fn foo() -> Result<(), FooError> {
    return Err(err!(1Bar));
    Ok(())
}

fn main() {}
