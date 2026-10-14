# kdx-codec

Framing for connections, as `tokio_util::codec` codecs: `HandshakeCodec` reads
and writes the hello and proof, and `SessionCodec` the encrypted messages after
them. `SessionCodec` streams the data of 414 Download File (D) and 417 Upload
File (D) in chunks, so a whole file is never held in memory.

It builds on [kdx-proto](kdx-proto.md) and is used by
[kdx-client](kdx-client.md) and [kdx-tracker](../tracker/kdx-tracker.md).
