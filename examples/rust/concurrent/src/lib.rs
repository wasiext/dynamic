use bindings::exports::wasiext::dynamic_examples::concurrent;
use bindings::wasi::sockets::network::Ipv4SocketAddress;
use bindings::wasi::sockets::network::{IpAddressFamily, IpSocketAddress, Network};
use bindings::wasi::sockets::tcp::TcpSocket;
use bindings::wasi::sockets::tcp_create_socket;
use bindings::wasi::{io::poll::Pollable, sockets::instance_network::instance_network};
use bindings::wasiext::dynamic::types::AsyncReturn;

mod bindings {
    use super::Component;

    wit_bindgen::generate!({ generate_all });

    export!(Component);
}

struct HelloReturn {
    sock: TcpSocket,
}

struct Component;

impl bindings::exports::wasiext::dynamic_examples::concurrent::GuestHelloReturn for HelloReturn {
    fn register_dynamic_type() -> AsyncReturn {
        unreachable!()
    }

    fn subscribe(&self) -> Pollable {
        self.sock.subscribe()
    }

    fn await_(this: concurrent::HelloReturn) -> String {
        let (rx, tx) = this
            .into_inner::<HelloReturn>()
            .sock
            .finish_connect()
            .expect("failed to connect");
        let buf = rx.blocking_read(4).expect("failed to read 4 bytes");
        drop(tx);
        drop(rx);
        String::from_utf8_lossy(&buf).to_string()
    }
}

impl bindings::exports::wasiext::dynamic_examples::concurrent::Guest for Component {
    type HelloReturn = HelloReturn;

    fn hello() -> concurrent::HelloReturn {
        let sock = tcp_create_socket::create_tcp_socket(IpAddressFamily::Ipv4)
            .expect("failed to create socket");
        let net = instance_network();
        sock.start_connect(
            &net,
            IpSocketAddress::Ipv4(Ipv4SocketAddress {
                port: 8080,
                address: (127, 0, 0, 1),
            }),
        )
        .expect("failed to start connect");
        concurrent::HelloReturn::new(HelloReturn { sock })
    }
}
