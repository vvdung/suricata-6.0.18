/* Copyright (C) 2020 Open Information Security Foundation
 *
 * You can copy, redistribute or modify this Program under the terms of
 * the GNU General Public License version 2 as published by the Free
 * Software Foundation.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * version 2 along with this program; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA
 * 02110-1301, USA.
 */

use super::huffman;
use crate::http2::http2::{HTTP2DynTable, HTTP2_MAX_TABLESIZE};
use nom::character::complete::digit1;
use nom::combinator::rest;
use nom::error::ErrorKind;
use nom::number::streaming::{be_u16, be_u32, be_u8};
use nom::Err;
use nom::IResult;
use std::fmt;
use std::str::FromStr;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, FromPrimitive, Debug)]
pub enum HTTP2FrameType {
    DATA = 0,
    HEADERS = 1,
    PRIORITY = 2,
    RSTSTREAM = 3,
    SETTINGS = 4,
    PUSHPROMISE = 5,
    PING = 6,
    GOAWAY = 7,
    WINDOWUPDATE = 8,
    CONTINUATION = 9,
}

impl fmt::Display for HTTP2FrameType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::str::FromStr for HTTP2FrameType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let su = s.to_uppercase();
        let su_slice: &str = &*su;
        match su_slice {
            "DATA" => Ok(HTTP2FrameType::DATA),
            "HEADERS" => Ok(HTTP2FrameType::HEADERS),
            "PRIORITY" => Ok(HTTP2FrameType::PRIORITY),
            "RSTSTREAM" => Ok(HTTP2FrameType::RSTSTREAM),
            "SETTINGS" => Ok(HTTP2FrameType::SETTINGS),
            "PUSHPROMISE" => Ok(HTTP2FrameType::PUSHPROMISE),
            "PING" => Ok(HTTP2FrameType::PING),
            "GOAWAY" => Ok(HTTP2FrameType::GOAWAY),
            "WINDOWUPDATE" => Ok(HTTP2FrameType::WINDOWUPDATE),
            "CONTINUATION" => Ok(HTTP2FrameType::CONTINUATION),
            _ => Err(format!("'{}' is not a valid value for HTTP2FrameType", s)),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct HTTP2FrameHeader {
    //we could add detection on (GOAWAY) additional data
    pub length: u32,
    pub ftype: u8,
    pub flags: u8,
    pub reserved: u8,
    pub stream_id: u32,
}

named!(pub http2_parse_frame_header<HTTP2FrameHeader>,
    do_parse!(
        length: bits!( take_bits!(24u32) ) >>
        ftype: be_u8 >>
        flags: be_u8 >>
        stream_id: bits!( tuple!( take_bits!(1u8),
                                  take_bits!(31u32) ) ) >>
        (HTTP2FrameHeader{length, ftype, flags,
                          reserved:stream_id.0,
                          stream_id:stream_id.1})
    )
);

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, FromPrimitive, Debug)]
pub enum HTTP2ErrorCode {
    NOERROR = 0,
    PROTOCOLERROR = 1,
    INTERNALERROR = 2,
    FLOWCONTROLERROR = 3,
    SETTINGSTIMEOUT = 4,
    STREAMCLOSED = 5,
    FRAMESIZEERROR = 6,
    REFUSEDSTREAM = 7,
    CANCEL = 8,
    COMPRESSIONERROR = 9,
    CONNECTERROR = 10,
    ENHANCEYOURCALM = 11,
    INADEQUATESECURITY = 12,
    HTTP11REQUIRED = 13,
}

