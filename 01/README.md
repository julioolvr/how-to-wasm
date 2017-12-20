# Compiling a simple function

WebAssembly supports 4 types - `i32`, `i64`, `f32` and `f64`. So to keep things simple,
I'll start by exporting a function that takes parameters of these types and has a return
value of one of these types as well.

## How to run the example
- Serve this directory (e.g. with `python -m SimpleHTTPServer 8081`).
- Go to `localhost:8081` and look at the console.

## How it works

`sum.rs` contains the Rust code that we want to export to WASM. It's a function that receives
two `i32` parameters, sums them and returns the result. Two important details here: the function
has to be public in order to be eventually accessible from JavaScript, _and_ it needs to be
tagged with `#[no_mangle]` so that its name is kept after compilation.

To compile it, we need Rust nightly (at the moment of this writing) with the target
`wasm32-unknown-unknown`. `rustc` can handle this, by updating the nightly channel and
then installing the target:

```
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

Once that's setup, we compile the Rust file to WASM as a library with the
`--crate-type=cdylib` flag:

```
rustc +nightly sum.rs --crate-type=cdylib --target wasm32-unknown-unknown
```

This will generate `sum.wasm`. That file can be optimized with
[`wasm-gc`](https://github.com/alexcrichton/wasm-gc), we do so:

```
wasm-gc sum.wasm sum-small.wasm
```

And that's all for the Rust side of things.

From JavaScript, we need to:

- Fetch the WASM file
- Instantiate a WebAssembly module
- Use the exported function!

The code has pretty much a 1-to-1 correspondence to those instructions:

```js
    fetch('./sum-small.wasm')
      .then(response => response.arrayBuffer())
      .then(bytes => WebAssembly.instantiate(bytes, {}))
      .then(results => {
        console.log(results.instance.exports.sum(1, 2));
      })
```

The only extra in there is using the response from `fetch` as an `arrayBuffer`, but that's
simply what `WebAssembly.instantiate` expects as a parameter. We reach out to the module's
`instance` from the results, and then we call the `sum` function which can be found in
`exports` (alongside other functions that we might have exported, and somehow memory can
be exported too - I still have to figure out exactly what this means).

## Lessons learned

- We have to tell `rustc` that this is a library and not a binary, so that it will not
  complain because of the missing `main` function. This is done with the `crate-type` flag.
  [This page](https://doc.rust-lang.org/reference/linkage.html) lists the different types -
  at this point the exact difference between the types go over my head, but
  [this tweet](https://twitter.com/steveklabnik/status/934769437974069248) from Steve Klabnik
  uses the `cdylib` type, and it makes sense according to the description in the reference:
  "This is used when compiling Rust code as a dynamic library to be loaded from another
  language."
- In order to generate WASM, we have to set it as a compilation _target_, with the `target`
  flag in `rustc`. The target we need is `wasm32-unknown-unknown`.
- That target is only available in nightly Rust. The tool used to manage different versions
  of Rust is `rustup`. So, `rustup update nightly` will make sure we have the latest version
  in the nightly channel, and `rustup target add wasm32-unknown-unknown --toolchain nightly`
  will add the WASM target to that version. Then, we can use the `+nightly` option in `rustc`
  to make sure the WASM target is available.
- Rust is smart! If the function is not public, it will not be exported. So make sure the
  module exposes public functions from Rust with `pub`. The compiler is also smart and will
  let you know :)
- When a binary is generated, there might be no need to store the function names specifically,
  at the binary level the functions are usually identified by memory offsets. Since we want to
  be able to call functions from JS by their name, we need to tag them with `#[no_mangle]` so
  that Rust will keep their names after they get compiled.
- The WASM file generated can be optimized by a tool called
  [`wasm-gc`](https://github.com/alexcrichton/wasm-gc). In my case, the generated `.wasm` file
  went from 177KB to 134KB.
- `i64` cannot be used to communicate between WASM and JS "because JavaScript currently has no
  precise way to represent an i64".

## Useful Links

- [WebAssembly — The missing tutorial](https://medium.com/@MadsSejersen/webassembly-the-missing-tutorial-95f8580b08ba).
- [Linkage in rust](https://doc.rust-lang.org/reference/linkage.html) - Talks about the
  different `crate-type`s.
- [Adding the `wasm32-unknown-unknown` target](https://github.com/rust-lang/rust/pull/45905).
- [Exported functions in MDN](https://developer.mozilla.org/en-US/docs/WebAssembly/Exported_functions). It mentions how
  `i64` cannot be used from JS.