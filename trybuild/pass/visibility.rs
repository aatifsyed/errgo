mod outer {
    pub use inner::FooError;

    mod inner {
        use errgo::errgo;

        #[errgo(visibility(pub))]
        fn foo() -> Result<(), FooError> {
            todo!()
        }
    }
}

fn main() {}
