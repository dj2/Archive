# Archive
Archive helps you maintain a set of notes and associated assets.

This is mostly just the skeleton at this point, see the [TODO](TODO.md) document
for a list of things that are planned.

Note, the server does not provide authentication and is designed to be used
either listening on `localhost` or behind something like
[Tailscale](https//tailscale.com). Basically, accept connections and data from
trusted hosts. It has not been tested, and doesn't provide much protection,
against untrusted hosts.
