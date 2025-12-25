#[derive(Clone, Copy, Debug)]
pub struct Bar{
    begin_time: f64,
    end_time: f64,
    number: u32,
}

impl Bar{
    pub fn new(begin_time: f64, end_time: f64, number: u32) -> Self{
        Bar{
            begin_time,
            end_time,
            number,
        }
    }
    pub fn begin_time(&self) -> f64{
        self.begin_time
    }
    pub fn end_time(&self) -> f64{
        self.end_time
    }
    pub fn set_end_time(&mut self, end_time: f64){
        self.end_time = end_time;
    }
    pub fn number(&self) -> u32{
        self.number
    }
}