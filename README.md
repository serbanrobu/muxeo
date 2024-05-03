# Multiplexer / Demultiplexer for standard error and standard output

The `muxeo` command combines a program's _stderr_ and _stdout_ streams into a
single stream of frames written to _stdout_. The frame has the following
structure:

```
+----------+--Frame---+---------+
| kind: u8 | len: u32 | payload |
+----------+----------+---------+
```

The `demuxeo` command knows how to decode the stream of frames received as
_stdin_ and then writes to both _stderr_ and _stdout_ depending on the frame
kind (err/out).

For example, the following command:

```sh
muxeo -- my-program --opt-1 --opt-2 -- arg-1 arg-2 | demuxeo
```

will produce a result similar to:

```sh
my-program --opt-1 --opt-2 -- arg-1 arg-2
```

One possible use case would be passing a program's stderr and stdout over HTTP.
Here is an example Bun HTTP server:

```ts
import { $ } from "bun";

Bun.serve({
  async fetch() {
    return new Response(
      await $`muxeo -- my-program --opt-1 --opt-2 -- arg-1 arg-2`.arrayBuffer(),
    );
  },
  port: 3000,
});
```

We could then decode the HTTP stream response as follows, redirecting
stderr/stdout if necessary.

```sh
curl --silent -- localhost:3000 | demuxeo
```
