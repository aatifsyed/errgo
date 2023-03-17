use err_as_you_go::err_as_you_go;

#[err_as_you_go(derive(Clone, Copy))]
fn foo() -> Result<(), FooError> {
    Err(err!(Bar))?;
    Ok(())
}

fn assert_clone(_: impl Clone) {}
fn assert_copy(_: impl Copy) {}

fn assert_foo_error(e: FooError) {
    assert_copy(e);
    assert_clone(e);
}

fn main() {}
