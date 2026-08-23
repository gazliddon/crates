#[derive(Clone)]
pub struct Run<'a, A: Eq + Copy> {
    pub run: usize,
    pub item: &'a A,
}

impl<'a, A: Eq + Copy> Run<'a, A> {
    pub fn new(item: &'a A) -> Self {
        Self { run: 1, item }
    }
    pub fn inc(&mut self) {
        self.run += 1;
    }

    pub fn reset(&mut self, val: &'a A) {
        self.run = 1;
        self.item = val;
    }
}

#[derive(Default)]
pub struct Rle<'a, A: Eq + Copy> {
    run: Option<Run<'a, A>>,
    result: Vec<Run<'a, A>>,
}

impl<'a, A: Eq + Copy> Rle<'a, A> {
    pub fn new() -> Self {
        Self {
            run: None,
            result: vec![],
        }
    }

    fn flush(&mut self, val: Option<&'a A>) {
        // Is there something to be flushed?
        if let Some(run) = &mut self.run {
            self.result.push(run.clone());
        }

        self.run = val.map(|item| Run::new(item));
    }

    /// Adds one item to the current run.
    pub fn add(&mut self, val: &'a A) {
        if let Some(run) = &mut self.run {
            if run.item == val {
                run.inc();
            } else {
                self.flush(Some(val));
            }
        } else {
            self.flush(Some(val));
        }
    }

    pub fn get(&mut self) -> Vec<Run<'a, A>> {
        self.flush(None);
        let mut ret = vec![];
        std::mem::swap(&mut self.result, &mut ret);
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::Rle;

    #[test]
    fn groups_adjacent_items_and_flushes_on_get() {
        let values = [1, 1, 2, 3, 3, 3];
        let mut rle = Rle::new();
        for value in &values {
            rle.add(value);
        }

        let runs = rle.get();
        assert_eq!(
            runs.iter().map(|run| run.run).collect::<Vec<_>>(),
            [2, 1, 3]
        );
        assert_eq!(
            runs.iter().map(|run| *run.item).collect::<Vec<_>>(),
            [1, 2, 3]
        );
        assert!(rle.get().is_empty());
    }
}
