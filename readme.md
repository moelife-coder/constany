# Constany: convert any rust function to const function

![Stage One](https://img.shields.io/crates/v/constany_stage_one) ![License](https://img.shields.io/crates/l/constany_stage_one) ![Downloads](https://img.shields.io/crates/d/constany_blank)

> Constany allows you to build const (or at least pseudo-const) function out of any expression

In rust, const functions are a type of functions that interpreted by the compiler at compile time. Const functions have various restrictions to make sure that they can be evaluated at compile time. For most of the time, these restrictions are beneficial because it prevent misuse. However, sometimes the use is intended:

```rust
use std::collections::HashMap;

const DEFAULT_USER: HashMap<&'static str, u8> = HashMap::new(); // Error!

fn main() {}
```

or

```rust
fn main() {}

const fn add_one_to_six() -> u8 {
   let mut a = 1;
   for b in 1..7 {
       a += b;
   } // Error!
   a
}
```

Constany provides a workaround to manually override those limitations.

## Why const function?

- Compile-time evaluation: faster runtime execution

- Smaller binary size (if the function itself is LARGE)

## How constany works?

Constany use a workaround for this: the function marked as `constany::const_fn` and the main function will be compiled twice. For the first time, the value of the function will be recorded. For the second time, the function will be replaced by the value.

## Usage

**WARNING: library support is not implemented yet. PRs are welcomed.**

Using constany is a bit tricker than normal library.

Firstly, you need to make sure there's two feature in `Cargo.toml` and their optional dependicies.

`Cargo.toml:`
```toml
[features]
stage_one = ["constany_stage_one"]
stage_two = ["constany_stage_two"]
[dependencies]
constany_stage_one = {version = "0.1", optional = true}
constany_stage_two = {version = "0.1", optional = true}
constany_blank = {version = "1"}
```

`constany_blank` is not necessary if there's no grammar checker and programmer will not accidently compile the code without `--feature` flag; it is simply a blank implementation for constany macros to avoid the compiler to complain.

The next step involves `main.rs`:

`main.rs:`
```rust
#[cfg(any(
    not(any(feature = "stage_one", feature = "stage_two")),
    all(feature = "stage_two", feature = "stage_one")
))]
use constany_blank as constany; // This line is for grammar checkers that enable all feature / disable all feature. If you do not have a checker, you can delete those lines safely.
#[cfg(all(feature = "stage_one", not(feature = "stage_two")))]
use constany_stage_one as constany;
#[cfg(all(feature = "stage_two", not(feature = "stage_one")))]
use constany_stage_two as constany;
#[constany::main_fn("function_evaled_at_compile_time")]
fn main() {
    // Blah Blah Blah
    function_evaled_at_compile_time();
    // Blah Blah Blah
}
#[constany::const_fn]
fn function_evaled_at_compile_time() -> i32 {
    let mut a = 1;
    let b = 5;
    for _ in 0..b {
        a += 1;
    }
    a
}
```

Make sure `main` function is marked with `constany::main_fn()` and the constant function list is inside the bracket. Otherwise the function will not be compiled to constant.

### Compile for binary application

#### Compile manually (the long way)

When you need to build the function, execute:

```bash
$ cargo run --featues stage_one
$ cargo build --features stage_two // If you want to run the code instead, use `cargo run`
```

And your function will be interpreted as constant function.

#### Compile using build script (the experimental way)

You can add [our build script](build.rs) to your code folder. Please add it outside `src`, in the same folder as `Cargo.toml`:

```
|- Cargo.toml
|- Cargo.lock
|- src/
    |- main.rs
    |- blah.rs
|- build.rs // HERE!!!
```
## Issues & Gotchas

### Multiple constant function

Having multiple constant functions are also applicable, you just need to make sure every function you want to make constant are labeled with `const_fn` and the function name is inside `main_fn`:

```rust
// --snip--
// Please look at previous example for this part
// --snip--
#[constany::main_fn("function_evaled_at_compile_time", "function_evaled_at_compile_time_2")]
fn main() {
    function_evaled_at_compile_time();
    function_evaled_at_compile_time_2();
}
#[constany::const_fn]
fn function_evaled_at_compile_time() -> i32 {
    let mut a = 1;
    let b = 5;
    for _ in 0..b {
        a += 1;
    }
    a
}
#[constany::const_fn]
fn function_evaled_at_compile_time_2() -> i32 {
    let mut a = 1;
    let b = 100;
    for _ in 0..b {
        a += 1;
    }
    a
}
```

### Function with non-primitive result

Returning a non-primitive result is troublesome and prone to error. The most elegant way is to use `lazy_static` for stage one to avoid compiler warning, and use constant value function for stage two:

```rust
#[cfg(feature = "stage_two")]
const ABC: String = constant_function().to_string();
#[cfg(not(feature = "stage_two"))]
lazy_static::lazy_static! {
    const ref ABC: String = constant_function().to_string();
}
```
However, this will not work for most of the non-primitive type because their constructor is unlikely to be `static`.

There are two workaround for this: the `debug + pub` solution and `memop` solution.

#### The Debug + Pub solution

The `debug + pub` solution first use `debug` trait to print the structure, and use the `pub` trait to rebuild it.

This solution can recreate the structure without `unsafe` code. However, this require the structure to derive `Debug`.

Current implementation also require the structure to not have `paths`, such as `std::string::String` (if there are `::` in the identifier, it's likely that this solution will not work out).

To use this solution, you can simply label `constany::const_fn` because this is the default solution for constany.

#### The Memop solution

The `memop` solution transmute the memory directly.
This solution can rebuild any structure, but please note that this method is `unsafe` and very dangerous.

The generated function will be `fn` instead of `const_fn` because memory allocation is not allowed in `const`, although the memory itself is hard-coded inside the function.

To use this solution, you need to label target function as `constany::const_fn(memop)`:

```rust
// --snip--
// Please look at previous example for this part
// --snip--
#[constany::main_fn("function_evaled_at_compile_time")]
fn main() {
    function_evaled_at_compile_time();
}
#[constany::const_fn(memop)]
fn function_evaled_at_compile_time() -> String {
    let mut a = 1;
    let b = 5;
    for _ in 0..b {
        a += 1;
    }
    a.to_string()
}
```

Please note that if the function is returning a primitive type in rust, the memory operation will not be used regardless the `memop` flag.

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.