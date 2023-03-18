use errgo::errgo;

#[errgo]
fn foo() -> std::io::Result<()> {
    Ok(())
}

#[errgo]
fn bar() {}

fn main() {}
