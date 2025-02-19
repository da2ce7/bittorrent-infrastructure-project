use common::{handshaker, tracing_stderr_init, MockTrackerHandler, DEFAULT_TIMEOUT, INIT, LOOPBACK_IPV4};
use futures::StreamExt as _;
use tracing::level_filters::LevelFilter;
use util::bt::{self};
use utracker::announce::{AnnounceEvent, ClientState};
use utracker::{ClientRequest, HandshakerMessage, TrackerClient, TrackerServer};

mod common;

#[tokio::test]
async fn positive_announce_stopped() {
    INIT.call_once(|| {
        tracing_stderr_init(LevelFilter::ERROR);
    });

    let (sink, mut stream) = handshaker();

    let mock_handler = MockTrackerHandler::new();
    let server = TrackerServer::run(LOOPBACK_IPV4, mock_handler).unwrap();

    let mut client = TrackerClient::run(LOOPBACK_IPV4, sink, None).unwrap();

    let info_hash = [0u8; bt::INFO_HASH_LEN].into();

    // Started
    {
        let _send_token = client
            .request(
                server.local_addr(),
                ClientRequest::Announce(info_hash, ClientState::new(0, 0, 0, AnnounceEvent::Started)),
            )
            .unwrap();

        let _init_msg = match tokio::time::timeout(DEFAULT_TIMEOUT, stream.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap()
        {
            HandshakerMessage::InitiateMessage(message) => message,
            HandshakerMessage::ClientMetadata(_) => unreachable!(),
        };

        let metadata = match tokio::time::timeout(DEFAULT_TIMEOUT, stream.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap()
        {
            HandshakerMessage::InitiateMessage(_) => unreachable!(),
            HandshakerMessage::ClientMetadata(metadata) => metadata,
        };

        let response = metadata.result().as_ref().unwrap().announce_response().unwrap();
        assert_eq!(response.leechers(), 1);
        assert_eq!(response.seeders(), 1);
        assert_eq!(response.peers().iter().count(), 1);
    }

    // Stopped
    {
        let _send_token = client
            .request(
                server.local_addr(),
                ClientRequest::Announce(info_hash, ClientState::new(0, 0, 0, AnnounceEvent::Stopped)),
            )
            .unwrap();

        let metadata = match tokio::time::timeout(DEFAULT_TIMEOUT, stream.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap()
        {
            HandshakerMessage::InitiateMessage(_) => unreachable!(),
            HandshakerMessage::ClientMetadata(metadata) => metadata,
        };

        let response = metadata.result().as_ref().unwrap().announce_response().unwrap();
        assert_eq!(response.leechers(), 0);
        assert_eq!(response.seeders(), 0);
        assert_eq!(response.peers().iter().count(), 0);
    }
}
