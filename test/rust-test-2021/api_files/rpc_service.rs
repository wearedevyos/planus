use core::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
};

use planus::ReadAsRoot;

struct MyGreeter;

impl Greeter for MyGreeter {
    type Error = planus::Error;

    fn say_hello(&self, request: HelloRequestRef<'_>) -> Result<HelloReply, Self::Error> {
        Ok(HelloReply {
            message: format!("Hello, {}!", request.name()?.unwrap_or("world")),
        })
    }

    fn say_hello_again(&self, request: HelloRequestRef<'_>) -> Result<HelloReply, Self::Error> {
        self.say_hello(request)
    }
}

struct MyAsyncGreeter;

impl GreeterAsync for MyAsyncGreeter {
    type Error = planus::Error;

    async fn say_hello(&self, request: HelloRequestRef<'_>) -> Result<HelloReply, Self::Error> {
        Ok(HelloReply {
            message: format!("Hello, {}!", request.name()?.unwrap_or("world")),
        })
    }

    async fn say_hello_again(
        &self,
        request: HelloRequestRef<'_>,
    ) -> Result<HelloReply, Self::Error> {
        self.say_hello(request).await
    }
}

assert_eq!(GREETER_NAME, "Greeter");
assert_eq!(GreeterMethod::SERVICE_NAME, "Greeter");
assert_eq!(<MyGreeter as Greeter>::NAME, "Greeter");
assert_eq!(<MyAsyncGreeter as GreeterAsync>::NAME, "Greeter");

assert_eq!(
    GreeterMethod::ENUM_VALUES,
    [GreeterMethod::SayHello, GreeterMethod::SayHelloAgain]
);
// Ids are the first 16 bytes of the SHA-256 of "<service>.<method>".
assert_eq!(GreeterMethod::SERVICE_ID, 0x3f798d9602eb0b0b03b86a43429c18d3_u128);
assert_eq!(GreeterMethod::SayHello.id(), 0xb707af5721448045972bb708f37df99c_u128);
assert_eq!(GreeterMethod::SayHelloAgain.id(), 0x9fe54c0127420c0cb4c9898d8c212e8b_u128);
assert_eq!(GreeterMethod::SayHello.name(), "SayHello");
assert_eq!(GreeterMethod::SayHelloAgain.name(), "SayHelloAgain");
assert_eq!(
    GreeterMethod::try_from(GreeterMethod::SayHello.id()).unwrap(),
    GreeterMethod::SayHello
);
assert_eq!(u128::from(GreeterMethod::SayHelloAgain), GreeterMethod::SayHelloAgain.id());
assert!(GreeterMethod::try_from(0u128).is_err());

assert_eq!(RpcService::ENUM_VALUES, [RpcService::Greeter]);
assert_eq!(RpcService::Greeter.name(), "Greeter");
assert_eq!(RpcService::Greeter.id(), GreeterMethod::SERVICE_ID);
assert_eq!(RpcService::try_from(0x3f798d9602eb0b0b03b86a43429c18d3_u128).unwrap(), RpcService::Greeter);

let mut builder = planus::Builder::new();
let offset = HelloRequest::create(&mut builder, "Planus");
let data = builder.finish(offset, None);

let request = HelloRequestRef::read_as_root(data).unwrap();
let reply = MyGreeter.say_hello(request).unwrap();
assert_eq!(reply.message, "Hello, Planus!");

let reply = MyGreeter.say_hello_again(request).unwrap();
assert_eq!(reply.message, "Hello, Planus!");

// The futures of `GreeterAsync` have no await points, so they complete on
// the first poll and can be driven without an executor.
let mut cx = Context::from_waker(Waker::noop());
let mut future = pin!(MyAsyncGreeter.say_hello_again(request));
let Poll::Ready(reply) = future.as_mut().poll(&mut cx) else {
    panic!("future should be ready on the first poll");
};
assert_eq!(reply.unwrap().message, "Hello, Planus!");
