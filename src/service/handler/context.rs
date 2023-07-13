use std::net::SocketAddr;
use crate::packet::data_packet::DataPacket;
use crate::packet::data_transmission::DataTransmit;
use crate::service::context::data_service_context::DataServiceContext;

pub struct HandlerContext<'a> {
    tt: &'a mut DataTransmit,
    packet: &'a DataPacket,
    socket_addr: SocketAddr,
    data_service_context: &'a DataServiceContext,
}

impl<'a> HandlerContext<'a> {
    pub fn new(
        tt: &'a mut DataTransmit,
        packet: &'a DataPacket,
        socket_addr: SocketAddr,
        data_service_context: &'a DataServiceContext,
    ) -> Self {
        Self {
            tt,
            packet,
            socket_addr,
            data_service_context,
        }
    }

    pub fn tt(&mut self) -> &mut DataTransmit {
        self.tt
    }

    pub fn packet(&self) -> &DataPacket {
        self.packet
    }

    pub fn socket_addr(&self) -> SocketAddr {
        self.socket_addr
    }

    pub fn data_service_context(&self) -> &DataServiceContext {
        self.data_service_context
    }
}

pub enum ConnectionControl {
    Default,
    CloseConnection,
}
