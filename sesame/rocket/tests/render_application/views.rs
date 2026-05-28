use sesame::pcon::PCon;
use sesame::policy::NoPolicy;
use sesame_rocket::render::PConRender;

#[derive(PConRender)]
pub struct Point(pub PCon<i64, NoPolicy>, pub PCon<i64, NoPolicy>);

#[derive(PConRender)]
pub enum Shape {
    Unknown,
    Circle(PCon<i64, NoPolicy>),
    Rectangle(PCon<i64, NoPolicy>, PCon<i64, NoPolicy>),
    Named {
        width: PCon<i64, NoPolicy>,
        label: String,
    },
}

// Context wrappers for the templates.

#[derive(PConRender)]
pub struct PointCtx {
    pub point: Point,
}

#[derive(PConRender)]
pub struct ShapeCtx {
    pub shape: Shape,
}
