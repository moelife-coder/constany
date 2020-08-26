# Constany: convert any rust function to constant fn

Constant functions in rust is a group of function with its result will be evaluated during compile time. It can significantly reduce generated binary size and improve performance. However, due to technical and logical limitations, some expression cannot be evaluated as constant function. For example:

```rust
fn main() {
   println!("{}", add_one_to_six());
}
const fn add_one_to_six() -> String {
   let mut a = 1;
   for b in 1..7 {
       a += b;
   } // error[E0744]: `for` is not allowed in a `const fn`
   return a.to_string();
}
```

will fail.

Constany use a workaround for this: by using Constany, the function and the main function will be compiled twice. The value of the function will be evaluated at the first time, and the value will be wrapped into a constant function at the second time.

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
    println!("Hello, world!");
    function_evaled_at_compile_time();
}
#[constany::const_fn]
fn function_evaled_at_compile_time() -> i32 {
    let mut a = 1;
    let b = 5;
    for _ in 0..b {
        a += 1; // For loop is not allowed in `const fn`
    }
    a
}
```

When you need to build the function, execute:

```bash
$ cargo run --featues stage_one
$ cargo build --features stage_two // If you want to run the command instead, use `cargo run`
```

And your function will be interpreted as constant function.

## Multiple constant function
Having multiple constant functions are also applicable, you just need to make sure every function you want to make constant are labeled with `const_fn` and the function name is inside `main_fn`:

```rust
#[cfg(any(
    not(any(feature = "stage_one", feature = "stage_two")),
    all(feature = "stage_two", feature = "stage_one")
))]
use constany_blank as constany;
#[cfg(all(feature = "stage_one", not(feature = "stage_two")))]
use constany_stage_one as constany;
#[cfg(all(feature = "stage_two", not(feature = "stage_one")))]
use constany_stage_two as constany;
#[constany::main_fn("function_evaled_at_compile_time", "function_evaled_at_compile_time")]
fn main() {
    println!("Hello, world!");
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

## Function with non-primitive result
Returning a non-primitive result (probably `struct` or `enum`) is troublesome and prone to error. The most elegant way is to use `lazy_static` for stage one and default to avoid compiler warning, and use constant value function for stage two:
```rust
#[cfg(feature = "stage_two")]
const ABC: String = constant_function().to_string();
#[cfg(not(feature = "stage_two"))]
lazy_static::lazy_static! {
    const ref ABC: String = constant_function().to_string();
}
```
However, this will not work for most of the non-primitive type because their constructor is usually not `static`.

There are two workaround for this: the `debug + pub` solution and `memop` solution.

The `debug + pub` solution first use `debug` trait to print the structure, and use the `pub` trait to rebuild it.
This solution can recreate the structure without `unsafe` code. However, this require the structure to derive `Debug`.
Current implementation also require the structure to not have `paths`, such as `std::string::String` (if there are `::` in the identifier, it's likely that this solution will not work out).

The `memop` solution transmute the memory directly.
This solution can rebuild any structure, but please note that this method is `unsafe` and very dangerous.
The generated function will be `fn` instead of `const_fn` because memory allocation is not allowed in `const`, although the memory itself is hard-coded inside the function.


