# Function that returns a string

Next, I want to figure out exactly how to get a String from the Rust world to the
JS world. Turns out there's a lot going on here. And I want to emphasize - this is
mostly a bunch of mildly-researched conclusions that shouldn't be taken as _the truth_,
but only as how I understood everything which could be completely wrong.

## How to run the example
- Serve this directory (e.g. with `python -m SimpleHTTPServer 8081`).
- Go to `localhost:8081` and look at the console.

## How it works

### Rust

On the Rust side, we won't be able to make a function that returns a `String` directly -
that's not part of the types that WASM can handle, and in the end only those can be
used as the interface between Rust and JS code.

Since we can't return a String, what we're going to return is a pointer. Our WASM module's
memory is accessible from JavaScript, so we'll return the index on which the "returned"
String can be found in that memory. From Rust's point of view this is almost transparent -
we'll define that our function returns a `*mut std::os::raw::c_char` and then the compiler
will do the job of giving meaning to that pointer, making it an index in WASM's memory.

The problem with the pointer is that it marks the starting point in memory of the string.
How do we know where it ends? How do we know how much to read? I've seen a couple of ways
to do that online. One is to make two functions - one returning the pointer, and the other
returning the length of the string. The other one, which I liked more, is relying on a 0
byte to mark the end of the string. If my memory serves me right, that's the way strings
are handled in C - there's no concept of "string" as a data type, but you just read char
after char until you reach a 0 byte. In order for this to work, it has to be ensured that
no 0 byte is present as part of the string.

Rust provides the [`std::ffi::CString`](https://doc.rust-lang.org/std/ffi/struct.CString.html)
struct for that purpose. It ensures that the string will end with a 0 byte, and that it
won't have any internal 0 bytes. But again, we can't return the `CString` by itself, we
need to return a [raw pointer](https://doc.rust-lang.org/book/second-edition/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer).

The way we usually return references to things in Rust is with, well, references. So I'd
be tempted to write something like `pub fn give_me_a_string() -> &'static CString`. But
references only work within the world of Rust - the compiler ensures that they are valid
as long as they have to be, for instance. Here we're interacting with the external world.
The compiler has no way of knowing what JavaScript will do with that `CString` once we give
a pointer to it, it cannot make any assumptions. The way Rust represents these unmanaged
references is with raw pointers.

So we create a `CString` and we look in the documentation for a way to get a raw pointer
from it. The conveniently named method `into_raw` does exactly that, and it returns a
`*mut std::os::raw::c_char`, so I just took that and put it as the function's return type
without giving much of a second thought. There's probably something to be learned from that,
but it'll have to wait for another day.

In order to compile it, we use:

```bash
rustc +nightly give_me_a_string.rs --crate-type=cdylib --target wasm32-unknown-unknown -C opt-level=3
```

There's something new here: `-C opt-level=3`. This flag is used by the compiler to know
how much it should optimize the resulting code. Higher optimization levels generate more
performant binaries, at the cost of compilation time. That being said, that's not the reason
I added that to the compilation command. For some reason I got into a bunch of weird
internal errors on the JavaScript side if I let the default optimization level take over.
After looking at working examples online I noticed this was what I was missing, and once I
added everything started working as it should. I have yet to dig deeper into this to know
exactly why I need to set the `opt-level`.

### JavaScript

Ok, that was a lot to do on Rust, now to JavaScript. We start pretty much the same as in the
first example, fetching the `.wasm` file and instantiating it. Then we take some things out of it:

```js
const {
  give_me_a_string: giveMeAString,
  memory
} = results.instance.exports;
```

First the exported function, nothing too interesting there, I just alias it to camelCase instead
of using Rust's snake_case.

The second thing we need to take out of the exported module is the linear memory. WASM
modules export the memory block they use so that it can be reached out from JavaScript -
this piece of memory contains a buffer which can be read and written and is what enables
communication between JavaScript and Rust. It is where the generated String will actually
end up and be read from. Now let's look at the next two lines:

```js
const ptr = giveMeAString();
const u8Buffer = new Uint8Array(memory.buffer, ptr);
```

First we execute the function exported from Rust. If you inspect the returned value, you'll
see that it's actually a number - the index where the String starts in the memory buffer.
Next, we'll need to read the String from memory, starting at `ptr` and ending whenever we
find the 0 byte.

`memory.buffer` contains an instance of [`ArrayBuffer`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer),
a generic data structure for binary data which has to be read using the appropriate view.
Since we'll look for a 0 _byte_, we'll take 8 bits at a time. We don't care about the sign
so we can use `Uint8Array` as a view. Also, conveniently, the second parameter to `Uint8Array`'s
constructor is the starting point - where do we want to start reading the underlying buffer from.
We can pass `ptr` in there and forget about that.

Now - so far we have the buffer and the starting point. We'll create another view into the buffer
that knows about the _length_ of the array. In order to do that we'll create a new `Uint8Array`
using `Uint8Array.from` that can take a generator function and be built out of the generated values.
Our generator function looks like this:

```js
function* collectCString(buffer) {
  let ptr = 0;

  while (buffer[ptr] !== 0) {
    if (buffer[ptr] === undefined) {
      throw new Error("Tried to read undef mem");
    }

    yield buffer[ptr];
    ptr += 1;
  }
}
```

It will `yield` new elements until we find that 0 byte. Since `buffer` will be a `Uint8Array`,
we know a 0 byte will be read as the number `0`. Here `ptr` starts at `0` - since we already
initialized the `buffer` then here we can think of `ptr` as in the context of the string we are
discovering. In order to use this function we do:

```js
const buffer = Uint8Array.from(collectCString(u8Buffer));
```

And that will give us in `buffer` a `Uint8Array` which contains the whole string we got from
Rust. In order to decode it, we'll turn to [`TextDecoder`](https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder/TextDecoder).
We'll initialize an `UTF-8` decoder and give it our buffer view for it to generate a string from:

```js
const utf8Decoder = new TextDecoder('UTF-8');
const string = utf8Decoder.decode(buffer);
```

And that's it! Now `string` contains the full string returned from Rust.

## Next steps

The way I wrote it there's no memory management at all - the memory used by the string
probably leaks and is never claimed again. I haven't done any research on this but I've
seen the usage of alloc and dealloc functions in the example from HelloRust. I ignored
it on purpose since I wanted to keep this example as minimal as possible so I could understand
it better and make sure I have the bare minimum for this to work, but I'll need to take
a look at memory management eventually.

## Lessons learned

- Simply exporting a function that returns `String` won't do - it will return `undefined`
  when executed. Not surprising, since WASM only has 4 numeric types available to it, but
  I wanted to see what happens anyway.
- **Optimization flags matter**. For some reason, the default compilation level doesn't
  work very well. For instance, calling the `alloc` function ends up on a `function
  signature mismatch` error down the road. I would have to research more into this to
  figure out exactly what's going on, but for now I'm going with adding `-C opt-level=3`
  to `rustc`.

## Open questions

- Why is it necessary to set the `opt-level` to something higher than the default? Shouldn't
  the different levels be functionally equivalent? (besides speed gained from optimizations
  and debug information lost when compiling on higher optimization levels)

## Links

- [(StackOverflow) How to return a string (or similar) from Rust in Webassembly?](https://stackoverflow.com/questions/47529643/how-to-return-a-string-or-similar-from-rust-in-webassembly)
- [HelloRust demos - SHA1](https://www.hellorust.com/demos/sha1/index.html)
- [Rust's FFI](https://doc.rust-lang.org/book/first-edition/ffi.html)
- [`TextDecoder`](https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder/TextDecoder)