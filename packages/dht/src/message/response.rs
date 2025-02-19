use bencode::ext::BConvertExt;
use bencode::{BConvert, BDictAccess, BListAccess, BRefAccess, BencodeConvertError};
use util::bt::NodeId;

use crate::error::DhtError;
use crate::message::announce_peer::AnnouncePeerResponse;
use crate::message::compact_info::{CompactNodeInfo, CompactValueInfo};
use crate::message::find_node::FindNodeResponse;
use crate::message::get_peers::GetPeersResponse;
use crate::message::ping::PingResponse;

pub const RESPONSE_ARGS_KEY: &str = "r";

// ----------------------------------------------------------------------------//

#[allow(clippy::module_name_repetitions)]
pub struct ResponseValidate<'a> {
    trans_id: &'a [u8],
}

impl<'a> ResponseValidate<'a> {
    #[must_use]
    pub fn new(trans_id: &'a [u8]) -> ResponseValidate<'a> {
        ResponseValidate { trans_id }
    }

    /// Validate and deserialize bytes into a `NodeId`
    ///
    /// # Errors
    ///
    /// This function will return an error if to generate the `NodeId`.
    pub fn validate_node_id(&self, node_id: &[u8]) -> Result<NodeId, DhtError> {
        NodeId::from_hash(node_id).map_err(|_| DhtError::InvalidResponse {
            details: format!(
                "TID {:?} Found Node ID With Invalid Length {:?}",
                self.trans_id,
                node_id.len()
            ),
        })
    }

    /// Validate and deserialize bytes into a `CompactNodeInfo`
    ///
    /// # Errors
    ///
    /// This function will return an error if to generate the `CompactNodeInfo`.
    pub fn validate_nodes<'b>(&self, nodes: &'b [u8]) -> Result<CompactNodeInfo<'b>, DhtError> {
        CompactNodeInfo::new(nodes).map_err(|_| DhtError::InvalidResponse {
            details: format!(
                "TID {:?} Found Nodes Structure With {} Number Of Bytes Instead \
                                  Of Correct Multiple",
                self.trans_id,
                nodes.len()
            ),
        })
    }

    /// Validate and deserialize bytes into a `CompactValueInfo`
    ///
    /// # Errors
    ///
    /// This function will return an error if to generate the `CompactValueInfo`.
    pub fn validate_values<'b, B>(&self, values: &'b dyn BListAccess<B::BType>) -> Result<CompactValueInfo<'b, B>, DhtError>
    where
        B: BRefAccess<BType = B> + Clone,
        B::BType: PartialEq + Eq + core::hash::Hash + std::fmt::Debug,
    {
        for bencode in values {
            match bencode.bytes() {
                Some(_) => (),
                None => {
                    return Err(DhtError::InvalidResponse {
                        details: format!("TID {:?} Found Values Structure As Non Bytes Type", self.trans_id),
                    })
                }
            }
        }

        CompactValueInfo::new(values).map_err(|_| DhtError::InvalidResponse {
            details: format!("TID {:?} Found Values Structure With Wrong Number Of Bytes", self.trans_id),
        })
    }
}

impl<'a> BConvert for ResponseValidate<'a> {
    type Error = DhtError;

    fn handle_error(&self, error: BencodeConvertError) -> DhtError {
        error.into()
    }
}

impl<'a> BConvertExt for ResponseValidate<'a> {}

// ----------------------------------------------------------------------------//

#[allow(clippy::module_name_repetitions)]
#[allow(unused)]
pub enum ExpectedResponse {
    Ping,
    FindNode,
    GetPeers,
    AnnouncePeer,
    GetData,
    PutData,
    None,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ResponseType<'a, B>
where
    B: BRefAccess<BType = B> + Clone,
    B::BType: PartialEq + Eq + core::hash::Hash + std::fmt::Debug,
{
    Ping(PingResponse<'a>),
    FindNode(FindNodeResponse<'a>),
    GetPeers(GetPeersResponse<'a, B>),
    AnnouncePeer(AnnouncePeerResponse<'a>), /* GetData(GetDataResponse<'a>),
                                             * PutData(PutDataResponse<'a>) */
}

impl<'a, B> ResponseType<'a, B>
where
    B: BRefAccess<BType = B> + Clone,
    B::BType: PartialEq + Eq + core::hash::Hash + std::fmt::Debug,
{
    /// Creates a new `ResponseType` from parts.
    ///
    /// # Errors
    ///
    /// This function will return an error if unable to lookup, convert, and generate correct type.
    pub fn from_parts(
        root: &'a dyn BDictAccess<B::BKey, B>,
        trans_id: &'a [u8],
        rsp_type: &ExpectedResponse,
    ) -> Result<ResponseType<'a, B>, DhtError>
    where
        B: BRefAccess<BType = B>,
    {
        let validate = ResponseValidate::new(trans_id);
        let rqst_root = validate.lookup_and_convert_dict(root, RESPONSE_ARGS_KEY)?;

        match rsp_type {
            ExpectedResponse::Ping => {
                let ping_rsp = PingResponse::from_parts(rqst_root, trans_id)?;
                Ok(ResponseType::Ping(ping_rsp))
            }
            ExpectedResponse::FindNode => {
                let find_node_rsp = FindNodeResponse::from_parts(rqst_root, trans_id)?;
                Ok(ResponseType::FindNode(find_node_rsp))
            }
            ExpectedResponse::GetPeers => {
                let get_peers_rsp = GetPeersResponse::<B>::from_parts(rqst_root, trans_id)?;
                Ok(ResponseType::GetPeers(get_peers_rsp))
            }
            ExpectedResponse::AnnouncePeer => {
                let announce_peer_rsp = AnnouncePeerResponse::from_parts(rqst_root, trans_id)?;
                Ok(ResponseType::AnnouncePeer(announce_peer_rsp))
            }
            ExpectedResponse::GetData => {
                unimplemented!();
            }
            ExpectedResponse::PutData => {
                unimplemented!();
            }
            ExpectedResponse::None => Err(DhtError::UnsolicitedResponse),
        }
    }
}
