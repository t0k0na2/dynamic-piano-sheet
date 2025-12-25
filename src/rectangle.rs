pub struct Rectangle{
    left: f64,
    top: f64,
    width: f64,
    height: f64,
}

impl Rectangle
{
    pub fn new(left: f64, top: f64, width: f64, height: f64) -> Self{
        Rectangle{
            left,
            top,
            width,
            height,
        }
    }

    pub fn left(&self) -> f64{
        self.left
    }
    pub fn top(&self) -> f64{
        self.top
    }
    pub fn width(&self) -> f64{
        self.width
    }
    pub fn height(&self) -> f64{
        self.height
    }
    pub fn right(&self) -> f64{
        self.left + self.width
    }
    pub fn bottom(&self) -> f64{
        self.top + self.height
    }
}