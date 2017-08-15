# MONOLOGUED

Monologued is an implementation of an
[RFC1288](https://tools.ietf.org/html/rfc1288) protocol server in Rust.

That's right.  Monologued is a
[*Finger*](https://en.wikipedia.org/wiki/Finger_protocol) server.

# WHY‽‽‽‽

Because I wanted to learn Rust, and this seemed like a really good idea
at the time.  When I learned that John Carmack still updates his
<code>.plan</code> file with what he's going to accomplish in the coming
weeks, I thought implementing a .plan server would be a lovely place to
begin learning the ins and outs of Rust.

After much deliberation over using Tokio, I decided to get down to the
basics.  Twenty years ago I was writing servers in C using
<code>select(2)</code>, and I figured before I started having my hand
held with asynchronous programming, I should go down to the basement and
see how it works on bare metal, so imagine my pleasure that the
<code>select(2)</code> implementation for Rust is literally named
<code>[Metal I/O](https://github.com/carllerche/mio)</code>.

This gives me a chance to learn MIO, Inotify in Rust, and writing my own
cache handler.  Which just sounds like a ton of fun, doesn't it?

# Status

Monologued is still very much not working.  Don't even bother
downloading.  It's mostly something to cut my teeth on while I try to
figure out how Rust works.
