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

pub trait SolutionCollector<T>: Default {
    type Mapped<S>: SolutionCollector<S>;

    fn collect_solution(&mut self, t: T);

    fn is_full(&self) -> bool;

    fn collect_mapped<S>(&mut self, mapped: Self::Mapped<S>, f: impl Fn(S) -> T);

    //fn map_solution<S>(self, f: impl Fn(T)-> S)-> Self::Mapped<S>;
}

impl<T> SolutionCollector<T> for Option<T> {
    fn collect_solution(&mut self, t: T) {
        *self = Some(t);
    }

    fn is_full(&self) -> bool {
        self.is_some()
    }

    type Mapped<S> = Option<S>;

    fn collect_mapped<S>(&mut self, mapped: Self::Mapped<S>, f: impl Fn(S) -> T) {
        if self.is_none() {
            *self = mapped.map(f)
        }
    }
}

impl<T> SolutionCollector<T> for Vec<T> {
    fn collect_solution(&mut self, t: T) {
        self.push(t);
    }

    fn is_full(&self) -> bool {
        false
    }

    type Mapped<S> = Vec<S>;

    fn collect_mapped<S>(&mut self, mapped: Self::Mapped<S>, f: impl Fn(S) -> T) {
        self.extend(mapped.into_iter().map(f))
    }
}
