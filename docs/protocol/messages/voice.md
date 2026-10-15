# Voice messages

Message types 1000 to 1003: voice between two clients on a DCC connection. The
sequence that uses them is in [dcc.md](../dcc.md#voice).

## <a id="m1000"></a>1000 Start Voice

Initiator: Client (to client). **[S `FUN_0808f1ac`]**
Handled by the client, on DCC only **[S `FUN_08094074`]**.

The caller sends it, empty, on login to the secondary connection.

## <a id="m1001"></a>1001 Start Voice (R)

Initiator: Client (to client). **[S `FUN_08094074`]**

Reply to [1000 Start Voice](#m1000), a u32:

- 0x00640007 if the peer's Voice Controls window is not open;
- 0, followed by the peer's own [1002 Set Voice Format](#m1002), if it is.

On 1001 the caller sends its 1002 if the value is 0. Otherwise it shows "The
remote user does not have his/her Voice Controls window open." and closes
the connection **[S `FUN_08094218`]**.

## <a id="m1002"></a>1002 Set Voice Format

Initiator: Client (to client). **[S `FUN_08093ae0`, `FUN_080943a0`]**

0x18 bytes:

| Off | Size | Field | Notes |
|---|---|---|---|
| 0x00 | 0xc | 0 | |
| 0x0c | 4 | sample rate | Hz |
| 0x10 | 4 | codec | `ADPC` |
| 0x14 | 2 | buffer | ms; the receiver divides by 10 and clamps to 35..500 |
| 0x16 | 1 | channels | 1 |
| 0x17 | 1 | bits per sample | 16 |

Both sides sent 11025 Hz (0x2b11), `ADPC`, 500 ms (0x01f4), 1 channel, and 16
bits (frames 136, 138) **[O]**.

## <a id="m1003"></a>1003 Voice Data

Initiator: Client (to client). **[S `FUN_08093670`, `FUN_0809455c`,
`FUN_080e15e4`]**

The payload is IMA ADPCM, two samples per byte, high nibble first. The
decoder state starts fresh for each message. It decodes to big-endian
signed 16-bit samples, four times the payload size. The sender drops audio
while more than 0x100000 bytes wait in its send queue.
