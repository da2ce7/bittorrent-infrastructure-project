//! Generic `PeerProtocol` implementations.

pub mod extension;
pub mod null;
pub mod unit;
pub mod wire;

/// Trait for implementing a bittorrent protocol message.
#[allow(clippy::module_name_repetitions)]
pub trait PeerProtocol {
    /// Type of message the protocol operates with.
    type ProtocolMessage;
    type ProtocolMessageError;

    /// Total number of bytes needed to parse a complete message. This is not
    /// in addition to what we were given, this is the total number of bytes, so
    /// if the given bytes has length >= needed, then we can parse it.
    ///
    /// If none is returned, it means we need more bytes to determine the number
    /// of bytes needed. If an error is returned, it means the connection should
    /// be dropped, as probably the message exceeded some maximum length.
    ///
    /// # Errors
    ///
    /// This function will return an IO result if unable to calculate the bytes needed.
    fn bytes_needed(&mut self, bytes: &[u8]) -> std::io::Result<Option<usize>>;

    /// Parse a `ProtocolMessage` from the given bytes.
    ///
    /// # Errors
    ///
    /// This function will return an IO error if unable to parse the bytes into a [`Self::ProtocolMessage`].
    fn parse_bytes(&mut self, bytes: &[u8]) -> std::io::Result<Result<Self::ProtocolMessage, Self::ProtocolMessageError>>;

    /// Write a `ProtocolMessage` to the given writer.
    ///
    /// # Errors
    ///
    /// This function will return an error if it fails to write-out.
    fn write_bytes<W>(
        &mut self,
        item: &Result<Self::ProtocolMessage, Self::ProtocolMessageError>,
        writer: W,
    ) -> std::io::Result<usize>
    where
        W: std::io::Write;

    /// Retrieve how many bytes the message will occupy on the wire.
    ///
    /// # Errors
    ///
    /// This function will return an error if unable to calculate the message length.
    fn message_size(&mut self, message: &Result<Self::ProtocolMessage, Self::ProtocolMessageError>) -> std::io::Result<usize>;
}

/// Trait for nested peer protocols to see higher level peer protocol messages.
///
/// This is useful when tracking certain states of a connection that happen at a higher
/// level peer protocol, but which nested peer protocols need to know about atomically
/// (before other messages dependent on that message start coming in, and the nested
/// protocol is expected to handle those).
///
/// Example: We handle `ExtensionMessage`s at the `PeerWireProtocol` layer, but the
/// `ExtensionMessage` contains mappings of id to message type that nested extensions
/// need to know about so they can determine what type of message a given id maps to.
/// We need to pass the `ExtensionMessage` down to them before we start receiving
/// messages with those ids (otherwise we will receive unrecognized messages and
/// kill the connection).
#[allow(clippy::module_name_repetitions)]
pub trait NestedPeerProtocol<M> {
    /// Notify a nested protocol that we have received the given message.
    fn received_message(&mut self, message: &M) -> usize;

    /// Notify a nested protocol that we have sent the given message.
    fn sent_message(&mut self, message: &M) -> usize;
}
