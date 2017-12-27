# Text manipulation

This should be the last step before I get to try all of this on rlox. While a REPL
would be more complex, on the first iteration I'm happy with thinking of it as a text
mapping from source code to whatever comes out of standard output. Thinking about it
that way, I already know how to pass strings from Rust to JavaScript from the previous
example, I just need a way to feed strings from JavaScript into Rust. To try it,
I'll build a function that can take a string (technically, a pointer to one), lowercase it
and return a pointer to the lowercase string.

## How to run the example
- Serve this directory (e.g. with `python -m SimpleHTTPServer 8081`).
- Go to `localhost:8081` and look at the console.

## How it works

Just as in the previous example Rust had to return a pointer to where a string started
in order to "give it" to JavaScript, in this case Rust will receive a pointer _from_
JavaScript in order to read a String from it. The process, from the JavaScript side,
is similar to what we did on the previous example but backwards - encode the string
to an `ArrayBuffer` using `TextEncoder`, write it to the shared memory starting at a
given pointer, add a 0 byte at the end and return a pointer to where the string starts.
One important details is that we have to make sure we _allocate_ memory for that string
before storing it. I created a helper function that does all that:

```js
function storeStringInBuffer(str, linearMemory, alloc) {
  const utf8Encoder = new TextEncoder("UTF-8");
  let stringBuffer = utf8Encoder.encode(str);
  let len = stringBuffer.length;
  let ptr = alloc(len+1);

  let memory = new Uint8Array(linearMemory.buffer, ptr);
  for (i = 0; i < len; i++) {
    memory[i] = stringBuffer[i];
  }

  memory[len] = 0;

  return ptr;
}
```

`alloc` is a parameter here on purpose - I wanted to make it abstract from the point
of view of this function how the allocation is being done: it's another function exported
from Rust:

```rust
#[no_mangle]
pub fn alloc(size: usize) -> *const c_void {
    let buf = Vec::with_capacity(size);
    let ptr = buf.as_ptr();
    mem::forget(buf);
    ptr
}
```

Memory allocation is not done manually in Rust - when you "instantiate" a struct Rust
will take care of allocating the corresponding memory for it, and it will take care of
freeing that memory when the reference to that struct is no longer valid. That means
we have to jump through a couple of hoops in order to allocate an arbitrary amount of
memory and give it to JavaScript without freeing it. First, we'll create a `Vec` with
the given capacity. This will make Rust allocate exactly that capacity:

```rust
let buf = Vec::with_capacity(size);
```

We'll need to return a raw pointer again, so we look itno `Vec`'s documentation to find
the proper method:

```rust
let ptr = buf.as_ptr();
```

And then we need to tell Rust that it no longer needs to care about that memory it just
allocated. This will be JavaScript's responsibility now, but Rust should not reallocate
the same address for some other purpose. We'll use [`mem::forget`](https://doc.rust-lang.org/std/mem/fn.forget.html#use-case-3)
for that:

```rust
mem::forget(buf);
```

After that we just return the `ptr`. So what's with the `c_void` return type? According
to Rust's documentation it's the way for Rust to represent what in C would be a void
pointer - a pointer that doesn't point to any specific data type. Since we're allocating
memory without any specific purpose here, that seems to make sense.

If we look back to the JavaScript code, the pointer that Rust returns from `alloc` is
what is being used as the starting point in order to store the given string. Once we
stored the string in memory, we feed it to the `lowercase` function exported from Rust:

```js
const input = storeStringInBuffer('SOMETHING', memory, alloc);
const result = readStringFromBuffer(lowercase(input), memory);
```

Again, that function will receive a pointer and return a pointer:

```rust
#[no_mangle]
pub fn lowercase(data: *const c_char) -> *const c_char {
    let incoming_str;

    unsafe {
        incoming_str = CStr::from_ptr(data).to_str().unwrap().to_owned();
    }

    let lowercased = CString::new(incoming_str.to_lowercase()).unwrap();
    lowercased.into_raw()
}
```

`CStr` is the type used in Rust to reference C-type strings got from external sources.
Constructing a `CStr` using `CStr::from_ptr` is `unsafe`, since Rust cannot know, for
instance, the lifetime of the data pointed y the `data` pointer. One way we can get
around this is to make an owned copy of the string pointed at by `data` and use that
outside of our `unsafe` block. After that it's similar to the previous example -
we convert the `String` to a `CString` and return a raw pointer to it. On the JavaScript
side we read starting from that pointer, decode the string and there we go!

## Links

- [Using `mem::forget` to transfer ownership to foreign code](https://doc.rust-lang.org/std/mem/fn.forget.html#use-case-3)
- [`CStr`](https://doc.rust-lang.org/std/ffi/struct.CStr.html)