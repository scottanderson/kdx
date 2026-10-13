//! GConsole messages: a peer draws in a window of the client, which reports
//! the keys and clicks in it (`docs/protocol/dcc.md`, GConsole).

use super::{need, u16_at, u32_at, DecodeError, EncodeError};

/// A colour: a transparency byte (0 opaque, 0xff invisible), then red, green, and blue.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Color(pub u32);

impl Color {
    /// Red, green, and blue.
    pub const fn rgb(self) -> [u8; 3] {
        let [_, r, g, b] = self.0.to_be_bytes();
        [r, g, b]
    }

    /// The transparency byte.
    pub const fn transparency(self) -> u8 {
        self.0.to_be_bytes()[0]
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [r, g, b] = self.rgb();
        write!(f, "#{r:02x}{g:02x}{b:02x}")?;
        match self.transparency() {
            0 => Ok(()),
            t => write!(f, "/{t:02x}"),
        }
    }
}

/// A point in an object.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Point {
    /// X.
    pub x: i16,
    /// Y.
    pub y: i16,
}

/// A rectangle: left, top, right, and bottom, right and bottom exclusive.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rect {
    /// Left.
    pub left: i16,
    /// Top.
    pub top: i16,
    /// Right.
    pub right: i16,
    /// Bottom.
    pub bottom: i16,
}

fn i16_at(buf: &[u8], at: usize) -> i16 {
    u16_at(buf, at) as i16
}

fn point_at(buf: &[u8], at: usize) -> Point {
    Point {
        x: i16_at(buf, at),
        y: i16_at(buf, at + 2),
    }
}

fn rect_at(buf: &[u8], at: usize) -> Rect {
    Rect {
        left: i16_at(buf, at),
        top: i16_at(buf, at + 2),
        right: i16_at(buf, at + 4),
        bottom: i16_at(buf, at + 6),
    }
}

fn color_at(buf: &[u8], at: usize) -> Color {
    Color(u32_at(buf, at))
}

fn put_point(out: &mut Vec<u8>, p: Point) {
    out.extend_from_slice(&p.x.to_be_bytes());
    out.extend_from_slice(&p.y.to_be_bytes());
}

fn put_rect(out: &mut Vec<u8>, r: Rect) {
    for v in [r.left, r.top, r.right, r.bottom] {
        out.extend_from_slice(&v.to_be_bytes());
    }
}

/// 4000 Open Window: the size and colour of the client's window.
/// Provisional name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpenWindow {
    /// Width, 0x40..=0x1000.
    pub width: u16,
    /// Height, 0x10..=0x1000.
    pub height: u16,
    /// The window's colour.
    pub color: Color,
    /// Bytes 0x08..0x18, which the client doesn't read.
    pub rest: [u8; 0x10],
}

impl OpenWindow {
    /// Shortest payload the client accepts.
    pub const LEN: usize = 0x18;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::LEN)?;
        Ok(Self {
            width: u16_at(buf, 0),
            height: u16_at(buf, 2),
            color: color_at(buf, 4),
            rest: buf[8..Self::LEN].try_into().unwrap(),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::LEN);
        out.extend_from_slice(&self.width.to_be_bytes());
        out.extend_from_slice(&self.height.to_be_bytes());
        out.extend_from_slice(&self.color.0.to_be_bytes());
        out.extend_from_slice(&self.rest);
        out
    }
}

