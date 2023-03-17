<div align="center">

[![crates-io](https://img.shields.io/crates/v/err-as-you-go.svg)](https://crates.io/crates/err-as-you-go)
[![docs-rs](https://docs.rs/err-as-you-go/badge.svg)](https://docs.rs/err-as-you-go)
[![github](https://img.shields.io/static/v1?label=&message=github&color=grey&logo=github)](https://github.com/aatifsyed/err-as-you-go)

</div>

# `err-as-you-go`

Generate `enum` error types inline.

If you want:
- to easy throw errors inline like with [anyhow]
- to make your error types handleable in a nice enum like [thiserror]

then this is the crate for you!

```rust
use err_as_you_go::err_as_you_go;

#[err_as_you_go]
fn shave_yaks(
    num_yaks: usize,
    empty_buckets: usize,
    num_razors: usize,
) -> Result<(), ShaveYaksError> {
    if num_razors == 0 {
        return Err(err!(NotEnoughRazors));
    }
    if num_yaks > empty_buckets {
        return Err(err!(NotEnoughBuckets {
            got: usize = empty_buckets,
            required: usize = num_yaks,
        }));
    }
    Ok(())
}
```
Under the hood, a struct like this is generated:
```rust
enum ShaveYaksError { // name and visibility are taken from function return type and visibility
    NotEnoughRazors,
    NotEnoughBuckets {
        got: usize,
        required: usize,
    }
}
```

Importantly, you can derive on the generated struct, _and_ passthrough attributes, allowing you to use crates like [thiserror].
```rust

#[err_as_you_go(derive(Debug, thiserror::Error))]
fn shave_yaks(
    num_yaks: usize,
    empty_buckets: usize,
    num_razors: usize,
) -> Result<(), ShaveYaksError> {
    if num_razors == 0 {
        return Err(err!(#[error("not enough razors!")] NotEnoughRazors));
    }
    if num_yaks > empty_buckets {
        return Err(err!(#[error("not enough buckets - needed {required}")] NotEnoughBuckets {
            got: usize = empty_buckets,
            required: usize = num_yaks,
        }));
    }
    Ok(())
}
```

Which generates the following:
```rust
#[derive(Debug, thiserror::Error)]
enum ShaveYaksError {
    #[error("not enough razors!")]
    NotEnoughRazors,
    #[error("not enough buckets - needed {required}")]
    NotEnoughBuckets {
        got: usize,
        required: usize,
    }
}
```
And `err!` macro invocations are replaced with struct instantiations - no matter where they are in the function body!

If you need to reuse the same variant within a function, just use the normal construction syntax:
```rust
#[err_as_you_go]
fn foo() -> Result<(), FooError> {
    if true {
        return Err(err!(Bar));
    }
    Err(FooError::Bar)
}
```

[anyhow]: https://docs.rs/anyhow
[thiserror]: https://docs.rs/thiserror
