# WeSAD
**WEbhook Status and Alerting Daemon** is a small tool designed to provide scheduled updates and ad-hoc alerts to webhook targets. 

I made it for my own home server so that across various services and scripts running independently from one another, I could have a single point of contact to send statuses and alerts without having to rewrite the logic in every application.

The library in this crate provides the daemon service, as well as a minimal public API for sending HTTP updates to the daemon. This is done to minimize the code required for sending updates to the daemon from external scripts.

In cases where importing this crate is undesired or not feasible (e.g. some embedded devices), under the hood the daemon is listening for HTTP requests, so this is a perfectly valid way to send updates.

*Example screenshot coming soon to a repo near you!*

## Architecture

The service is currently designed and functions on my particular Linux distribution (x86 CachyOS), however some small steps have been taken towards eventual cross-compatibility. If this is something you'd like to see, please let me know!

The service consumes any number user-defined webhook templates and configuration files to send webhook updates.