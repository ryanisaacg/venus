use glam::{Mat3, Vec2};

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn position(&self) -> Vec2 {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }

    pub fn size(&self) -> Vec2 {
        Vec2 {
            x: self.width,
            y: self.height,
        }
    }
}

pub struct IRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub fn orthographic_projection(x: f32, y: f32, width: f32, height: f32) -> Mat3 {
    Mat3::from_scale(Vec2::new(2.0, -2.0))
        * Mat3::from_translation(Vec2::new(-0.5, -0.5))
        * Mat3::from_scale(Vec2::new(1.0 / width, 1.0 / height))
        * Mat3::from_translation(Vec2::new(-x, -y))
}

#[cfg(test)]
mod test {
    use approx::assert_abs_diff_eq;
    use glam::Vec2;

    use super::orthographic_projection;

    #[test]
    fn basic_orthographic() {
        let projection = orthographic_projection(20.0, 20.0, 100.0, 100.0);
        let tests = [
            // center
            (Vec2::new(70.0, 70.0), Vec2::new(0.0, 0.0)),
            // top left
            (Vec2::new(20.0, 20.0), Vec2::new(-1.0, 1.0)),
            // bottom right
            (Vec2::new(120.0, 120.0), Vec2::new(1.0, -1.0)),
        ];
        for (from, to) in tests {
            let from = projection.transform_point2(from);
            assert_abs_diff_eq!(from.x, to.x);
            assert_abs_diff_eq!(from.y, to.y);
        }
    }
}
