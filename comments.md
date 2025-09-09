## LL optimizations

**Zero-copy parsing**: read fields directly from the NIC buffer using casts/pointer arithmetic.
**Cache alignment**: NIC buffers are sized and aligned to fit CPU cache lines to minimize L1/L2 cache misses.
**Batching**: grab several descriptors in a burst before accessing them, to amortize overhead.
**Prefetching**: issue CPU prefetch instructions to bring future packets into cache before parsing.
**NUMA awareness**: pin your parsing thread to the same NUMA node as the NIC’s DMA memory.

## Fixlite review

* A single `chrono::NaiveDateTime::parse_from_str` call can take anywhere from a few hundred nanoseconds to several microseconds!!! Using `f64` in HFT is questionable due to unpredictable rounding and other issues. `f64` is often considered the road to bugs and money loss. These types should be removed from the README and examples, so it doesn’t appear that fixlite “recommends” using them in real LL applications.

* Heap allocations must be eliminated. An **arena** could be very useful here, especially one with a large preallocated and possibly pinned (immovable) buffer. Ideally, vector construction would be avoided entirely: **a lazy iterator** could help here.

* The approach in fixlite is quite eager: it takes a message and converts it into a Rust structure, even though not all fields may be needed. That means extra work is being done parsing dates, numbers, etc. Conversions can be deferred until they’re actually needed. Maybe some lazy data structure or lazy accessor methods can help.

* When the set of target fields is known (we have the struct), parsing can stop as soon as those fields are filled.

* Fixlite passes over the input string more than once, but this format seems parsable in a single pass. Using something like memchr crate or custom iter that does the serach by N bytes at a time may speed it up. Note: = may appear inside value (0x01 may also appear inside value in raw data). Replace split by 0x01 or '=' with FixParser (see fixlite_example). FixParser implements try-iter with alternating = and 0x01 (solves = inside value problem).

DONE * Convert tags to `u32` and then `match` instead of matching on `&str` tags.

* Investigate why the fix3 benchmark runs 2x faster even after the optimizations.