/// One record of 4001 Draw.
///
/// Every record but [`Create`](Self::Create) names the object it draws in.
/// Fields named by offset are passed on by the client with no known effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrawRecord {
    /// Kind 2: creates an object at `rect` in the window.
    Create {
        /// Where the object sits in the window; also its size.
        rect: Rect,
        /// Bytes 0x08..0x0c.
        value_08: u32,
        /// Object ID.
        id: u16,
        /// Flags; the object shows only with bit 8 set.
        flags: u16,
        /// Bytes 0x10..0x12.
        value_10: u16,
        /// Object type; the client knows only 1, an image.
        object_type: u8,
        /// Byte 0x13.
        value_13: u8,
    },
    /// Kind 3: deletes an object.
    Delete {
        /// Object ID.
        id: u16,
    },
    /// Kind 4: moves an object's top left corner to `to`.
    Move {
        /// Object ID.
        id: u16,
        /// The new position in the window.
        to: Point,
    },
    /// Kind 100: fills a rectangle.
    Fill {
        /// Object ID.
        id: u16,
        /// Bytes 0x02..0x04.
        value_02: u16,
        /// Rectangle.
        rect: Rect,
        /// Colour.
        color: Color,
    },
    /// Kind 101: fills a rectangle with a gradient from `from` on the left to `to` on the right.
    Gradient {
        /// Object ID.
        id: u16,
        /// Bytes 0x02..0x04; 1 in the only capture.
        value_02: u16,
        /// Rectangle.
        rect: Rect,
        /// Colour at the left.
        from: Color,
        /// Colour at the right.
        to: Color,
    },
    /// Kind 102: draws a rectangle's outline.
    Frame {
        /// Object ID.
        id: u16,
        /// Line width; 0 draws nothing.
        width: u16,
        /// Rectangle.
        rect: Rect,
        /// Colour.
        color: Color,
    },
    /// Kind 103: sets one pixel.
    Pixel {
        /// Object ID.
        id: u16,
        /// Bytes 0x02..0x04.
        value_02: u16,
        /// The pixel.
        at: Point,
        /// Colour.
        color: Color,
    },
    /// Kind 104: draws a line.
    Line {
        /// Object ID.
        id: u16,
        /// Byte 0x02.
        value_02: u8,
        /// Line width.
        width: u8,
        /// Start.
        from: Point,
        /// End.
        to: Point,
        /// Colour.
        color: Color,
    },
    /// Kinds 105 and 106: draws text.
    Text {
        /// 105 or 106; no difference shows.
        kind: u16,
        /// Object ID.
        id: u16,
        /// Bytes 0x02..0x04.
        value_02: u16,
        /// Where the text goes.
        rect: Rect,
        /// Bytes 0x0c..0x14.
        value_0c: [u8; 8],
        /// The text, to the end of the record.
        text: Vec<u8>,
    },
    /// Kind 107: fills a polygon.
    Polygon {
        /// Object ID.
        id: u16,
        /// Bytes 0x02..0x04.
        value_02: u16,
        /// Colour.
        color: Color,
        /// Point at 0x08; (0, 0) in the only capture.
        origin: Point,
        /// Corners; at least three.
        points: Vec<Point>,
    },
    /// Kind 108: flood-fills the area around a point.
    FloodFill {
        /// Object ID.
        id: u16,
        /// Bytes 0x02..0x04.
        value_02: u16,
        /// Where the fill starts.
        at: Point,
        /// Colour.
        color: Color,
    },
    /// Kind 109: copies part of an object within itself.
    Copy {
        /// Object ID.
        id: u16,
        /// Bytes 0x02..0x04.
        value_02: u16,
        /// Destination top left.
        to: Point,
        /// Source top left.
        from: Point,
        /// Width.
        width: u16,
        /// Height.
        height: u16,
    },
    /// A kind the client ignores, or a record too short for its kind.
    Other {
        /// Kind.
        kind: u16,
        /// Body.
        body: Vec<u8>,
    },
}

impl DrawRecord {
    /// The record's kind.
    pub const fn kind(&self) -> u16 {
        match self {
            Self::Create { .. } => 2,
            Self::Delete { .. } => 3,
            Self::Move { .. } => 4,
            Self::Fill { .. } => 100,
            Self::Gradient { .. } => 101,
            Self::Frame { .. } => 102,
            Self::Pixel { .. } => 103,
            Self::Line { .. } => 104,
            Self::Polygon { .. } => 107,
            Self::FloodFill { .. } => 108,
            Self::Copy { .. } => 109,
            Self::Text { kind, .. } | Self::Other { kind, .. } => *kind,
        }
    }

