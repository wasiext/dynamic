//! Example of instantiating a wasm module which uses WASI preview1 imports
//! implemented through the async preview2 WASI implementation.

/*
You can execute this example with:
    cmake examples/
    cargo run --example wasip2-async
*/

use std::env::args;

use bindings::{
    exports::wasiext::dynamic_examples::concurrent::GuestHelloReturn,
    wasiext::dynamic::types::{AsyncReturn, DynamicFuture, DynamicStream, FutureSend, StreamSend},
};
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::*;
use wasmtime_wasi::{
    IoImpl, IoView, WasiCtx, WasiCtxBuilder, WasiView,
    bindings::io::{poll::Host, streams::Pollable},
};
use wasmtime_wasi::{WasiImpl, bindings::Command};

pub struct ComponentRunStates {
    // These two are required basically as a standard way to enable the impl of IoView and
    // WasiView.
    // impl of WasiView is required by [`wasmtime_wasi::add_to_linker_sync`]
    pub wasi_ctx: WasiCtx,
    pub resource_table: ResourceTable,
    // You can add other custom host states if needed
}

impl IoView for ComponentRunStates {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resource_table
    }
}
impl WasiView for ComponentRunStates {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
}

mod bindings {
    wasmtime::component::bindgen!({
        path: "../rust/concurrent/wit",
        async: true,
        with: {
            "wasi:io": wasmtime_wasi::bindings::io,
        }
    });
}

impl bindings::wasiext::dynamic::types::Host for ComponentRunStates {}
impl bindings::wasiext::dynamic::types::HostAsyncReturn for ComponentRunStates {
    async fn drop(
        &mut self,
        rep: wasmtime::component::Resource<AsyncReturn>,
    ) -> wasmtime::Result<()> {
        todo!()
    }
}
impl bindings::wasiext::dynamic::types::HostDynamicStream for ComponentRunStates {
    async fn drop(
        &mut self,
        rep: wasmtime::component::Resource<DynamicStream>,
    ) -> wasmtime::Result<()> {
        todo!()
    }
}
impl bindings::wasiext::dynamic::types::HostDynamicFuture for ComponentRunStates {
    async fn drop(
        &mut self,
        rep: wasmtime::component::Resource<DynamicFuture>,
    ) -> wasmtime::Result<()> {
        todo!()
    }
}
impl bindings::wasiext::dynamic::types::HostFutureSend for ComponentRunStates {
    async fn subscribe(
        &mut self,
        self_: wasmtime::component::Resource<FutureSend>,
    ) -> wasmtime::component::Resource<Pollable> {
        todo!()
    }

    async fn await_(&mut self, self_: wasmtime::component::Resource<FutureSend>) -> bool {
        todo!()
    }

    async fn drop(
        &mut self,
        rep: wasmtime::component::Resource<FutureSend>,
    ) -> wasmtime::Result<()> {
        todo!()
    }
}
impl bindings::wasiext::dynamic::types::HostStreamSend for ComponentRunStates {
    async fn subscribe(
        &mut self,
        self_: wasmtime::component::Resource<StreamSend>,
    ) -> wasmtime::component::Resource<Pollable> {
        todo!()
    }

    async fn await_(&mut self, self_: wasmtime::component::Resource<StreamSend>) -> u32 {
        todo!()
    }

    async fn drop(
        &mut self,
        rep: wasmtime::component::Resource<StreamSend>,
    ) -> wasmtime::Result<()> {
        todo!()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Construct the wasm engine with async support enabled.
    let mut config = Config::new();
    config.async_support(true);
    let engine = Engine::new(&config)?;
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_async(&mut linker)?;
    bindings::wasiext::dynamic::types::add_to_linker(&mut linker, |cx| cx)?;

    // Create a WASI context and put it in a Store; all instances in the store
    // share this context. `WasiCtxBuilder` provides a number of ways to
    // configure what the target program will have access to.
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()
        .inherit_network()
        .build();
    let state = ComponentRunStates {
        wasi_ctx: wasi,
        resource_table: ResourceTable::new(),
    };
    let mut store = Store::new(&engine, state);

    let mut args = args();
    let (_, Some(path)) = (args.next(), args.next()) else {
        panic!("invalid args");
    };

    // Instantiate our component with the imports we've created, and run it.
    let component = Component::from_file(&engine, path)?;
    let component = bindings::Component::instantiate_async(&mut store, &component, &linker).await?;
    let concurrent = component.wasiext_dynamic_examples_concurrent();
    let fut0 = concurrent.call_hello(&mut store).await?;
    let p0 = concurrent
        .hello_return()
        .call_subscribe(&mut store, fut0)
        .await?;
    let fut1 = concurrent.call_hello(&mut store).await?;
    let p1 = concurrent
        .hello_return()
        .call_subscribe(&mut store, fut1)
        .await?;
    let mut ready = IoImpl(store.data_mut()).poll(vec![p0, p1]).await?;
    while let Some(n) = ready.pop() {
        let s = match n {
            0 => {
                eprintln!("call await on future 0");
                concurrent
                    .hello_return()
                    .call_await(&mut store, fut0)
                    .await?
            }
            1 => {
                eprintln!("call await on future 1");
                concurrent
                    .hello_return()
                    .call_await(&mut store, fut1)
                    .await?
            }
            n => panic!("invalid pollable index {n}"),
        };
        eprintln!("future {n} resolved to: {s}");
    }
    Ok(())
}
