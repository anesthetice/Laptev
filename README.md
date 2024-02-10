### Laptev

A collection of scripts and applications meant to capture, process, and view videos captured when motion is detected on a raspberry pi

### laptev-host

* laptev-host : an http server that deals with the client's requets
  * build using rust and mainly the axum crate
  * decent logging using the tracing crate
  * key exchange for an AES-GCM-SIV cipher using Diffie-Hellman with and x25519
* motioncapture.py : a python script that continuously captures and encodes videos when motion is detected
  * dynamic threshold for motion detection
  * good guardrails against bloated videos
* launch.sh : a bash script to simplify running everything

#### installation for a raspbery-pi

1. download a pre-compiled binary for the raspberry pi or compile laptev-host yourself (I recommend using "cross" to cross-compile to a raspberry pi instead of actually compiling this on a raspberry pi)
2. on the raspbery pi, run the following: mkdir -p $HOME/Laptev/data
3. copy over motioncapture.py, launch.sh, and laptev-host (binary executable) to ~/Laptev
4. run the following: chmod +x launch.sh motioncapture.py laptev-host
5. follow the instructions found inside launch.sh (run: cat launch.sh)

### laptev-client

* laptev-client : an application built in rust using the iced crate to communicate with laptev-host

#### installation

1. download a pre-compiled binary for your preffered system of compile it yourself (cargo build --release)

### usage

When launching laptev-host for the first time, a configuration file will be automatically generated, it will look something like this:

file: laptev.config
```
{
  "port": 12675,
  "password": [1,213,114,168,67,6,14,135,...,90],
  "client_expiration_time": 1800,
  "file_expiration_time": 259200
}
```

let's break each element down:
1. port: the port (u16) where the server will listen on, 12675 is the default
2. password: the server's password to authenticate clients
3. client_expiration_time: after how long will clients be considered invalid
4. file_expiration_time: for how long are .mp4 and .jpg files inside $HOME/Laptev/data are kept

Same thing for laptev-client, a configuration file will also be created on launch:

file: laptev.config
```
{
  "default_address": "127.0.0.1:12675",
  "size": 25,
  "skip": 0,
  "local_offset": [
    0,
    0,
    0
  ],
  "entries": {
    "127.0.0.1": []
  }
 }
```

Let's break each element down as well:
1. default_address : the default address shown when launching the client
2. size: the amount of thumbnails the server sends when syncing
3. skip: the amount of thumbnails the server skips when syncing, i.e. to view older thumbnails
4. local_offset: your local UtcOffset, "[hours, minutes, seconds]"
5. entries: a list of servers the client knows and their associated password

In summary, just add the host's ip address and password to the client's config before attempting to sync with the server
