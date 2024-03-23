## Interesting things to think about

1. Do we need thread-local bpf storage?

Currently the progarm can read/write global storage but there is no notion
of local storage associated with a given VM worker.