impl fmt::Display for HTTP2ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::str::FromStr for HTTP2ErrorCode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let su = s.to_uppercase();
        let su_slice: &str = &*su;
        match su_slice {
            "NO_ERROR" => Ok(HTTP2ErrorCode::NOERROR),
            "PROTOCOL_ERROR" => Ok(HTTP2ErrorCode::PROTOCOLERROR),
            "FLOW_CONTROL_ERROR" => Ok(HTTP2ErrorCode::FLOWCONTROLERROR),
            "SETTINGS_TIMEOUT" => Ok(HTTP2ErrorCode::SETTINGSTIMEOUT),
            "STREAM_CLOSED" => Ok(HTTP2ErrorCode::STREAMCLOSED),
            "FRAME_SIZE_ERROR" => Ok(HTTP2ErrorCode::FRAMESIZEERROR),
            "REFUSED_STREAM" => Ok(HTTP2ErrorCode::REFUSEDSTREAM),
            "CANCEL" => Ok(HTTP2ErrorCode::CANCEL),
            "COMPRESSION_ERROR" => Ok(HTTP2ErrorCode::COMPRESSIONERROR),
            "CONNECT_ERROR" => Ok(HTTP2ErrorCode::CONNECTERROR),
            "ENHANCE_YOUR_CALM" => Ok(HTTP2ErrorCode::ENHANCEYOURCALM),
            "INADEQUATE_SECURITY" => Ok(HTTP2ErrorCode::INADEQUATESECURITY),
            "HTTP_1_1_REQUIRED" => Ok(HTTP2ErrorCode::HTTP11REQUIRED),
            _ => Err(format!("'{}' is not a valid value for HTTP2ErrorCode", s)),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HTTP2FrameGoAway {
    pub errorcode: u32, //HTTP2ErrorCode
}

named!(pub http2_parse_frame_goaway<HTTP2FrameGoAway>,
    do_parse!(
        errorcode: be_u32 >>
        (HTTP2FrameGoAway{errorcode})
    )
);

#[derive(Clone, Copy, Debug)]
pub struct HTTP2FrameRstStream {
    pub errorcode: u32, ////HTTP2ErrorCode
}

named!(pub http2_parse_frame_rststream<HTTP2FrameRstStream>,
    do_parse!(
        errorcode: be_u32 >>
        (HTTP2FrameRstStream{errorcode})
    )
);

#[derive(Clone, Copy, Debug)]
pub struct HTTP2FramePriority {
    pub exclusive: u8,
    pub dependency: u32,
    pub weight: u8,
}

named!(pub http2_parse_frame_priority<HTTP2FramePriority>,
    do_parse!(
        sid: bits!( tuple!( take_bits!(1u8),
                            take_bits!(31u32) ) ) >>
        weight: be_u8 >>
        (HTTP2FramePriority{exclusive:sid.0, dependency:sid.1, weight})
    )
);

#[derive(Clone, Copy, Debug)]
pub struct HTTP2FrameWindowUpdate {
    pub reserved: u8,
    pub sizeinc: u32,
}

named!(pub http2_parse_frame_windowupdate<HTTP2FrameWindowUpdate>,
    do_parse!(
        sizeinc: bits!( tuple!( take_bits!(1u8),
                                take_bits!(31u32) ) ) >>
        (HTTP2FrameWindowUpdate{reserved:sizeinc.0, sizeinc:sizeinc.1})
    )
);

#[derive(Clone, Copy, Debug)]
pub struct HTTP2FrameHeadersPriority {
    pub exclusive: u8,
    pub dependency: u32,
    pub weight: u8,
}

named!(pub http2_parse_headers_priority<HTTP2FrameHeadersPriority>,
    do_parse!(
        sid: bits!( tuple!( take_bits!(1u8),
                                take_bits!(31u32) ) ) >>
        weight: be_u8 >>
        (HTTP2FrameHeadersPriority{exclusive:sid.0, dependency:sid.1, weight})
    )
);

pub const HTTP2_STATIC_HEADERS_NUMBER: usize = 61;

fn http2_frame_header_static(n: u64, dyn_headers: &HTTP2DynTable) -> Option<HTTP2FrameHeaderBlock> {
    let (name, value) = match n {
        1 => (":authority", ""),
        2 => (":method", "GET"),
        3 => (":method", "POST"),
        4 => (":path", "/"),
        5 => (":path", "/index.html"),
        6 => (":scheme", "http"),
        7 => (":scheme", "https"),
        8 => (":status", "200"),
        9 => (":status", "204"),
        10 => (":status", "206"),
        11 => (":status", "304"),
        12 => (":status", "400"),
        13 => (":status", "404"),
        14 => (":status", "500"),
        15 => ("accept-charset", ""),
        16 => ("accept-encoding", "gzip, deflate"),
        17 => ("accept-language", ""),
        18 => ("accept-ranges", ""),
        19 => ("accept", ""),
        20 => ("access-control-allow-origin", ""),
        21 => ("age", ""),
        22 => ("allow", ""),
        23 => ("authorization", ""),
        24 => ("cache-control", ""),
        25 => ("content-disposition", ""),
        26 => ("content-encoding", ""),
        27 => ("content-language", ""),
        28 => ("content-length", ""),
        29 => ("content-location", ""),
        30 => ("content-range", ""),
        31 => ("content-type", ""),
        32 => ("cookie", ""),
        33 => ("date", ""),
        34 => ("etag", ""),
        35 => ("expect", ""),
        36 => ("expires", ""),
        37 => ("from", ""),
        38 => ("host", ""),
        39 => ("if-match", ""),
        40 => ("if-modified-since", ""),
        41 => ("if-none-match", ""),
        42 => ("if-range", ""),
        43 => ("if-unmodified-since", ""),
        44 => ("last-modified", ""),
        45 => ("link", ""),
        46 => ("location", ""),
        47 => ("max-forwards", ""),
        48 => ("proxy-authenticate", ""),
        49 => ("proxy-authorization", ""),
        50 => ("range", ""),
        51 => ("referer", ""),
        52 => ("refresh", ""),
        53 => ("retry-after", ""),
        54 => ("server", ""),
        55 => ("set-cookie", ""),
        56 => ("strict-transport-security", ""),
        57 => ("transfer-encoding", ""),
        58 => ("user-agent", ""),
        59 => ("vary", ""),
        60 => ("via", ""),
        61 => ("www-authenticate", ""),
        _ => ("", ""),
    };
    if name.len() > 0 {
        return Some(HTTP2FrameHeaderBlock {
            name: name.as_bytes().to_vec(),
            value: value.as_bytes().to_vec(),
            error: HTTP2HeaderDecodeStatus::HTTP2HeaderDecodeSuccess,
            sizeupdate: 0,
        });
    } else {
        //use dynamic table
        if n == 0 {
            return Some(HTTP2FrameHeaderBlock {
                name: Vec::new(),
                value: Vec::new(),
                error: HTTP2HeaderDecodeStatus::HTTP2HeaderDecodeIndex0,
                sizeupdate: 0,
            });
        } else if dyn_headers.table.len() + HTTP2_STATIC_HEADERS_NUMBER < n as usize {
            return Some(HTTP2FrameHeaderBlock {
                name: Vec::new(),
                value: Vec::new(),
                error: HTTP2HeaderDecodeStatus::HTTP2HeaderDecodeNotIndexed,
                sizeupdate: 0,
            });
        } else {
            let indyn = dyn_headers.table.len() - (n as usize - HTTP2_STATIC_HEADERS_NUMBER);
            let headcopy = HTTP2FrameHeaderBlock {
                name: dyn_headers.table[indyn].name.to_vec(),
                value: dyn_headers.table[indyn].value.to_vec(),
                error: HTTP2HeaderDecodeStatus::HTTP2HeaderDecodeSuccess,
                sizeupdate: 0,
            };
            return Some(headcopy);
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
pub enum HTTP2HeaderDecodeStatus {
    HTTP2HeaderDecodeSuccess = 0,
    HTTP2HeaderDecodeSizeUpdate = 1,
    HTTP2HeaderDecodeError = 0x80,
    HTTP2HeaderDecodeNotIndexed = 0x81,
    HTTP2HeaderDecodeIntegerOverflow = 0x82,
    HTTP2HeaderDecodeIndex0 = 0x83,
}

impl fmt::Display for HTTP2HeaderDecodeStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug)]
pub struct HTTP2FrameHeaderBlock {
    pub name: Vec<u8>,
    pub value: Vec<u8>,
    pub error: HTTP2HeaderDecodeStatus,
    pub sizeupdate: u64,
}

fn http2_parse_headers_block_indexed<'a>(
    input: &'a [u8], dyn_headers: &HTTP2DynTable,
) -> IResult<&'a [u8], HTTP2FrameHeaderBlock> {
    fn parser(input: &[u8]) -> IResult<&[u8], (u8, u8)> {
        bits!(
            input,
            complete!(tuple!(
                verify!(take_bits!(1u8), |&x| x == 1),
                take_bits!(7u8)
            ))
        )
    }
    let (i2, indexed) = parser(input)?;
    let (i3, indexreal) = http2_parse_var_uint(i2, indexed.1 as u64, 0x7F)?;
    if indexreal == 0 && indexed.1 == 0x7F {
        return Err(Err::Error((i3, ErrorKind::LengthValue)));
    }
    match http2_frame_header_static(indexreal, dyn_headers) {
        Some(h) => Ok((i3, h)),
        _ => Err(Err::Error((i3, ErrorKind::MapOpt))),
    }
}

