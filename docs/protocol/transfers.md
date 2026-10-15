# Transfer connections

How a download or upload runs on its own connection. The messages it carries
are [412 Download File](messages/files.md#m412) to
[418 Upload File (C)](messages/files.md#m418) and
[900 Open View Display](messages/view-display.md#m900). The server sends
[419 Notify Speed Limit](messages/files.md#m419) on the main connection.

## Connecting

Files move on a second connection to the same port, attached to a user who is
already logged in. **[S `FUN_0805270c`, `FUN_080528c8`, `FUN_08052194`; O
kdx-probe `--attach`]** When [105 Server Info](messages/session.md#m105) names a
file transfer port, the client connects there instead **[S client
`FUN_08052cc0`]**.

1. The client connects and runs the normal handshake with a special login:
   - the login name is 5 bytes: `00`, then the u32 user ID from 105 Server
     Info offset 0x24;
   - the account key is the 32 bytes at 105 Server Info offset 0x00, not a
     password hash.
2. The server accepts it only if that user is logged in, is not itself an
   attached connection, and connects from the same IP address. The attached
   connection gets its own user ID, but it acts with the parent's
   privileges and base folder. The server log calls it a transfer, and on
   close it logs `Transfer Disconnect` **[O]**.
3. On this connection, the server handles only
   [400 Get File List](messages/files.md#m400),
   [402 Create Folder](messages/files.md#m402),
   [412 Download File](messages/files.md#m412),
   [415 Upload File](messages/files.md#m415), and
   [900 Open View Display](messages/view-display.md#m900),
   [904 View Display Mouse](messages/view-display.md#m904),
   [905 View Display Keyboard](messages/view-display.md#m905) (view display). It
   ignores everything else, including [108 Ping](messages/session.md#m108) **[S,
   O]**.
4. One transfer runs at a time per connection. A second 412 Download File or 415
   Upload File during a transfer gets [1 Status](messages/status.md#m1) 0x10003.
   Afterwards the connection takes requests again **[S]**.

## Download

A download is [412 Download File](messages/files.md#m412): the server
replies [413 Download File (R)](messages/files.md#m413), sends one
[414 Download File (D)](messages/files.md#m414) per fork, and writes a
[188 Finished File Download](messages/administration.md#m188) history record.

## Upload

An upload is [415 Upload File](messages/files.md#m415): the server replies
[416 Upload File (R)](messages/files.md#m416), the client sends one
[417 Upload File (D)](messages/files.md#m417) per fork, and the server writes a
[189 Finished File Upload](messages/administration.md#m189) history record and
sends [418 Upload File (C)](messages/files.md#m418).

## Encryption

The stream is encrypted unless the client asks for plain mode (request
offset 0x17 bit 0, which needs privilege bit 65, probably Disable
Encryption **[I]**). Encrypted data is framed exactly like a message: a header
whose length gives the byte count, followed by that many bytes in 16-byte
Twofish blocks. A client can read it with its normal receive path **[O]**.
Plain mode has not been tried.
