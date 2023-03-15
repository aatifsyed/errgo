use err_as_you_go::err_as_you_go;

#[err_as_you_go]
fn foo() {
    err!(1Bar);
}

fn main() {}
