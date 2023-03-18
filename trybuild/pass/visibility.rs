mod outer {
    pub use inner::FooError;

    mod inner {
        use err_as_you_go::err_as_you_go;

        #[err_as_you_go(visibility(pub))]
        fn foo() -> Result<(), FooError> {
            todo!()
        }
    }
}

fn main() {}
