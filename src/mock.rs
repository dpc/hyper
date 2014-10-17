use std::fmt;
use std::io::{IoResult, MemReader, MemWriter};
use std::io::net::ip::{SocketAddr, ToSocketAddr};

use net::{NetworkStream, NetworkConnector};

pub struct MockStream {
    pub read: MemReader,
    pub write: MemWriter,
}

impl Clone for MockStream {
    fn clone(&self) -> MockStream {
        MockStream {
            read: MemReader::new(self.read.get_ref().to_vec()),
            write: MemWriter::from_vec(self.write.get_ref().to_vec()),
        }
    }
}

impl PartialEq for MockStream {
    fn eq(&self, other: &MockStream) -> bool {
        self.read.get_ref() == other.read.get_ref() &&
            self.write.get_ref() == other.write.get_ref()
    }
}

impl fmt::Show for MockStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MockStream {{ read: {}, write: {} }}",
               self.read.get_ref(), self.write.get_ref())
    }

}

impl MockStream {
    pub fn new() -> MockStream {
        MockStream {
            read: MemReader::new(vec![]),
            write: MemWriter::new(),
        }
    }

    pub fn with_input(input: &[u8]) -> MockStream {
        MockStream {
            read: MemReader::new(input.to_vec()),
            write: MemWriter::new(),
        }
    }
}
impl Reader for MockStream {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        self.read.read(buf)
    }
}

impl Writer for MockStream {
    fn write(&mut self, msg: &[u8]) -> IoResult<()> {
        self.write.write(msg)
    }
}

impl NetworkStream for MockStream {

    fn peer_name(&mut self) -> IoResult<SocketAddr> {
        Ok(from_str("127.0.0.1:1337").unwrap())
    }
}

impl NetworkConnector for MockStream {
    fn connect<To: ToSocketAddr>(_addr: To, _scheme: &str) -> IoResult<MockStream> {
        Ok(MockStream::new())
    }
}

/// new connectors must be created if you wish to intercept requests.
macro_rules! mock_connector (
    ($name:ident {
        $($url:expr => $res:expr)*
    }) => (
        struct $name {
            rcvr: ::std::io::MemWriter,
            res: ::std::io::BufReader<'static>,
        }

        impl Clone for $name {
            fn clone(&self) -> $name {
                panic!("cant clone BufReader")
            }
        }

        impl ::net::NetworkStream for $name {
            fn peer_name(&mut self) -> ::std::io::IoResult<::std::io::net::ip::SocketAddr> {
                Ok(from_str("127.0.0.1:1337").unwrap())
            }
        }

        impl ::net::NetworkConnector for $name {
            fn connect<To: ::std::io::net::ip::ToSocketAddr>(addr: To, scheme: &str) -> ::std::io::IoResult<$name> {
                use std::collections::HashMap;
                let addr = addr.to_socket_addr().unwrap();
                debug!("MockStream::connect({}, {})", addr, scheme);
                let mut map = HashMap::new();
                $(map.insert($url, $res);)*


                let key = format!("{}://{}", scheme, addr.ip);
                // ignore port for now
                match map.find(&key[]) {
                    Some(res) => Ok($name {
                        rcvr: ::std::io::MemWriter::new(),
                        res: ::std::io::BufReader::new(res.as_bytes())
                    }),
                    None => panic!("mock stream doesn't know url")
                }
            }

        }

        impl Reader for $name {
            fn read(&mut self, buf: &mut [u8]) -> ::std::io::IoResult<uint> {
                self.res.read(buf)
            }
        }

        impl Writer for $name {
            fn write(&mut self, msg: &[u8]) -> ::std::io::IoResult<()> {
                self.rcvr.write(msg)
            }
        }
    )
)