fn http2_parse_headers_block_string(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    fn parser(input: &[u8]) -> IResult<&[u8], (u8, u8)> {
        bits!(input, tuple!(take_bits!(1u8), take_bits!(7u8)))
    }
    let (i1, huffslen) = parser(input)?;
    let (i2, stringlen) = http2_parse_var_uint(i1, huffslen.1 as u64, 0x7F)?;
    if stringlen == 0 && huffslen.1 == 0x7F {
        return Err(Err::Error((i2, ErrorKind::LengthValue)));
    }
    let (i3, data) = take!(i2, stringlen as usize)?;
    if huffslen.0 == 0 {
        return Ok((i3, data.to_vec()));
    } else {
        let (_, val) = bits!(data, many0!(huffman::http2_decode_huffman))?;
        return Ok((i3, val));
    }
}

fn http2_parse_headers_block_literal_common<'a>(
    input: &'a [u8], index: u64, dyn_headers: &HTTP2DynTable,
) -> IResult<&'a [u8], HTTP2FrameHeaderBlock> {
    let (i3, name, error) = if index == 0 {
        match http2_parse_headers_block_string(input) {
            Ok((r, n)) => Ok((r, n, HTTP2HeaderDecodeStatus::HTTP2HeaderDecodeSuccess)),
            Err(e) => Err(e),
        }
    } else {
        match http2_frame_header_static(index, dyn_headers) {
            Some(x) => Ok((
                input,
                x.name,
                HTTP2HeaderDecodeStatus::HTTP2HeaderDecodeSuccess,
            )),
            None => Ok((
                input,
                Vec::new(),
                HTTP2HeaderDecodeStatus::HTTP2HeaderDecodeNotIndexed,
            )),
        }
    }?;
    let (i4, value) = http2_parse_headers_block_string(i3)?;
    return Ok((
        i4,
        HTTP2FrameHeaderBlock {
            name,
            value,
            error,
            sizeupdate: 0,
        },
    ));
}

fn http2_parse_headers_block_literal_incindex<'a>(
    input: &'a [u8], dyn_headers: &mut HTTP2DynTable,
) -> IResult<&'a [u8], HTTP2FrameHeaderBlock> {
    fn parser(input: &[u8]) -> IResult<&[u8], (u8, u8)> {
        bits!(
            input,
            complete!(tuple!(
                verify!(take_bits!(2u8), |&x| x == 1),
                take_bits!(6u8)
            ))
        )
    }
    let (i2, indexed) = parser(input)?;
    let (i3, indexreal) = http2_parse_var_uint(i2, indexed.1 as u64, 0x3F)?;
    if indexreal == 0 && indexed.1 == 0x3F {
        return Err(Err::Error((i3, ErrorKind::LengthValue)));
    }
    let r = http2_parse_headers_block_literal_common(i3, indexreal, dyn_headers);
    match r {
        Ok((r, head)) => {
            let headcopy = HTTP2FrameHeaderBlock {
                name: head.name.to_vec(),
                value: head.value.to_vec(),
                error: head.error,
                sizeupdate: 0,
            };
            if head.error == HTTP2HeaderDecodeStatus::HTTP2HeaderDecodeSuccess {
                dyn_headers.current_size += 32 + headcopy.name.len() + headcopy.value.len();
                //in case of overflow, best effort is to keep first headers
                if dyn_headers.overflow > 0 {
                    if dyn_headers.overflow == 1 {
                        if dyn_headers.current_size <= (HTTP2_MAX_TABLESIZE as usize) {
                            //overflow had not yet happened
                            dyn_headers.table.push(headcopy);
                        } else if dyn_headers.current_size > dyn_headers.max_size {
                            //overflow happens, we cannot replace evicted headers
                            dyn_headers.overflow = 2;
                        }
                    }
                } else {
                    dyn_headers.table.push(headcopy);
                }
                let mut toremove = 0;
                while dyn_headers.current_size > dyn_headers.max_size
                    && toremove < dyn_headers.table.len()
                {
                    dyn_headers.current_size -=
                        32 + dyn_headers.table[toremove].name.len() + dyn_headers.table[toremove].value.len();
                    toremove += 1;
                }
                dyn_headers.table.drain(0..toremove);
            }
            return Ok((r, head));
        }
        Err(e) => {
            return Err(e);
        }
    }
}

