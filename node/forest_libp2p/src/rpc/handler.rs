use super::codec::RPCError;
use super::protocol::*;
use super::{InboundCodec, OutboundCodec, RPCEvent, RPCRequest};
use futures::prelude::*;
use futures::{AsyncRead, AsyncWrite};
use futures_codec::Framed;
use libp2p::core::Negotiated;
use libp2p::swarm::{
    KeepAlive, ProtocolsHandler, ProtocolsHandlerEvent, ProtocolsHandlerUpgrErr, SubstreamProtocol,
};
use libp2p::{InboundUpgrade, OutboundUpgrade};
use smallvec::SmallVec;
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

/// The time (in seconds) before a substream that is awaiting a response from the user times out.
pub const RESPONSE_TIMEOUT: u64 = 10;

struct RPCHandler<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite,
{
    /// Upgrade configuration for the gossipsub protocol.
    listen_protocol: SubstreamProtocol<RPCProtocol>,

    /// The single long-lived outbound substream.
    outbound_substream: Option<OutboundSubstreamState<TSubstream>>,

    /// The single long-lived inbound substream.
    inbound_substream: Option<InboundSubstreamState<TSubstream>>,

    /// Queue of values that we want to send to the remote.
    send_queue: SmallVec<[RPCRequest; 16]>,

    /// Flag determining whether to maintain the connection to the peer.
    keep_alive: KeepAlive,
}

/// State of the inbound substream, opened either by us or by the remote.
enum InboundSubstreamState<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite,
{
    /// Waiting for a message from the remote. The idle state for an inbound substream.
    WaitingInput(Framed<Negotiated<TSubstream>, InboundCodec>),
    /// The substream is being closed.
    Closing(Framed<Negotiated<TSubstream>, InboundCodec>),
    /// An error occurred during processing.
    Poisoned,
}

/// State of the outbound substream, opened either by us or by the remote.
enum OutboundSubstreamState<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite,
{
    /// Waiting for the user to send a message. The idle state for an outbound substream.
    WaitingOutput(Framed<Negotiated<TSubstream>, OutboundCodec>),
    /// Waiting to send a message to the remote.
    PendingSend(Framed<Negotiated<TSubstream>, OutboundCodec>, RPCRequest),
    /// Waiting to flush the substream so that the data arrives to the remote.
    PendingFlush(Framed<Negotiated<TSubstream>, OutboundCodec>),
    /// The substream is being closed. Used by either substream.
    _Closing(Framed<Negotiated<TSubstream>, OutboundCodec>),
    /// An error occurred during processing.
    Poisoned,
}

