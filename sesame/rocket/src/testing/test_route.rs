#[macro_export]
macro_rules! test_route {
    ($method:ident, $uri:literal, $handler:ident) => {
        ::sesame_rocket::rocket::BBoxRoute::from(::sesame_rocket::rocket::BBoxRouteInfo {
            method: ::rocket::http::Method::$method,
            uri: $uri,
            bbox_handler: |request, data| ::std::boxed::Box::pin($handler(request, data)),
        })
    };
}