fn http2_parse_headers_block_literal_noindex<'a>(
    input: &'a [u8], dyn_headers: &HTTP2DynTable,
) -> IResult<&'a [u8], HTTP2FrameHeaderBlock> {
    fn parser(input: &[u8]) -> IResult<&[u8], (u8, u8)> {
        bits!(
            input,
            complete!(tuple!(
                verify!(take_bits!(4u8), |&x| x == 0),
                take_bits!(4u8)
            ))
        )
    }
    let (i2, indexed) = parser(input)?;
    let (i3, indexreal) = http2_parse_var_uint(i2, indexed.1 as u64, 0xF)?;
    if indexreal == 0 && indexed.1 == 0xF {
        return Err(Err::Error((i3, ErrorKind::LengthValue)));
    }
    let r = http2_parse_headers_block_literal_common(i3, indexreal, dyn_headers);
    return r;
}

fn http2_parse_headers_block_literal_neverindex<'a>(
    input: &'a [u8], dyn_headers: &HTTP2DynTable,
) -> IResult<&'a [u8], HTTP2FrameHeaderBlock> {
    fn parser(input: &[u8]) -> IResult<&[u8], (u8, u8)> {
        bits!(
            input,
            complete!(tuple!(
                verify!(take_bits!(4u8), |&x| x == 1),
                take_bits!(4u8)
            ))
        )
    }
    let (i2, indexed) = parser(input)?;
    let (i3, indexreal) = http2_parse_var_uint(i2, indexed.1 as u64, 0xF)?;
    if indexreal == 0 && indexed.1 == 0xF {
        return Err(Err::Error((i3, ErrorKind::LengthValue)));
    }
    let r = http2_parse_headers_block_literal_common(i3, indexreal, dyn_headers);
    return r;
}

fn http2_parse_var_uint(input: &[u8], value: u64, max: u64) -> IResult<&[u8], u64> {
    if value < max {
        return Ok((input, value));
    }
    let (i2, varia) = take_while!(input, |ch| (ch & 0x80) != 0)?;
    let (i3, finalv) = be_u8(i2)?;
    if varia.len() > 9 || (varia.len() == 9 && finalv > 1) {
        // this will overflow u64
        return Ok((i3, 0));
    }
    let mut varval = max;
    for i in 0..varia.len() {
        varval += ((varia[i] & 0x7F) as u64) << (7 * i);
    }
    if (finalv as u64) << (7 * varia.len()) > std::u64::MAX - varval {
        // this would overflow u64
        return Ok((i3, 0));
    }
    varval += (finalv as u64) << (7 * varia.len());
    return Ok((i3, varval));
}

fn http2_parse_headers_block_dynamic_size<'a>(
    input: &'a [u8], dyn_headers: &mut HTTP2DynTable,
) -> IResult<&'a [u8], HTTP2FrameHeaderBlock> {
    fn parser(input: &[u8]) -> IResult<&[u8], (u8, u8)> {
        bits!(
            input,
            complete!(tuple!(
                verify!(take_bits!(3u8), |&x| x == 1),
                take_bits!(5u8)
            ))
        )
    }
    let (i2, maxsize) = parser(input)?;
    let (i3, maxsize2) = http2_parse_var_uint(i2, maxsize.1 as u64, 0x1F)?;
    if maxsize2 == 0 && maxsize.1 == 0x1F {
        return Ok((
            i3,
            HTTP2FrameHeaderBlock {
                name: Vec::new(),
                value: Vec::new(),
                error: HTTP2HeaderDecodeStatus::HTTP2HeaderDecodeIntegerOverflow,
                sizeupdate: 0,
            },
        ));
    }
    if (maxsize2 as usize) < dyn_headers.max_size {
        //dyn_headers.max_size is updated later with all headers
        //may evict entries
        let mut toremove = 0;
        while dyn_headers.current_size > (maxsize2 as usize) && toremove < dyn_headers.table.len() {
            // we check dyn_headers.table as we may be in best effort
            // because the previous maxsize was too big for us to retain all the headers
            dyn_headers.current_size -= 32
                + dyn_headers.table[toremove].name.len()
                + dyn_headers.table[toremove].value.len();
            toremove += 1;
        }
        dyn_headers.table.drain(0..toremove);
    }
    return Ok((
        i3,
        HTTP2FrameHeaderBlock {
            name: Vec::new(),
            value: Vec::new(),
            error: HTTP2HeaderDecodeStatus::HTTP2HeaderDecodeSizeUpdate,
            sizeupdate: maxsize2,
        },
    ));
}

fn http2_parse_headers_block<'a>(
    input: &'a [u8], dyn_headers: &mut HTTP2DynTable,
) -> IResult<&'a [u8], HTTP2FrameHeaderBlock> {
    //caller garantees o have at least one byte
    if input[0] & 0x80 != 0 {
        return http2_parse_headers_block_indexed(input, dyn_headers);
    } else if input[0] & 0x40 != 0 {
        return http2_parse_headers_block_literal_incindex(input, dyn_headers);
    } else if input[0] & 0x20 != 0 {
        return http2_parse_headers_block_dynamic_size(input, dyn_headers);
    } else if input[0] & 0x10 != 0 {
        return http2_parse_headers_block_literal_neverindex(input, dyn_headers);
    } else {
        return http2_parse_headers_block_literal_noindex(input, dyn_headers);
    }
}

