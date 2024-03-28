#[macro_export]
macro_rules! test_route {
    ($method:ident, $uri:literal, $handler:ident) => {
        ::alohomora::rocket::BBoxRoute::from(::alohomora::rocket::BBoxRouteInfo {
            method: Method::$method,
            uri: $uri,
            bbox_handler: |request, data| {
                ::std::boxed::Box::pin($handler(request, data))
            },
        })
    };
}