use err_as_you_go::err_as_you_go;

#[err_as_you_go(derive(Default))]
fn foo() -> Result<(), FooError> {
    Err(err!(
        #[default] // derive Default for enum will fail without a #[default] on a variant
        Bar
    ))?;
    Ok(())
}

fn assert_default(_: impl Default) {}

fn assert_foo_error(e: FooError) {
    assert_default(e);
}

fn main() {}