#[derive(Clone, Debug)]
pub struct HTTP2FrameHeaders {
    pub padlength: Option<u8>,
    pub priority: Option<HTTP2FrameHeadersPriority>,
    pub blocks: Vec<HTTP2FrameHeaderBlock>,
}

//end stream
pub const HTTP2_FLAG_HEADER_EOS: u8 = 0x1;
pub const HTTP2_FLAG_HEADER_END_HEADERS: u8 = 0x4;
pub const HTTP2_FLAG_HEADER_PADDED: u8 = 0x8;
const HTTP2_FLAG_HEADER_PRIORITY: u8 = 0x20;

pub fn http2_parse_frame_headers<'a>(
    input: &'a [u8], flags: u8, dyn_headers: &mut HTTP2DynTable,
) -> IResult<&'a [u8], HTTP2FrameHeaders> {
    let (i2, padlength) = cond!(input, flags & HTTP2_FLAG_HEADER_PADDED != 0, be_u8)?;
    let (mut i3, priority) = cond!(
        i2,
        flags & HTTP2_FLAG_HEADER_PRIORITY != 0,
        http2_parse_headers_priority
    )?;
    let mut blocks = Vec::new();
    while i3.len() > 0 {
        match http2_parse_headers_block(i3, dyn_headers) {
            Ok((rem, b)) => {
                blocks.push(b);
                debug_validate_bug_on!(i3.len() == rem.len());
                if i3.len() == rem.len() {
                    //infinite loop
                    return Err(Err::Error((input, ErrorKind::Eof)));
                }
                i3 = rem;
            }
            Err(x) => {
                return Err(x);
            }
        }
    }
    return Ok((
        i3,
        HTTP2FrameHeaders {
            padlength,
            priority,
            blocks,
        },
    ));
}

#[derive(Clone, Debug)]
pub struct HTTP2FramePushPromise {
    pub padlength: Option<u8>,
    pub reserved: u8,
    pub stream_id: u32,
    pub blocks: Vec<HTTP2FrameHeaderBlock>,
}

pub fn http2_parse_frame_push_promise<'a>(
    input: &'a [u8], flags: u8, dyn_headers: &mut HTTP2DynTable,
) -> IResult<&'a [u8], HTTP2FramePushPromise> {
    let (i2, padlength) = cond!(input, flags & HTTP2_FLAG_HEADER_PADDED != 0, be_u8)?;
    let (mut i3, stream_id) = bits!(i2, tuple!(take_bits!(1u8), take_bits!(31u32)))?;
    let mut blocks = Vec::new();
    while i3.len() > 0 {
        match http2_parse_headers_block(i3, dyn_headers) {
            Ok((rem, b)) => {
                blocks.push(b);
                debug_validate_bug_on!(i3.len() == rem.len());
                if i3.len() == rem.len() {
                    //infinite loop
                    return Err(Err::Error((input, ErrorKind::Eof)));
                }
                i3 = rem;
            }
            Err(x) => {
                return Err(x);
            }
        }
    }
    return Ok((
        i3,
        HTTP2FramePushPromise {
            padlength,
            reserved: stream_id.0,
            stream_id: stream_id.1,
            blocks,
        },
    ));
}

#[derive(Clone, Debug)]
pub struct HTTP2FrameContinuation {
    pub blocks: Vec<HTTP2FrameHeaderBlock>,
}

pub fn http2_parse_frame_continuation<'a>(
    input: &'a [u8], dyn_headers: &mut HTTP2DynTable,
) -> IResult<&'a [u8], HTTP2FrameContinuation> {
    let mut i3 = input;
    let mut blocks = Vec::new();
    while i3.len() > 0 {
        match http2_parse_headers_block(i3, dyn_headers) {
            Ok((rem, b)) => {
                blocks.push(b);
                debug_validate_bug_on!(i3.len() == rem.len());
                if i3.len() == rem.len() {
                    //infinite loop
                    return Err(Err::Error((input, ErrorKind::Eof)));
                }
                i3 = rem;
            }
            Err(x) => {
                return Err(x);
            }
        }
    }
    return Ok((i3, HTTP2FrameContinuation { blocks }));
}

#[repr(u16)]
#[derive(Clone, Copy, PartialEq, FromPrimitive, Debug)]
pub enum HTTP2SettingsId {
    SETTINGSHEADERTABLESIZE = 1,
    SETTINGSENABLEPUSH = 2,
    SETTINGSMAXCONCURRENTSTREAMS = 3,
    SETTINGSINITIALWINDOWSIZE = 4,
    SETTINGSMAXFRAMESIZE = 5,
    SETTINGSMAXHEADERLISTSIZE = 6,
}

impl fmt::Display for HTTP2SettingsId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::str::FromStr for HTTP2SettingsId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let su = s.to_uppercase();
        let su_slice: &str = &*su;
        match su_slice {
            "SETTINGS_HEADER_TABLE_SIZE" => Ok(HTTP2SettingsId::SETTINGSHEADERTABLESIZE),
            "SETTINGS_ENABLE_PUSH" => Ok(HTTP2SettingsId::SETTINGSENABLEPUSH),
            "SETTINGS_MAX_CONCURRENT_STREAMS" => Ok(HTTP2SettingsId::SETTINGSMAXCONCURRENTSTREAMS),
            "SETTINGS_INITIAL_WINDOW_SIZE" => Ok(HTTP2SettingsId::SETTINGSINITIALWINDOWSIZE),
            "SETTINGS_MAX_FRAME_SIZE" => Ok(HTTP2SettingsId::SETTINGSMAXFRAMESIZE),
            "SETTINGS_MAX_HEADER_LIST_SIZE" => Ok(HTTP2SettingsId::SETTINGSMAXHEADERLISTSIZE),
            _ => Err(format!("'{}' is not a valid value for HTTP2SettingsId", s)),
        }
    }
}

