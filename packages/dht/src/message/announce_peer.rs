// TODO: Remove this when announces are implemented
#![allow(unused)]

use bencode::{ben_bytes, ben_int, ben_map, BConvert, BDictAccess, BRefAccess};
use util::bt::{InfoHash, NodeId};

use crate::error::DhtError;
use crate::message;
use crate::message::request::{self, RequestValidate};

const PORT_KEY: &str = "port";
const IMPLIED_PORT_KEY: &str = "implied_port";

// TODO: Integrate the Token type into the request message.

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ConnectPort {
    Implied,
    Explicit(u16),
}

#[allow(clippy::module_name_repetitions)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct AnnouncePeerRequest<'a> {
    trans_id: &'a [u8],
    node_id: NodeId,
    info_hash: InfoHash,
    token: &'a [u8],
    port: ConnectPort,
}

impl<'a> AnnouncePeerRequest<'a> {
    #[must_use]
    pub fn new(
        trans_id: &'a [u8],
        node_id: NodeId,
        info_hash: InfoHash,
        token: &'a [u8],
        port: ConnectPort,
    ) -> AnnouncePeerRequest<'a> {
        AnnouncePeerRequest {
            trans_id,
            node_id,
            info_hash,
            token,
            port,
        }
    }

    /// Generate a  `AnnouncePeerRequest` from parts
    ///
    /// # Errors
    ///
    /// This function will return an error unable to get bytes unable do lookup.
    pub fn from_parts<B>(
        rqst_root: &'a dyn BDictAccess<B::BKey, B>,
        trans_id: &'a [u8],
    ) -> Result<AnnouncePeerRequest<'a>, DhtError>
    where
        B: BRefAccess,
    {
        let validate = RequestValidate::new(trans_id);

        let node_id_bytes = validate.lookup_and_convert_bytes(rqst_root, message::NODE_ID_KEY)?;
        let node_id = validate.validate_node_id(node_id_bytes)?;

        let info_hash_bytes = validate.lookup_and_convert_bytes(rqst_root, message::INFO_HASH_KEY)?;
        let info_hash = validate.validate_info_hash(info_hash_bytes)?;

        let token = validate.lookup_and_convert_bytes(rqst_root, message::TOKEN_KEY)?;
        let port = validate.lookup_and_convert_int(rqst_root, PORT_KEY);

        // Technically, the specification says that the value is either 0 or 1 but goes on to say that
        // if it is not zero, then the source port should be used. We will allow values other than 0 or 1.
        let response_port = match rqst_root.lookup(IMPLIED_PORT_KEY.as_bytes()).map(bencode::BRefAccess::int) {
            Some(Some(n)) if n != 0 => ConnectPort::Implied,
            _ => {
                // If we hit this, the port either was not provided or it was of the wrong bencode type
                #[allow(clippy::cast_possible_truncation)]
                let port_number = (port?).unsigned_abs() as u16;
                ConnectPort::Explicit(port_number)
            }
        };

        Ok(AnnouncePeerRequest::new(trans_id, node_id, info_hash, token, response_port))
    }

    #[must_use]
    pub fn transaction_id(&self) -> &'a [u8] {
        self.trans_id
    }

    #[must_use]
    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    #[must_use]
    pub fn info_hash(&self) -> InfoHash {
        self.info_hash
    }

    #[must_use]
    pub fn token(&self) -> &'a [u8] {
        self.token
    }

    #[must_use]
    pub fn connect_port(&self) -> ConnectPort {
        self.port
    }

    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        // In case a client errors out when the port key is not present, even when
        // implied port is specified, we will provide a dummy value in that case.
        let (displayed_port, implied_value) = match self.port {
            ConnectPort::Implied => (0, 1),
            ConnectPort::Explicit(n) => (n, 0),
        };

        (ben_map! {
            //message::CLIENT_TYPE_KEY => ben_bytes!(dht::CLIENT_IDENTIFICATION),
            message::TRANSACTION_ID_KEY => ben_bytes!(self.trans_id),
            message::MESSAGE_TYPE_KEY => ben_bytes!(message::REQUEST_TYPE_KEY),
            message::REQUEST_TYPE_KEY => ben_bytes!(request::ANNOUNCE_PEER_TYPE_KEY),
            request::REQUEST_ARGS_KEY => ben_map!{
                message::NODE_ID_KEY => ben_bytes!(self.node_id.as_ref()),
                IMPLIED_PORT_KEY => ben_int!(implied_value),
                message::INFO_HASH_KEY => ben_bytes!(self.info_hash.as_ref()),
                PORT_KEY => ben_int!(i64::from(displayed_port)),
                message::TOKEN_KEY => ben_bytes!(self.token)
            }
        })
        .encode()
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct AnnouncePeerResponse<'a> {
    trans_id: &'a [u8],
    node_id: NodeId,
}

impl<'a> AnnouncePeerResponse<'a> {
    #[must_use]
    pub fn new(trans_id: &'a [u8], node_id: NodeId) -> AnnouncePeerResponse<'a> {
        AnnouncePeerResponse { trans_id, node_id }
    }

    /// Generate a  `AnnouncePeerResponse` from parts
    ///
    /// # Errors
    ///
    /// This function will return an error unable to get bytes or unable to validate the node id.
    pub fn from_parts<B>(
        rqst_root: &dyn BDictAccess<B::BKey, B>,
        trans_id: &'a [u8],
    ) -> Result<AnnouncePeerResponse<'a>, DhtError>
    where
        B: BRefAccess,
    {
        let validate = RequestValidate::new(trans_id);

        let node_id_bytes = validate.lookup_and_convert_bytes(rqst_root, message::NODE_ID_KEY)?;
        let node_id = validate.validate_node_id(node_id_bytes)?;

        Ok(AnnouncePeerResponse::new(trans_id, node_id))
    }

    #[must_use]
    pub fn transaction_id(&self) -> &'a [u8] {
        self.trans_id
    }

    #[must_use]
    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        (ben_map! {
            //message::CLIENT_TYPE_KEY => ben_bytes!(dht::CLIENT_IDENTIFICATION),
            message::TRANSACTION_ID_KEY => ben_bytes!(self.trans_id),
            message::MESSAGE_TYPE_KEY => ben_bytes!(message::RESPONSE_TYPE_KEY),
            message::RESPONSE_TYPE_KEY => ben_map!{
                message::NODE_ID_KEY => ben_bytes!(self.node_id.as_ref())
            }
        })
        .encode()
    }
}