    /// Decodes a body of `kind`, keeping it whole when it's shorter than the client requires.
    fn decode(kind: u16, b: &[u8]) -> Self {
        let id = || u16_at(b, 0);
        let fits = |len| b.len() >= len;
        match kind {
            2 if fits(0x14) => Self::Create {
                rect: rect_at(b, 0),
                value_08: u32_at(b, 8),
                id: u16_at(b, 0x0c),
                flags: u16_at(b, 0x0e),
                value_10: u16_at(b, 0x10),
                object_type: b[0x12],
                value_13: b[0x13],
            },
            3 if fits(2) => Self::Delete { id: id() },
            4 if fits(6) => Self::Move {
                id: id(),
                to: point_at(b, 2),
            },
            100 if fits(0x10) => Self::Fill {
                id: id(),
                value_02: u16_at(b, 2),
                rect: rect_at(b, 4),
                color: color_at(b, 0x0c),
            },
            101 if fits(0x14) => Self::Gradient {
                id: id(),
                value_02: u16_at(b, 2),
                rect: rect_at(b, 4),
                from: color_at(b, 0x0c),
                to: color_at(b, 0x10),
            },
            102 if fits(0x10) => Self::Frame {
                id: id(),
                width: u16_at(b, 2),
                rect: rect_at(b, 4),
                color: color_at(b, 0x0c),
            },
            103 if fits(0x0c) => Self::Pixel {
                id: id(),
                value_02: u16_at(b, 2),
                at: point_at(b, 4),
                color: color_at(b, 8),
            },
            104 if fits(0x10) => Self::Line {
                id: id(),
                value_02: b[2],
                width: b[3],
                from: point_at(b, 4),
                to: point_at(b, 8),
                color: color_at(b, 0x0c),
            },
            105 | 106 if fits(0x15) => Self::Text {
                kind,
                id: id(),
                value_02: u16_at(b, 2),
                rect: rect_at(b, 4),
                value_0c: b[0x0c..0x14].try_into().unwrap(),
                text: b[0x14..].to_vec(),
            },
            107 if fits(0x18) && (b.len() - 0x0c).is_multiple_of(4) => Self::Polygon {
                id: id(),
                value_02: u16_at(b, 2),
                color: color_at(b, 4),
                origin: point_at(b, 8),
                points: b[0x0c..].chunks(4).map(|p| point_at(p, 0)).collect(),
            },
            108 if fits(0x0c) => Self::FloodFill {
                id: id(),
                value_02: u16_at(b, 2),
                at: point_at(b, 4),
                color: color_at(b, 8),
            },
            109 if fits(0x10) => Self::Copy {
                id: id(),
                value_02: u16_at(b, 2),
                to: point_at(b, 4),
                from: point_at(b, 8),
                width: u16_at(b, 0x0c),
                height: u16_at(b, 0x0e),
            },
            _ => Self::Other {
                kind,
                body: b.to_vec(),
            },
        }
    }

