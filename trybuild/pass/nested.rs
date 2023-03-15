use err_as_you_go::err_as_you_go;

#[err_as_you_go]
fn foo(s: &str) -> Result<usize, FooError> {
    s.parse::<usize>().map_err(|_| err!(ParseError))
}

fn assert_foo_error(e: FooError) {
    match e {
        FooError::ParseError => (),
    }
}

fn main() {}
