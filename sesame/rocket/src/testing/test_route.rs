#[macro_export]
macro_rules! test_route {
    ($method:ident, $uri:literal, $handler:ident) => {
        ::sesame_rocket::rocket::SesameRoute::from(::sesame_rocket::rocket::SesameRouteInfo {
            method: ::rocket::http::Method::$method,
            uri: $uri,
            handler: |request, data| ::std::boxed::Box::pin($handler(request, data)),
        })
    };
}
