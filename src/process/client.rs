use std::{os::unix::net::UnixStream, path::Path};

use super::command_grpc::HandleProcessClient;

struct Client<'a> {
    socket_path: &'a Path,
}

impl<'a> Client<'a> {
    pub fn new(socket_path: &str) -> Self {
        let socket = Path::new(socket_path);

        let mut stream = match UnixStream::connect(&socket) {
            Err(e) => panic!("connect to socket {} failed: {}", socket_path, e),
            Ok(stream) => stream,
        };

        Self { socket_path }
    }
}