impl<TSubstream> ProtocolsHandler for RPCHandler<TSubstream>
where
    TSubstream: AsyncWrite + AsyncRead + Unpin + Send + 'static,
{
    type InEvent = RPCEvent;
    type OutEvent = RPCEvent;
    type Error = RPCError;
    type Substream = TSubstream;
    type InboundProtocol = RPCProtocol;
    type OutboundProtocol = RPCProtocol;
    type OutboundOpenInfo = RPCRequest;

    fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol> {
        self.listen_protocol.clone()
    }

    fn inject_fully_negotiated_inbound(
        &mut self,
        substream: <Self::InboundProtocol as InboundUpgrade<Negotiated<Self::Substream>>>::Output,
    ) {
        println!("Negotiated inbound");
        self.inbound_substream = Some(InboundSubstreamState::WaitingInput(substream));
    }

    fn inject_fully_negotiated_outbound(
        &mut self,
        substream: <Self::OutboundProtocol as OutboundUpgrade<Negotiated<TSubstream>>>::Output,
        message: Self::OutboundOpenInfo,
    ) {
        // Should never establish a new outbound substream if one already exists.
        // If this happens, an outbound message is not sent.
        if self.outbound_substream.is_some() {
            println!("Established an outbound substream with one already available");
            // Add the message back to the send queue
            self.send_queue.push(message);
        } else {
            self.outbound_substream = Some(OutboundSubstreamState::PendingSend(substream, message));
        }
    }

    fn inject_event(&mut self, message: Self::InEvent) {
        if let RPCEvent::Request(r) = message {
            println!("Adding event to queue");
            self.send_queue.push(r);
        } else {
            // TODO remove this
            panic!("This may happen but shouldn't");
        }
    }

    fn inject_dial_upgrade_error(
        &mut self,
        _: Self::OutboundOpenInfo,
        _: ProtocolsHandlerUpgrErr<
            <Self::OutboundProtocol as OutboundUpgrade<Self::Substream>>::Error,
        >,
    ) {
        // Can maybe ignore
        println!("Ignoring dial error");
    }

    fn connection_keep_alive(&self) -> KeepAlive {
        self.keep_alive
    }

    fn poll(
        &mut self,
        cx: &mut Context,
    ) -> Poll<
        ProtocolsHandlerEvent<
            Self::OutboundProtocol,
            Self::OutboundOpenInfo,
            Self::OutEvent,
            Self::Error,
        >,
    > {
        // determine if we need to create the stream
        if !self.send_queue.is_empty() && self.outbound_substream.is_none() {
            let message = self.send_queue.remove(0);
            self.send_queue.shrink_to_fit();
            return Poll::Ready(ProtocolsHandlerEvent::OutboundSubstreamRequest {
                protocol: self.listen_protocol.clone(),
                info: message,
            });
        }

        loop {
            match std::mem::replace(
                &mut self.inbound_substream,
                Some(InboundSubstreamState::Poisoned),
            ) {
                // inbound idle state
                Some(InboundSubstreamState::WaitingInput(mut substream)) => {
                    match substream.poll_next_unpin(cx) {
                        Poll::Ready(Some(Ok(message))) => {
                            self.inbound_substream =
                                Some(InboundSubstreamState::WaitingInput(substream));
                            return Poll::Ready(ProtocolsHandlerEvent::Custom(RPCEvent::Request(
                                message,
                            )));
                        }
                        Poll::Ready(Some(Err(e))) => {
                            println!("Inbound substream error while awaiting input: {:?}", e);
                            self.inbound_substream =
                                Some(InboundSubstreamState::Closing(substream));
                        }
                        // peer closed the stream
                        Poll::Ready(None) => {
                            self.inbound_substream =
                                Some(InboundSubstreamState::Closing(substream));
                        }
                        Poll::Pending => {
                            self.inbound_substream =
                                Some(InboundSubstreamState::WaitingInput(substream));
                            break;
                        }
                    }
                }
                Some(InboundSubstreamState::Closing(mut substream)) => {
                    match Sink::poll_close(Pin::new(&mut substream), cx) {
                        Poll::Ready(res) => {
                            if let Err(e) = res {
                                // Don't close the connection but just drop the inbound substream.
                                // In case the remote has more to send, they will open up a new
                                // substream.
                                println!("Inbound substream error while closing: {:?}", e);
                            }

                            self.inbound_substream = None;
                            if self.outbound_substream.is_none() {
                                self.keep_alive = KeepAlive::No;
                            }
                            break;
                        }
                        Poll::Pending => {
                            self.inbound_substream =
                                Some(InboundSubstreamState::Closing(substream));
                            break;
                        }
                    }
                }
                None => {
                    self.inbound_substream = None;
                    break;
                }
                Some(InboundSubstreamState::Poisoned) => {
                    panic!("Error occurred during inbound stream processing")
                }
            }
        }

        loop {
            match std::mem::replace(
                &mut self.outbound_substream,
                Some(OutboundSubstreamState::Poisoned),
            ) {
                // outbound idle state
                Some(OutboundSubstreamState::WaitingOutput(substream)) => {
                    if !self.send_queue.is_empty() {
                        let message = self.send_queue.remove(0);
                        self.send_queue.shrink_to_fit();
                        self.outbound_substream =
                            Some(OutboundSubstreamState::PendingSend(substream, message));
                    } else {
                        self.outbound_substream =
                            Some(OutboundSubstreamState::WaitingOutput(substream));
                        break;
                    }
                }
                Some(OutboundSubstreamState::PendingSend(mut substream, message)) => {
                    match Sink::poll_ready(Pin::new(&mut substream), cx) {
                        Poll::Ready(Ok(())) => {
                            match Sink::start_send(Pin::new(&mut substream), message) {
                                Ok(()) => {
                                    self.outbound_substream =
                                        Some(OutboundSubstreamState::PendingFlush(substream))
                                }
                                Err(e) => {
                                    return Poll::Ready(ProtocolsHandlerEvent::Close(e));
                                }
                            }
                        }
                        Poll::Ready(Err(e)) => {
                            println!("Outbound substream error while sending output: {:?}", e);
                            return Poll::Ready(ProtocolsHandlerEvent::Close(e));
                        }
                        Poll::Pending => {
                            self.outbound_substream =
                                Some(OutboundSubstreamState::PendingSend(substream, message));
                            break;
                        }
                    }
                }
                Some(OutboundSubstreamState::PendingFlush(mut substream)) => {
                    match Sink::poll_flush(Pin::new(&mut substream), cx) {
                        Poll::Ready(Ok(())) => {
                            self.outbound_substream =
                                Some(OutboundSubstreamState::WaitingOutput(substream))
                        }
                        Poll::Ready(Err(e)) => return Poll::Ready(ProtocolsHandlerEvent::Close(e)),
                        Poll::Pending => {
                            self.outbound_substream =
                                Some(OutboundSubstreamState::PendingFlush(substream));
                            break;
                        }
                    }
                }
                // Currently never used - manual shutdown may implement this in the future
                Some(OutboundSubstreamState::_Closing(mut substream)) => {
                    match Sink::poll_close(Pin::new(&mut substream), cx) {
                        Poll::Ready(Ok(())) => {
                            self.outbound_substream = None;
                            if self.inbound_substream.is_none() {
                                self.keep_alive = KeepAlive::No;
                            }
                            break;
                        }
                        Poll::Ready(Err(e)) => {
                            println!("Outbound substream error while closing: {:?}", e);
                            return Poll::Ready(ProtocolsHandlerEvent::Close(
                                io::Error::new(
                                    io::ErrorKind::BrokenPipe,
                                    "Failed to close outbound substream",
                                )
                                .into(),
                            ));
                        }
                        Poll::Pending => {
                            self.outbound_substream =
                                Some(OutboundSubstreamState::_Closing(substream));
                            break;
                        }
                    }
                }
                None => {
                    self.outbound_substream = None;
                    break;
                }
                Some(OutboundSubstreamState::Poisoned) => {
                    panic!("Error occurred during outbound stream processing")
                }
            }
        }

        Poll::Pending
    }
}
