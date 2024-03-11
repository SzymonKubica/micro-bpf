# Proposed use cases for long running eBPF VMs in RIOT
## First scenario - long running programs for collecting data + short ones for serving requests
-> one VM executes a long running program that reads data from the DHT
   sensor and writes it into the global storage

-> another VM accepts CoAP requests to read the latest values of temp/hum
   and returns those by writing directly into the packet data

-> another 'privileged' VM has access to GPIO and ajusts the let status based
   on the latest temperature values. It can also trigger an alarm

## Second scenario - hook-style workflow with programs triggered by specific events in the main application

## Things needed to make it work:
-> support for at least 3 VMs executing in parallel (think about dynamic spawning)
-> support bpf_store_global and ensure proper synchronisation
   -> this requires a rewrite of the helper architecture in rbpf (not necessarily, can call into the existing global impl)

-> allow for killing / spawning VMs from the level of RIOT shell

