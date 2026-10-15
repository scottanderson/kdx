# kdx-client

An async library on tokio for writing KDX clients.

- `Connection` runs the handshake and sends raw session messages through
  [kdx-codec](kdx-codec.md).
- `Client` logs in and offers typed requests on top of it.
- `Tracker` does the same for a tracker.

The [kdx](../cli/kdx.md) command-line client is built on it. The messages it
sends are in [the protocol reference](../protocol/README.md).
