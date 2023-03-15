use err_as_you_go::err_as_you_go;

#[err_as_you_go]
fn foo() {}

fn assert_foo_error(_: FooError) {}

fn main() {}
