# Table of Contents
- [Table of Contents](#table-of-contents)
- [About](#about)
- [Crate](#crate)
  - [OS \& Webhook Support](#os--webhook-support)
  - [Usage](#usage)
  - [Architecture](#architecture)
- [Examples](#examples)
  - [Astley example](#astley-example)
  - [Client API](#client-api)

# About
**WeSAD (WEbhook Status and Alerting Daemon)**, is a small tool built in Rust designed as a centralized service for sending webhook updates and alerts from various disconnected sources.

I made it for my home server to solve a simple problem: Building a highly modifiable discord dashboard for my various, unrelated services running on my server. Of course, there are other solutions capable of doing this, but I wanted to do it myself :)

WeSAD solves this problem by acting as a single recipient for any and all status updates and alerts from my various devices and scripts. It stores all the data it receives, and then periodically refreshes webhook dashboards based on the current fresh data.

WeSAD allows for an indefinite number of concurrent webhooks all with independent data, update intervals, `status` messages which are edited in-place such as for dashboards, or `alert` messages which fire off when requested.

Note - `alert` types are not yet implemented.

# Crate
The crate is split into two main public APIs:
1. The core daemonizable service
2. A minimal public API for sending updates to the running daemon. 

This "sender" API was included to further minimize the code required for sending updates to the daemon from external scripts. [Example]

However, the daemon service listens for updates via HTTP, so it is intended that you can still send status updates in situations where importing this crate is undesired or not feasible, *e.g.* some embedded devices, using other languages, etc...

*Example screenshot coming soon to a repo near you!*


## OS & Webhook Support

WeSAD is currently designed for, and functions on, my particular Linux distribution (x86 CachyOS). Some small steps have been taken towards eventual cross-compatibility, but have not been tested. If this is something you'd like to see, please let me know!

The service is designed primarily with **Discord**'s webhooks in mind, though more configuration options are planned to more robustly support various endpoints, such as allowing the user to specify the HTTP method & headers.

## Usage
The user provides configuration in the form of name-paired `.conf` and `.json` files in the project's `webhooks.d` directory, located in:
| OS      | Config Path                                                  |
|---------|--------------------------------------------------------------|
| Linux   | `~/alice/.config/wesad`                                        |
| Mac     | `/Users/Alice/Library/Application Support/com.beepdodev.wesad` |
| Windows | `C:\Users\Alice\AppData\Roaming\wesad\`                        |

If these locations do not appear, run the program once first with the `-i` or `--init` shout (Not yet implemented).

Each pair of `.conf` and `.json` files must have the same name to be associated with one another, and both **must** be present for the program to read them e.g. `astley.conf` and `astley.json`.
- The `.conf` file contains header information about the webhook; 
  - The endpoint url
  - Update type (`status` vs `alert`)
  - Update interval
- The `.json` file contains the full raw HTTP body for the webhook request, including all placeholder values, called `tags`.

The tags are identified with double dollar-signs around the text, and are used for any piece of data in the template that should be updated by an external program, for example `Never gonna $$give$$.`

By default, WeSAD will create a webserver on `localhost:22000`, and you can then send new placeholder data as a POST to the `/status` api endpoint. The body of the request should be structured as follows:




## Architecture
The program uses an [sqlite](https://docs.rs/sqlite/latest/sqlite/) database to store all persistent webhook and placeholder data. On startup, it will build paths to the OS's project directories using the [directories](https://docs.rs/directories/latest/directories/index.html) crate. 

A long-running `listener` thread uses [tokio](https://docs.rs/tokio/latest/tokio/) to build an asynchronous TCP webserver and listen for updates which get pushed into the [sqlite](https://docs.rs/sqlite/latest/sqlite/) database. Simultaneously, the long-lived `updater` thread also uses [tokio](https://docs.rs/tokio/latest/tokio/) for sending HTTP requests to the webhook target endpoints based on the current values in the database.

Each webhook configuration file contains its own information about update type, and interval. Within the `updater` thread, the program dynamically sleeps after every update cycle until the next udpate is due, so as to reduce impact on compute resources. The exact timing of the `updater` is likely quite unreliable, and your prescribed interval should be considered more an estimate than an exact value.


# Examples
## Astley example
In the `example_dirs` directory of this repo you can find example configs `astley.conf` and `astley.json`. If you make sure to replace the `endpoint` in `astley.conf` with a valid discord webhook, you can test things out right away: 
- Compile the binary and run it once with the `-i` or `--init` shout.
  - This will run the service in "initialization" mode which will create the required folders on your computer then exit.
- Move the `astley.conf` and `astley.json` files into your OS's `webhooks.d` folder (see [here](#os--webhook-support) for that location)
- Run the program again, this time with no additional command line arguments.
  - This will open attached to your terminal for easy log viewing and shutdown. 
- Download the `example-update.json` file from `example_dirs`. This is a fully formatted tag payload designed to work with the default `astley` examples.
- Open or otherwise navigate your terminal to the location of the `example-update.json` you just downloaded
- Run the (Linux) command `curl -i --header "Content-Type: application/json" --data "$(cat example-update.json)" "localhost:22000/status"`

In your WeSAD terminal, you should see an INFO log acknowledgement of the POST request.

The `astley.conf` example is set to update every `7` seconds, so wait, and soon enough you should see the *empty* tags replaced with the data from `example-update.json`!

## Client API
Sending a tag update using the provided client api is simple:
```rust
use wesad::{
    self,
    TagValue
}

let client = wesad::Client::new().unwrap();
// or, for a specific ip:port, use
// let client = wesad::Client::new_from_ip("192.168.0.123:12345");

let my_tag_data = Vec::new();
my_tag_data.push(TagValue::new("give", "give you up"));
my_tag_data.push(TagValue::new("let", "let you down"));
my_tag_data.push(TagValue::new("run", "run around and desert you"));

client.send_tags("astley", my_tag_data).unwrap();
```