//TODOask move elsewhere generic with DetectU64Data and such
#[derive(PartialEq, Debug)]
pub enum DetectUintMode {
    DetectUintModeEqual,
    DetectUintModeLt,
    DetectUintModeGt,
    DetectUintModeRange,
}

pub struct DetectU32Data {
    pub value: u32,
    pub valrange: u32,
    pub mode: DetectUintMode,
}

pub struct DetectHTTP2settingsSigCtx {
    pub id: HTTP2SettingsId,          //identifier
    pub value: Option<DetectU32Data>, //optional value
}

named!(detect_parse_u32_start_equal<&str,DetectU32Data>,
    do_parse!(
        opt!( is_a!( " " ) ) >>
        opt! (tag!("=") ) >>
        opt!( is_a!( " " ) ) >>
        value : map_opt!(digit1, |s: &str| s.parse::<u32>().ok()) >>
        (DetectU32Data{value, valrange:0, mode:DetectUintMode::DetectUintModeEqual})
    )
);

named!(detect_parse_u32_start_interval<&str,DetectU32Data>,
    do_parse!(
        opt!( is_a!( " " ) ) >>
        value : map_opt!(digit1, |s: &str| s.parse::<u32>().ok()) >>
        opt!( is_a!( " " ) ) >>
        tag!("-") >>
        opt!( is_a!( " " ) ) >>
        valrange : map_opt!(digit1, |s: &str| s.parse::<u32>().ok()) >>
        (DetectU32Data{value, valrange, mode:DetectUintMode::DetectUintModeRange})
    )
);

named!(detect_parse_u32_start_lesser<&str,DetectU32Data>,
    do_parse!(
        opt!( is_a!( " " ) ) >>
        tag!("<") >>
        opt!( is_a!( " " ) ) >>
        value : map_opt!(digit1, |s: &str| s.parse::<u32>().ok()) >>
        (DetectU32Data{value, valrange:0, mode:DetectUintMode::DetectUintModeLt})
    )
);

named!(detect_parse_u32_start_greater<&str,DetectU32Data>,
    do_parse!(
        opt!( is_a!( " " ) ) >>
        tag!(">") >>
        opt!( is_a!( " " ) ) >>
        value : map_opt!(digit1, |s: &str| s.parse::<u32>().ok()) >>
        (DetectU32Data{value, valrange:0, mode:DetectUintMode::DetectUintModeGt})
    )
);

named!(detect_parse_u32<&str,DetectU32Data>,
    do_parse!(
        u32 : alt! (
            detect_parse_u32_start_lesser |
            detect_parse_u32_start_greater |
            complete!( detect_parse_u32_start_interval ) |
            detect_parse_u32_start_equal
        ) >>
        (u32)
    )
);

named!(pub http2_parse_settingsctx<&str,DetectHTTP2settingsSigCtx>,
    do_parse!(
        opt!( is_a!( " " ) ) >>
        id: map_opt!( alt! ( complete!( is_not!( " <>=" ) ) | rest ),
            |s: &str| HTTP2SettingsId::from_str(s).ok() ) >>
        value: opt!( complete!( detect_parse_u32 ) ) >>
        (DetectHTTP2settingsSigCtx{id, value})
    )
);

pub struct DetectU64Data {
    pub value: u64,
    pub valrange: u64,
    pub mode: DetectUintMode,
}

named!(detect_parse_u64_start_equal<&str,DetectU64Data>,
    do_parse!(
        opt!( is_a!( " " ) ) >>
        opt! (tag!("=") ) >>
        opt!( is_a!( " " ) ) >>
        value : map_opt!(digit1, |s: &str| s.parse::<u64>().ok()) >>
        (DetectU64Data{value, valrange:0, mode:DetectUintMode::DetectUintModeEqual})
    )
);

named!(detect_parse_u64_start_interval<&str,DetectU64Data>,
    do_parse!(
        opt!( is_a!( " " ) ) >>
        value : map_opt!(digit1, |s: &str| s.parse::<u64>().ok()) >>
        opt!( is_a!( " " ) ) >>
        tag!("-") >>
        opt!( is_a!( " " ) ) >>
        valrange : map_opt!(digit1, |s: &str| s.parse::<u64>().ok()) >>
        (DetectU64Data{value, valrange, mode:DetectUintMode::DetectUintModeRange})
    )
);

named!(detect_parse_u64_start_lesser<&str,DetectU64Data>,
    do_parse!(
        opt!( is_a!( " " ) ) >>
        tag!("<") >>
        opt!( is_a!( " " ) ) >>
        value : map_opt!(digit1, |s: &str| s.parse::<u64>().ok()) >>
        (DetectU64Data{value, valrange:0, mode:DetectUintMode::DetectUintModeLt})
    )
);

named!(detect_parse_u64_start_greater<&str,DetectU64Data>,
    do_parse!(
        opt!( is_a!( " " ) ) >>
        tag!(">") >>
        opt!( is_a!( " " ) ) >>
        value : map_opt!(digit1, |s: &str| s.parse::<u64>().ok()) >>
        (DetectU64Data{value, valrange:0, mode:DetectUintMode::DetectUintModeGt})
    )
);

named!(pub detect_parse_u64<&str,DetectU64Data>,
    do_parse!(
        u64 : alt! (
            detect_parse_u64_start_lesser |
            detect_parse_u64_start_greater |
            complete!( detect_parse_u64_start_interval ) |
            detect_parse_u64_start_equal
        ) >>
        (u64)
    )
);

