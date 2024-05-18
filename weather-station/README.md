# Example application: Weather Station

This directory contains an example application which implements a fleet of
eBPF programs that are responsible for executing compartmentalised business
logic of an embedded system responsible for collecting temperature and humidity
data and displaying them to the users.

The list of compartments:
- sensor reading module -> responsible for periodically reading the temperature
  and humidity data and storing it in the shared global storage
- display module -> responsible for logging the temperature data and responding
  to user input
- moving average calculation module -> reads collected data and computes moving
  average
