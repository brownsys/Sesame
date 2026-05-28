use sesame::pcon::PCon;
use sesame::policy::NoPolicy;
use sesame_rocket::rocket::{PConData, PConRequest, PConResponseOutcome, PConTemplate};

use crate::render_application::context::RenderContext;
use crate::render_application::views::{Point, PointCtx, Shape, ShapeCtx};

pub async fn render_tuple_struct<'a, 'r>(
    request: PConRequest<'a, 'r>,
    _data: PConData<'a>,
) -> PConResponseOutcome<'a> {
    let context: RenderContext = request.guard().await.unwrap();
    let model = PointCtx {
        point: Point(PCon::new(5i64, NoPolicy {}), PCon::new(10i64, NoPolicy {})),
    };
    PConResponseOutcome::from(request, PConTemplate::render("tuple_struct", &model, context))
}

pub async fn render_unit<'a, 'r>(
    request: PConRequest<'a, 'r>,
    _data: PConData<'a>,
) -> PConResponseOutcome<'a> {
    let context: RenderContext = request.guard().await.unwrap();
    let model = ShapeCtx {
        shape: Shape::Unknown,
    };
    PConResponseOutcome::from(request, PConTemplate::render("unit", &model, context))
}

pub async fn render_newtype<'a, 'r>(
    request: PConRequest<'a, 'r>,
    _data: PConData<'a>,
) -> PConResponseOutcome<'a> {
    let context: RenderContext = request.guard().await.unwrap();
    let model = ShapeCtx {
        shape: Shape::Circle(PCon::new(7i64, NoPolicy {})),
    };
    PConResponseOutcome::from(request, PConTemplate::render("newtype", &model, context))
}

pub async fn render_tuple<'a, 'r>(
    request: PConRequest<'a, 'r>,
    _data: PConData<'a>,
) -> PConResponseOutcome<'a> {
    let context: RenderContext = request.guard().await.unwrap();
    let model = ShapeCtx {
        shape: Shape::Rectangle(PCon::new(3i64, NoPolicy {}), PCon::new(4i64, NoPolicy {})),
    };
    PConResponseOutcome::from(request, PConTemplate::render("tuple", &model, context))
}

pub async fn render_struct_variant<'a, 'r>(
    request: PConRequest<'a, 'r>,
    _data: PConData<'a>,
) -> PConResponseOutcome<'a> {
    let context: RenderContext = request.guard().await.unwrap();
    let model = ShapeCtx {
        shape: Shape::Named {
            width: PCon::new(6i64, NoPolicy {}),
            label: String::from("hello"),
        },
    };
    PConResponseOutcome::from(
        request,
        PConTemplate::render("struct_variant", &model, context),
    )
}