    /// Encodes the body, without the record header.
    fn encode_body(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let head = |out: &mut Vec<u8>, id: u16, value: u16| {
            out.extend_from_slice(&id.to_be_bytes());
            out.extend_from_slice(&value.to_be_bytes());
        };
        match self {
            Self::Create {
                rect,
                value_08,
                id,
                flags,
                value_10,
                object_type,
                value_13,
            } => {
                put_rect(&mut out, *rect);
                out.extend_from_slice(&value_08.to_be_bytes());
                out.extend_from_slice(&id.to_be_bytes());
                out.extend_from_slice(&flags.to_be_bytes());
                out.extend_from_slice(&value_10.to_be_bytes());
                out.extend_from_slice(&[*object_type, *value_13]);
            }
            Self::Delete { id } => out.extend_from_slice(&id.to_be_bytes()),
            Self::Move { id, to } => {
                out.extend_from_slice(&id.to_be_bytes());
                put_point(&mut out, *to);
            }
            Self::Fill {
                id,
                value_02,
                rect,
                color,
            }
            | Self::Frame {
                id,
                width: value_02,
                rect,
                color,
            } => {
                head(&mut out, *id, *value_02);
                put_rect(&mut out, *rect);
                out.extend_from_slice(&color.0.to_be_bytes());
            }
            Self::Gradient {
                id,
                value_02,
                rect,
                from,
                to,
            } => {
                head(&mut out, *id, *value_02);
                put_rect(&mut out, *rect);
                out.extend_from_slice(&from.0.to_be_bytes());
                out.extend_from_slice(&to.0.to_be_bytes());
            }
            Self::Pixel {
                id,
                value_02,
                at,
                color,
            }
            | Self::FloodFill {
                id,
                value_02,
                at,
                color,
            } => {
                head(&mut out, *id, *value_02);
                put_point(&mut out, *at);
                out.extend_from_slice(&color.0.to_be_bytes());
            }
            Self::Line {
                id,
                value_02,
                width,
                from,
                to,
                color,
            } => {
                out.extend_from_slice(&id.to_be_bytes());
                out.extend_from_slice(&[*value_02, *width]);
                put_point(&mut out, *from);
                put_point(&mut out, *to);
                out.extend_from_slice(&color.0.to_be_bytes());
            }
            Self::Text {
                id,
                value_02,
                rect,
                value_0c,
                text,
                ..
            } => {
                head(&mut out, *id, *value_02);
                put_rect(&mut out, *rect);
                out.extend_from_slice(value_0c);
                out.extend_from_slice(text);
            }
            Self::Polygon {
                id,
                value_02,
                color,
                origin,
                points,
            } => {
                head(&mut out, *id, *value_02);
                out.extend_from_slice(&color.0.to_be_bytes());
                put_point(&mut out, *origin);
                for p in points {
                    put_point(&mut out, *p);
                }
            }
            Self::Copy {
                id,
                value_02,
                to,
                from,
                width,
                height,
            } => {
                head(&mut out, *id, *value_02);
                put_point(&mut out, *to);
                put_point(&mut out, *from);
                out.extend_from_slice(&width.to_be_bytes());
                out.extend_from_slice(&height.to_be_bytes());
            }
            Self::Other { body, .. } => out.extend_from_slice(body),
        }
        out
    }
}

/// 4001 Draw. Provisional name.
///
/// Draw records, each a u32 of kind (top 10 bits) and body length,
/// then the body, padded with zeros to 4 bytes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Draw(pub Vec<DrawRecord>);

impl Draw {
    /// Longest record body the header can describe.
    pub const MAX_BODY_LEN: usize = 0x3f_ffff;

    /// Decodes the payload. Like the client, it stops at a record that runs past the end.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        let mut records = Vec::new();
        let mut at = 0;
        while buf.len() - at >= 4 {
            let word = u32_at(buf, at);
            let len = (word & 0x3f_ffff) as usize;
            let end = at + 4 + len;
            let next = (end + 3) & !3;
            if next > buf.len() {
                return Err(DecodeError::Invalid("draw record length"));
            }
            records.push(DrawRecord::decode((word >> 22) as u16, &buf[at + 4..end]));
            at = next;
        }
        if at != buf.len() {
            return Err(DecodeError::Invalid("draw record length"));
        }
        Ok(Self(records))
    }

    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = Vec::new();
        for record in &self.0 {
            let body = record.encode_body();
            if body.len() > Self::MAX_BODY_LEN || record.kind() >= 0x400 {
                return Err(EncodeError::TooLong("draw record"));
            }
            let word = u32::from(record.kind()) << 22 | body.len() as u32;
            out.extend_from_slice(&word.to_be_bytes());
            out.extend_from_slice(&body);
            out.resize(out.len().next_multiple_of(4), 0);
        }
        Ok(out)
    }
}

/// 4200 Key: a key pressed or released in the window. Provisional name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Key {
    /// The client's clock in milliseconds.
    pub time: u32,
    /// Modifiers held before the event: 8 Shift, 2 Alt.
    pub modifiers: u32,
    /// The character typed; 0 on a release.
    pub character: u32,
    /// The USB HID usage of the key, such as 4 for A; 0 on most releases.
    pub usage: u16,
    /// 1 pressed, 2 released, 3 for the client's event 7, never seen.
    pub kind: u8,
    /// Byte 0x0f, 0.
    pub value_0f: u8,
}

