Symbiotic Clipboard
===================
Share the clipboard between various computers.

Features
--------
* Secure connections only.
* Limit the maximum size of clipboard data that will be sent for each computer.
* Make a computer only a receiver or sender of clipboard data.
* Filter clipboard content by mime type.

Data Format Support
-------------------
* Plain text and various variants like HTML and such.
* Images are supported, although on Windows only formats supported by
  [image][1] are supported.
* Any format as long as the platform dependant format name is a MIME type.

Platforms
---------
The supported plaftorms so far are Linux and Windows, in theory it should work
on BSDs using X11 as long as Rust compiles on there.

OS X is in the plans, but it would be extremely painful to implement, so
whatever, contributions welcome tho.

Security
--------
By default client certificates won't be verified and a server certificate will
be automatically generated.

To make it verify certificates you have to generate the `cert.pem` and
`key.pem` for every machine and copy the `cert.pem` of every machine to every
machine and add the `cert` field to each `connection` in the configuration
file.

Usage
-----

```
Usage: symbiotic-clipboard (-c PATH | --config PATH)
       symbiotic-clipboard [options] <peers>...
       symbiotic-clipboard --help

Options:
  -h, --help         Show this message.
  -b, --bind IP      IP to bind on (default 0.0.0.0).
  -p, --port PORT    Port to listen on (default 23421).
  -c, --config PATH  Path to the config file.
  
  -l, --limit SIZE   Maximum size of the clipboard data to send.
  -f, --filter MIME  List of ':' separated mime types to ignore.
  
  -i, --incoming     Only receive clipboard changes.
  -o, --outgoing     Only send clipboard changes.
```

You can find an example configuration file [here][2].

[1]: https://github.com/PistonDevelopers/image
[2]: https://github.com/meh/symbiotic/blob/clipboard/example.toml
