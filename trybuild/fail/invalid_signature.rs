use err_as_you_go::err_as_you_go;

#[err_as_you_go]
fn foo() -> std::io::Result<()> {
    Ok(())
}

#[err_as_you_go]
fn bar() {}

fn main() {}