impl Key {
    /// Wire length in bytes.
    pub const LEN: usize = 0x10;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::LEN)?;
        Ok(Self {
            time: u32_at(buf, 0),
            modifiers: u32_at(buf, 4),
            character: u32_at(buf, 8),
            usage: u16_at(buf, 0x0c),
            kind: buf[0x0e],
            value_0f: buf[0x0f],
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> [u8; Self::LEN] {
        let mut out = [0; Self::LEN];
        out[0..4].copy_from_slice(&self.time.to_be_bytes());
        out[4..8].copy_from_slice(&self.modifiers.to_be_bytes());
        out[8..0x0c].copy_from_slice(&self.character.to_be_bytes());
        out[0x0c..0x0e].copy_from_slice(&self.usage.to_be_bytes());
        out[0x0e] = self.kind;
        out[0x0f] = self.value_0f;
        out
    }
}

/// 4201 Mouse: a button pressed or released, or the wheel turned, in the window.
/// Provisional name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Mouse {
    /// The client's clock in milliseconds.
    pub time: u32,
    /// Bytes 0x04..0x08; 0x00040000 when the right button goes down, else 0.
    pub value_04: u32,
    /// X in the window.
    pub x: u16,
    /// Y in the window.
    pub y: u16,
    /// 1 left, 2 right, 3 middle, 4 wheel up, 5 wheel down.
    pub button: u8,
    /// 5 pressed, 6 released, 10 wheel.
    pub kind: u8,
    /// Wheel movement, 120 a step, negative down.
    pub wheel: i16,
}

impl Mouse {
    /// Wire length in bytes.
    pub const LEN: usize = 0x10;

    /// Decodes the payload.
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        need(buf, Self::LEN)?;
        Ok(Self {
            time: u32_at(buf, 0),
            value_04: u32_at(buf, 4),
            x: u16_at(buf, 8),
            y: u16_at(buf, 0x0a),
            button: buf[0x0c],
            kind: buf[0x0d],
            wheel: i16_at(buf, 0x0e),
        })
    }

    /// Encodes the payload.
    pub fn encode(&self) -> [u8; Self::LEN] {
        let mut out = [0; Self::LEN];
        out[0..4].copy_from_slice(&self.time.to_be_bytes());
        out[4..8].copy_from_slice(&self.value_04.to_be_bytes());
        out[8..0x0a].copy_from_slice(&self.x.to_be_bytes());
        out[0x0a..0x0c].copy_from_slice(&self.y.to_be_bytes());
        out[0x0c] = self.button;
        out[0x0d] = self.kind;
        out[0x0e..].copy_from_slice(&self.wheel.to_be_bytes());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_round_trip_pads_records() {
        let draw = Draw(vec![
            DrawRecord::Delete { id: 3 },
            DrawRecord::Text {
                kind: 105,
                id: 1,
                value_02: 0,
                rect: Rect::default(),
                value_0c: [0; 8],
                text: b"abc".to_vec(),
            },
        ]);
        let wire = draw.encode().unwrap();
        assert_eq!(wire[..8], [0x00, 0xc0, 0x00, 0x02, 0x00, 0x03, 0x00, 0x00]);
        assert_eq!(wire.len(), 8 + 4 + 0x18);
        assert_eq!(Draw::decode(&wire).unwrap(), draw);
    }

    #[test]
    fn draw_rejects_a_record_past_the_end() {
        assert_eq!(
            Draw::decode(&[0x00, 0xc0, 0x00, 0x08, 0, 3]),
            Err(DecodeError::Invalid("draw record length"))
        );
    }

    #[test]
    fn color_display() {
        assert_eq!(Color(0x0033_6699).to_string(), "#336699");
        assert_eq!(Color(0xff00_0000).to_string(), "#000000/ff");
    }
}
