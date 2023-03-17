use err_as_you_go::err_as_you_go;

#[err_as_you_go]
fn foo() -> Result<(), FooError> {
    Ok(())
}

fn assert_foo_error(_: FooError) {}

fn main() {}
