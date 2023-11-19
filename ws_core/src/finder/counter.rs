pub trait Counter {
    fn try_increment(&mut self) -> bool;
}

pub struct RealCounter {
    pub max: usize,
    pub current: usize,
}

impl Counter for RealCounter {
    fn try_increment(&mut self) -> bool {
        if self.current >= self.max {
            return false;
        }
        self.current += 1;
        true
    }
}

pub struct FakeCounter;

impl Counter for FakeCounter {
    fn try_increment(&mut self) -> bool {
        true
    }
}
