//! Messaging primitives for requests.

use std::io::Write as _;

use byteorder::{BigEndian, WriteBytesExt};
use nom::bytes::complete::take;
use nom::combinator::{map, map_res};
use nom::number::complete::{be_u32, be_u64};
use nom::sequence::tuple;
use nom::IResult;
use tracing::instrument;

use crate::announce::AnnounceRequest;
use crate::scrape::ScrapeRequest;

// For all practical applications, this value should be hardcoded as a valid
// connection id for connection requests when operating in server mode and processing
// incoming requests.
/// Global connection id for connect requests.
pub const CONNECT_ID_PROTOCOL_ID: u64 = 0x0417_2710_1980;

/// Enumerates all types of requests that can be made to a tracker.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub enum RequestType<'a> {
    Connect,
    Announce(AnnounceRequest<'a>),
    Scrape(ScrapeRequest<'a>),
}

impl<'a> RequestType<'a> {
    /// Create an owned version of the `RequestType`.
    #[must_use]
    pub fn to_owned(&self) -> RequestType<'static> {
        match self {
            &RequestType::Connect => RequestType::Connect,
            RequestType::Announce(req) => RequestType::Announce(req.to_owned()),
            RequestType::Scrape(req) => RequestType::Scrape(req.to_owned()),
        }
    }
}

/// `TrackerRequest` which encapsulates any request sent to a tracker.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct TrackerRequest<'a> {
    // Both the connection id and transaction id are technically not unsigned according
    // to the spec, but since they are just bits we will keep them as unsigned since it
    // doesn't really make sense to not have them as unsigned (easier to generate transactions).
    connection_id: u64,
    transaction_id: u32,
    request_type: RequestType<'a>,
}

impl<'a> TrackerRequest<'a> {
    /// Create a new `TrackerRequest`.
    #[must_use]
    pub fn new(conn_id: u64, trans_id: u32, req_type: RequestType<'a>) -> TrackerRequest<'a> {
        TrackerRequest {
            connection_id: conn_id,
            transaction_id: trans_id,
            request_type: req_type,
        }
    }

    /// Create a new `TrackerRequest` from the given bytes.
    ///
    /// # Errors
    ///
    /// It will return an error when unable to parse the bytes.
    pub fn from_bytes(bytes: &'a [u8]) -> IResult<&'a [u8], TrackerRequest<'a>> {
        parse_request(bytes)
    }

    /// Write the `TrackerRequest` to the given writer.
    ///
    /// # Errors
    ///
    /// It would return an IO Error if unable to write the bytes.
    #[allow(clippy::needless_borrows_for_generic_args)]
    #[instrument(skip(self, writer), err)]
    pub fn write_bytes<W>(&self, mut writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writer.write_u64::<BigEndian>(self.connection_id())?;

        {
            match self.request_type() {
                &RequestType::Connect => {
                    writer.write_u32::<BigEndian>(crate::CONNECT_ACTION_ID)?;
                    writer.write_u32::<BigEndian>(self.transaction_id())?;
                }
                RequestType::Announce(req) => {
                    let action_id = if req.source_ip().is_ipv4() {
                        crate::ANNOUNCE_IPV4_ACTION_ID
                    } else {
                        crate::ANNOUNCE_IPV6_ACTION_ID
                    };
                    writer.write_u32::<BigEndian>(action_id)?;
                    writer.write_u32::<BigEndian>(self.transaction_id())?;

                    req.write_bytes(&mut writer)?;
                }
                RequestType::Scrape(req) => {
                    writer.write_u32::<BigEndian>(crate::SCRAPE_ACTION_ID)?;
                    writer.write_u32::<BigEndian>(self.transaction_id())?;

                    req.write_bytes(&mut writer)?;
                }
            };
        }
        writer.flush()?;

        Ok(())
    }

    /// Connection ID supplied with a request to validate the senders address.
    ///
    /// For Connect requests, this will always be equal to 0x41727101980. Therefore,
    /// you should not hand out that specific ID to peers that make a connect request.
    #[must_use]
    pub fn connection_id(&self) -> u64 {
        self.connection_id
    }

    /// Transaction ID supplied with a request to uniquely identify a response.
    #[must_use]
    pub fn transaction_id(&self) -> u32 {
        self.transaction_id
    }

    /// Actual type of request that this `TrackerRequest` represents.
    #[must_use]
    pub fn request_type(&self) -> &RequestType<'_> {
        &self.request_type
    }

    /// Create an owned version of the `TrackerRequest`.
    #[must_use]
    pub fn to_owned(&self) -> TrackerRequest<'static> {
        TrackerRequest {
            connection_id: self.connection_id,
            transaction_id: self.transaction_id,
            request_type: self.request_type.to_owned(),
        }
    }
}

fn parse_request(bytes: &[u8]) -> IResult<&[u8], TrackerRequest<'_>> {
    let (remaining, (connection_id, action_id, transaction_id)) = tuple((be_u64, be_u32, be_u32))(bytes)?;

    match (connection_id, action_id) {
        (CONNECT_ID_PROTOCOL_ID, crate::CONNECT_ACTION_ID) => Ok((
            remaining,
            TrackerRequest::new(CONNECT_ID_PROTOCOL_ID, transaction_id, RequestType::Connect),
        )),
        (cid, crate::ANNOUNCE_IPV4_ACTION_ID) => {
            let (remaining, ann_req) = AnnounceRequest::from_bytes_v4(remaining)?;
            Ok((
                remaining,
                TrackerRequest::new(cid, transaction_id, RequestType::Announce(ann_req)),
            ))
        }
        (cid, crate::SCRAPE_ACTION_ID) => {
            let (remaining, scr_req) = ScrapeRequest::from_bytes(remaining)?;
            Ok((
                remaining,
                TrackerRequest::new(cid, transaction_id, RequestType::Scrape(scr_req)),
            ))
        }
        (cid, crate::ANNOUNCE_IPV6_ACTION_ID) => {
            let (remaining, ann_req) = AnnounceRequest::from_bytes_v6(remaining)?;
            Ok((
                remaining,
                TrackerRequest::new(cid, transaction_id, RequestType::Announce(ann_req)),
            ))
        }
        _ => Err(nom::Err::Error(nom::error::Error::new(
            remaining,
            nom::error::ErrorKind::Switch,
        ))),
    }
}
