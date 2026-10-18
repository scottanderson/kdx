# Voice messages

For hex and field notation, see [Conventions](../README.md#conventions).

Message types 1000 to 1003 carry voice between two clients on a DCC connection.
[dcc.md](../dcc.md#voice) gives the sequence.

## <a id="m1000"></a>1000 Start Voice

Initiator: Client (to client). <!-- [S `FUN_0808f1ac`] -->
Only the client handles it, and only on DCC. <!-- [S `FUN_08094074`] -->

No payload.

The caller sends it on the secondary connection, after the login.

## <a id="m1001"></a>1001 Start Voice (R)

Initiator: Client (to client). <!-- [S `FUN_08094074`] -->

This is the reply to [1000 Start Voice](#m1000):

Struct `StartVoiceReply` (4 bytes):<a id="sStartVoiceReply"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 4 | u32 | `result` | See below. |

The value is:

- 0x00640007 if the peer's Voice Controls window is not open.
- 0 if it is open. The peer's own [1002 Set Voice Format](#m1002) follows.

If the value is 0, the caller sends its own 1002. If not, it shows "The remote
user does not have his/her Voice Controls window open." and closes the
connection. <!-- [S `FUN_08094218`] -->

## <a id="m1002"></a>1002 Set Voice Format

Initiator: Client (to client). <!-- [S `FUN_08093ae0`, `FUN_080943a0`] -->

Struct `VoiceFormat` (24 bytes):<a id="sVoiceFormat"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | 12 | u8[12] | `ignored` | 0 as sent. |
| 0x0c | 4 | u32 | `sample_rate` | Hz |
| 0x10 | 4 | char[4] | `codec` | `ADPC` |
| 0x14 | 2 | u16 | `buffer` | In ms. The receiver divides it by 10, and clamps it to 35 to 500. |
| 0x16 | 1 | u8 | `channels` | 1 |
| 0x17 | 1 | u8 | `bits_per_sample` | 16 |

Both sides sent 11025 Hz (0x2b11), `ADPC`, 500 ms (0x01f4), 1 channel, and 16
bits. <!-- [O `captures/linux-dcc-voice-failed.pcapng` frames 136, 138] -->

## <a id="m1003"></a>1003 Voice Data

Initiator: Client (to client). <!-- [S `FUN_08093670`, `FUN_0809455c`, `FUN_080e15e4`] -->

Struct `VoiceData` (dynamic):<a id="sVoiceData"></a>

| Off | Size | Type | Field | Notes |
|---|---|---|---|---|
| 0x00 | rest | u8[] | `audio` | IMA ADPCM |

The payload is IMA ADPCM, with two samples in each byte, high nibble first. The
decoder starts again from zero for each message. It decodes to signed 16-bit
samples, big-endian, four times the size of the payload. While more than
0x100000 bytes wait in its send queue, the sender drops audio.
