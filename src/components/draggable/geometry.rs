use dioxus::html::geometry::{euclid::Point2D, ClientSpace};
use web_sys::DomRect;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct RectData {
    pub position: Point2D<f64, ClientSpace>,
    pub size: Point2D<f64, ClientSpace>,
}

impl RectData {
    pub fn get_is_within_bounds<U>(&self, point: Point2D<f64, U>) -> bool {
        (point.x >= self.position.x && point.x <= self.position.x + self.size.x)
            && (point.y >= self.position.y && point.y <= self.position.y + self.size.y)
    }

    pub fn is_overlapping(&self, other: RectData) -> bool {
        self.get_is_within_bounds(other.position)
    }

    pub fn from_bounding_box(web_sys_data: DomRect) -> Self {
        let position: Point2D<f64, ClientSpace> = Point2D::new(web_sys_data.x(), web_sys_data.y());
        let size: Point2D<f64, ClientSpace> =
            Point2D::new(web_sys_data.width(), web_sys_data.height());
        Self { position, size }
    }

    pub fn percent_transition(&self, from: Self, percent: f64) -> Self {
        let offset = RectData {
            position: Self::point_subtract(self.position, from.position),
            size: Self::point_subtract(self.size, from.size),
        };

        let final_offset = offset.from_percent(percent);

        Self {
            position: Self::point_add(self.position, final_offset.position),
            size: Self::point_add(from.size, final_offset.size),
        }
    }

    pub fn from_percent(&self, percent: f64) -> Self {
        let position = Self::point_from_percent(self.position, percent);
        let size = Self::point_from_percent(self.size, percent);
        Self { position, size }
    }

    fn point_from_percent<U>(point: Point2D<f64, U>, percent: f64) -> Point2D<f64, U> {
        Point2D::new(point.x * percent, point.y * percent)
    }

    fn point_subtract<U>(point_a: Point2D<f64, U>, point_b: Point2D<f64, U>) -> Point2D<f64, U> {
        Point2D::new(point_a.x - point_b.x, point_a.y - point_b.y)
    }

    fn point_add<U>(point_a: Point2D<f64, U>, point_b: Point2D<f64, U>) -> Point2D<f64, U> {
        Point2D::new(point_a.x + point_b.x, point_a.y + point_b.y)
    }
}
