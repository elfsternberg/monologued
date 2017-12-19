Monologued is a server that accepts connections and responds with a
static message.

Main() starts up a single poller and provides it with a number of
servers.  The server share a token pool, and when the token pool runs
out the server can start no more listening connections.  ☐ - Limiter!