#[derive(Clone, Copy, Debug)]
pub struct HTTP2FrameSettings {
    pub id: HTTP2SettingsId,
    pub value: u32,
}

named!(
    http2_parse_frame_setting<HTTP2FrameSettings>,
    do_parse!(
        id: map_opt!(be_u16, num::FromPrimitive::from_u16)
            >> value: be_u32
            >> (HTTP2FrameSettings { id, value })
    )
);

named!(pub http2_parse_frame_settings<Vec<HTTP2FrameSettings>>,
    many0!( complete!(http2_parse_frame_setting) )
);

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_http2_parse_header() {
        let buf0: &[u8] = &[0x82];
        let mut dynh = HTTP2DynTable::new();
        let r0 = http2_parse_headers_block(buf0, &mut dynh);
        match r0 {
            Ok((remainder, hd)) => {
                // Check the first message.
                assert_eq!(hd.name, ":method".as_bytes().to_vec());
                assert_eq!(hd.value, "GET".as_bytes().to_vec());
                // And we should have no bytes left.
                assert_eq!(remainder.len(), 0);
            }
            Err(Err::Incomplete(_)) => {
                panic!("Result should not have been incomplete.");
            }
            Err(Err::Error(err)) | Err(Err::Failure(err)) => {
                panic!("Result should not be an error: {:?}.", err);
            }
        }
        let buf1: &[u8] = &[0x53, 0x03, 0x2A, 0x2F, 0x2A];
        let r1 = http2_parse_headers_block(buf1, &mut dynh);
        match r1 {
            Ok((remainder, hd)) => {
                // Check the first message.
                assert_eq!(hd.name, "accept".as_bytes().to_vec());
                assert_eq!(hd.value, "*/*".as_bytes().to_vec());
                // And we should have no bytes left.
                assert_eq!(remainder.len(), 0);
                assert_eq!(dynh.table.len(), 1);
            }
            Err(Err::Incomplete(_)) => {
                panic!("Result should not have been incomplete.");
            }
            Err(Err::Error(err)) | Err(Err::Failure(err)) => {
                panic!("Result should not be an error: {:?}.", err);
            }
        }
        let buf: &[u8] = &[
            0x41, 0x8a, 0xa0, 0xe4, 0x1d, 0x13, 0x9d, 0x09, 0xb8, 0xc8, 0x00, 0x0f,
        ];
        let result = http2_parse_headers_block(buf, &mut dynh);
        match result {
            Ok((remainder, hd)) => {
                // Check the first message.
                assert_eq!(hd.name, ":authority".as_bytes().to_vec());
                assert_eq!(hd.value, "localhost:3000".as_bytes().to_vec());
                // And we should have no bytes left.
                assert_eq!(remainder.len(), 0);
                assert_eq!(dynh.table.len(), 2);
            }
            Err(Err::Incomplete(_)) => {
                panic!("Result should not have been incomplete.");
            }
            Err(Err::Error(err)) | Err(Err::Failure(err)) => {
                panic!("Result should not be an error: {:?}.", err);
            }
        }
        let buf3: &[u8] = &[0xbe];
        let r3 = http2_parse_headers_block(buf3, &mut dynh);
        match r3 {
            Ok((remainder, hd)) => {
                // same as before
                assert_eq!(hd.name, ":authority".as_bytes().to_vec());
                assert_eq!(hd.value, "localhost:3000".as_bytes().to_vec());
                // And we should have no bytes left.
                assert_eq!(remainder.len(), 0);
                assert_eq!(dynh.table.len(), 2);
            }
            Err(Err::Incomplete(_)) => {
                panic!("Result should not have been incomplete.");
            }
            Err(Err::Error(err)) | Err(Err::Failure(err)) => {
                panic!("Result should not be an error: {:?}.", err);
            }
        }
        let buf4: &[u8] = &[0x80];
        let r4 = http2_parse_headers_block(buf4, &mut dynh);
        match r4 {
            Ok((remainder, hd)) => {
                assert_eq!(hd.error, HTTP2HeaderDecodeStatus::HTTP2HeaderDecodeIndex0);
                assert_eq!(remainder.len(), 0);
                assert_eq!(dynh.table.len(), 2);
            }
            Err(Err::Incomplete(_)) => {
                panic!("Result should not have been incomplete.");
            }
            Err(Err::Error(err)) | Err(Err::Failure(err)) => {
                panic!("Result should not be an error: {:?}.", err);
            }
        }
        let buf2: &[u8] = &[
            0x04, 0x94, 0x62, 0x43, 0x91, 0x8a, 0x47, 0x55, 0xa3, 0xa1, 0x89, 0xd3, 0x4d, 0x0c,
            0x1a, 0xa9, 0x0b, 0xe5, 0x79, 0xd3, 0x4d, 0x1f,
        ];
        let r2 = http2_parse_headers_block(buf2, &mut dynh);
        match r2 {
            Ok((remainder, hd)) => {
                // Check the first message.
                assert_eq!(hd.name, ":path".as_bytes().to_vec());
                assert_eq!(hd.value, "/doc/manual/html/index.html".as_bytes().to_vec());
                // And we should have no bytes left.
                assert_eq!(remainder.len(), 0);
                assert_eq!(dynh.table.len(), 2);
            }
            Err(Err::Incomplete(_)) => {
                panic!("Result should not have been incomplete.");
            }
            Err(Err::Error(err)) | Err(Err::Failure(err)) => {
                panic!("Result should not be an error: {:?}.", err);
            }
        }
    }

    /// Simple test of some valid data.
    #[test]
    fn test_http2_parse_settingsctx() {
        let s = "SETTINGS_ENABLE_PUSH";
        let r = http2_parse_settingsctx(s);
        match r {
            Ok((rem, ctx)) => {
                assert_eq!(ctx.id, HTTP2SettingsId::SETTINGSENABLEPUSH);
                match ctx.value {
                    Some(_) => {
                        panic!("Unexpected value");
                    }
                    None => {}
                }
                assert_eq!(rem.len(), 0);
            }
            Err(e) => {
                panic!("Result should not be an error {:?}.", e);
            }
        }

        //spaces in the end
        let s1 = "SETTINGS_ENABLE_PUSH ";
        let r1 = http2_parse_settingsctx(s1);
        match r1 {
            Ok((rem, ctx)) => {
                assert_eq!(ctx.id, HTTP2SettingsId::SETTINGSENABLEPUSH);
                match ctx.value {
                    Some(_) => {
                        panic!("Unexpected value");
                    }
                    None => {}
                }
                assert_eq!(rem.len(), 1);
            }
            Err(e) => {
                panic!("Result should not be an error {:?}.", e);
            }
        }

        let s2 = "SETTINGS_MAX_CONCURRENT_STREAMS  42";
        let r2 = http2_parse_settingsctx(s2);
        match r2 {
            Ok((rem, ctx)) => {
                assert_eq!(ctx.id, HTTP2SettingsId::SETTINGSMAXCONCURRENTSTREAMS);
                match ctx.value {
                    Some(ctxval) => {
                        assert_eq!(ctxval.value, 42);
                    }
                    None => {
                        panic!("No value");
                    }
                }
                assert_eq!(rem.len(), 0);
            }
            Err(e) => {
                panic!("Result should not be an error {:?}.", e);
            }
        }

        let s3 = "SETTINGS_MAX_CONCURRENT_STREAMS 42-68";
        let r3 = http2_parse_settingsctx(s3);
        match r3 {
            Ok((rem, ctx)) => {
                assert_eq!(ctx.id, HTTP2SettingsId::SETTINGSMAXCONCURRENTSTREAMS);
                match ctx.value {
                    Some(ctxval) => {
                        assert_eq!(ctxval.value, 42);
                        assert_eq!(ctxval.mode, DetectUintMode::DetectUintModeRange);
                        assert_eq!(ctxval.valrange, 68);
                    }
                    None => {
                        panic!("No value");
                    }
                }
                assert_eq!(rem.len(), 0);
            }
            Err(e) => {
                panic!("Result should not be an error {:?}.", e);
            }
        }

        let s4 = "SETTINGS_MAX_CONCURRENT_STREAMS<54";
        let r4 = http2_parse_settingsctx(s4);
        match r4 {
            Ok((rem, ctx)) => {
                assert_eq!(ctx.id, HTTP2SettingsId::SETTINGSMAXCONCURRENTSTREAMS);
                match ctx.value {
                    Some(ctxval) => {
                        assert_eq!(ctxval.value, 54);
                        assert_eq!(ctxval.mode, DetectUintMode::DetectUintModeLt);
                    }
                    None => {
                        panic!("No value");
                    }
                }
                assert_eq!(rem.len(), 0);
            }
            Err(e) => {
                panic!("Result should not be an error {:?}.", e);
            }
        }

        let s5 = "SETTINGS_MAX_CONCURRENT_STREAMS > 76";
        let r5 = http2_parse_settingsctx(s5);
        match r5 {
            Ok((rem, ctx)) => {
                assert_eq!(ctx.id, HTTP2SettingsId::SETTINGSMAXCONCURRENTSTREAMS);
                match ctx.value {
                    Some(ctxval) => {
                        assert_eq!(ctxval.value, 76);
                        assert_eq!(ctxval.mode, DetectUintMode::DetectUintModeGt);
                    }
                    None => {
                        panic!("No value");
                    }
                }
                assert_eq!(rem.len(), 0);
            }
            Err(e) => {
                panic!("Result should not be an error {:?}.", e);
            }
        }
    }

    #[test]
    fn test_http2_parse_headers_block_string() {
        let buf: &[u8] = &[0x01, 0xFF];
        let r = http2_parse_headers_block_string(buf);
        match r {
            Ok((remainder, _)) => {
                assert_eq!(remainder.len(), 0);
            }
            Err(Err::Error(err)) | Err(Err::Failure(err)) => {
                panic!("Result should not be an error: {:?}.", err);
            }
            _ => {
                panic!("Result should have been ok");
            }
        }
        let buf2: &[u8] = &[0x83, 0xFF, 0xFF, 0xEA];
        let r2 = http2_parse_headers_block_string(buf2);
        match r2 {
            Ok((remainder, _)) => {
                assert_eq!(remainder.len(), 0);
            }
            _ => {
                panic!("Result should have been ok");
            }
        }
    }

    #[test]
    fn test_http2_parse_frame_header() {
        let buf: &[u8] = &[
            0x00, 0x00, 0x06, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00,
            0x64,
        ];
        let result = http2_parse_frame_header(buf);
        match result {
            Ok((remainder, frame)) => {
                // Check the first message.
                assert_eq!(frame.length, 6);
                assert_eq!(frame.ftype, HTTP2FrameType::SETTINGS as u8);
                assert_eq!(frame.flags, 0);
                assert_eq!(frame.reserved, 0);
                assert_eq!(frame.stream_id, 0);

                // And we should have 6 bytes left.
                assert_eq!(remainder.len(), 6);
            }
            Err(Err::Incomplete(_)) => {
                panic!("Result should not have been incomplete.");
            }
            Err(Err::Error(err)) | Err(Err::Failure(err)) => {
                panic!("Result should not be an error: {:?}.", err);
            }
        }
    }
}
