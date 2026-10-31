# WeSAD
**WEbhook Status and Alerting Daemon** is a small tool designed to provide scheduled updates and ad-hoc alerts to webhook targets. 

I made it for my own home server so that across various services and scripts running independently from one another, I could have a single point of contact to send statuses and alerts without having to rewrite that logic in every application. Furthermore, by daemoninizing and centralizing the process, I can combine data from various tasks that run on different intervals without requiring those processes to agree on a shared status update format or timeframe.

The library in this crate provides the core daemon service, as well as a minimal public API for sending updates to the running daemon. This was included to further minimize the code required for sending updates to the daemon from external scripts. However, the daemon service listens for updates via HTTP, so it is intended that you can still send status updates in situations where importing this crate is undesired or not feasible, *e.g.* some embedded devices, using other languages, etc...

*Example screenshot coming soon to a repo near you!*


## OS & Webhook Support

WeSAD is currently designed and functions on my particular Linux distribution (x86 CachyOS), however some small steps have been taken towards eventual cross-compatibility. If this is something you'd like to see, please let me know!

The service is designed primarily with **Discord**'s webhooks in mind, though more configuration options are planned to more robustly support various endpoints, such as:
- Optional user-defined HTTP method & headers
- Optional user-defined

## Architecture
The user provides configuration in the form of paired `.conf` and `.json` files in the `webhooks.d` directory. Each pair of files must have the same name to be associated with one another. The `.json` file contains the raw webhook content, including all placeholder values, called `tags`, which should be replaced with live data for the udpate. 

The tags are identified with double dollar-signs around the text, for example `The animal is $$animal$$.`

On initialization, the service will read all pairs of configuration files and store the webhooks and tags data in a small sqlite database, with all tags initialized to the value "*empty*".

The service then splits into two main threads:
### Updater
The updater will periodically fetch all tag data from the database, replace it into the template provided in the `.json` file, and send the webhook update as 
### Listener