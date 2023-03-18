use errgo::errgo;

#[errgo]
fn foo(s: &str) -> Result<usize, FooError> {
    s.parse::<usize>().map_err(|_| err!(ParseError))
}

fn assert_foo_error(e: FooError) {
    match e {
        FooError::ParseError => (),
    }
}

fn main() {}
