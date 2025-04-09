// SPDX-License-Identifier: MPL-2.0

mod ext;
mod init;
mod poll;
mod sched;

pub use init::{init, IFACES};
pub use poll::lazy_init;

pub type Iface = dyn astros_bigtcp::iface::Iface<ext::BigtcpExt>;
pub type BoundPort = astros_bigtcp::iface::BoundPort<ext::BigtcpExt>;

pub type RawTcpSocketExt = astros_bigtcp::socket::RawTcpSocketExt<ext::BigtcpExt>;

pub type TcpConnection = astros_bigtcp::socket::TcpConnection<ext::BigtcpExt>;
pub type TcpListener = astros_bigtcp::socket::TcpListener<ext::BigtcpExt>;
pub type UdpSocket = astros_bigtcp::socket::UdpSocket<ext::BigtcpExt>;
