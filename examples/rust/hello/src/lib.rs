mod bindings {
    use super::Component;

    wit_bindgen::generate!({ generate_all });

    export!(Component);
}

use bindings::wasiext::dynamic_examples::hello::hello;
use bindings::wasiext::dynamic_examples::types::{
    StringFuture, StringFutureReturn, StringStream, StringStreamReturn,
};

struct Component;

impl bindings::exports::wasiext::dynamic_examples::hello::Guest for Component {
    fn hello(name: &StringFuture) -> StringFutureReturn {
        StringFuture::register_dynamic_type();
        StringFutureReturn::register_dynamic_type();

        let ret_fut = StringFuture::new();

        let ret = StringFutureReturn::new(&ret_fut);

        let hello_tx = StringFuture::new();
        let hello_rx = hello(&hello_tx);

        // wait for name to arrive
        // Optionally, get a pollable:
        //name.subscribe().block();
        let name = name.await_().expect("future sender dropped");
        let hello_tx_send = StringFuture::send(hello_tx, &name);

        // Optionally, get a pollable:
        //hello_tx_send.subscribe().block();
        assert!(hello_tx_send.await_(), "future receiver dropped");
        // Optionally, get a pollable:
        //hello_rx.subscribe().block();
        let name = StringFutureReturn::await_(hello_rx)
            .await_()
            .expect("future sender dropped");

        // send `name` on the future
        let send = StringFuture::send(ret_fut, &name);
        // Optionally, get a pollable:
        //send.subscribe().block();
        assert!(send.await_(), "future receiver dropped");

        ret
    }

    fn hello_stream(names: &StringStream) -> StringStreamReturn {
        StringStream::register_dynamic_type();
        StringStreamReturn::register_dynamic_type();

        let stream = StringStream::new();

        let ret = StringStreamReturn::new(&stream);

        loop {
            names.subscribe().block();
            let names = names.receive(128);
            if names.is_empty() {
                return ret;
            }
            let mut names = names.as_slice();
            while !names.is_empty() {
                let send = stream.send(&names);
                send.subscribe().block();
                let n = send.await_();
                assert_ne!(n, 0, "future receiver dropped");
                let n = n.try_into().expect("sent count overflows usize");
                names = &names[n..];
            }
        }
    }
}
