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

    pub fn add(&mut self, val: &'a A) -> Option<Run<'a, A>> {
        if let Some(run) = &mut self.run {
            if run.item == val {
                run.inc();
            } else {
                self.flush(Some(val));
            }
        } else {
            self.flush(Some(val));
        }
        None
    }

    pub fn get(&mut self) -> Vec<Run<'a, A>> {
        self.flush(None);
        let mut ret = vec![];
        std::mem::swap(&mut self.result, &mut ret);
        ret
    }
}

// struct RleIt<I : Iterator> where
//     A::Item : Eq + Copy
// {
//     iter : ,
//     last_item : Option<A::Item>,
//     count : usize,
//     done : bool,
// }

// impl <A:Iterator + Copy> RleIt<A> where
// A::Item : Eq + Copy
// {
//     pub fn new(iter : A) -> Self {
//         Self {
//             last_item : None,
//             iter,
//             count : 0,
//             done: false,
//         }
//     }

// }

// impl<A :Iterator + Copy> Iterator for  RleIt<A> where
// A::Item : Eq + Copy
// {
//     type Item = (usize, A::Item);

//     fn next(&mut self) -> Option<Self::Item> {

//         // return if we're done

//         if self.done {
//             return None
//         }

//         // If we're not done and we don't have a current item
//         // then get one

//         let next_item = self.iter.next();

//         if next_item.is_none() {
//             return None
//         }

//         let mut count = 1;

//         let mut i = self.iter.skip_while(|i| {
//             let is_same = next_item.unwrap() == *i;

//             if is_same {
//                 count = count + 1
//             }
//             is_same
//         });

//         self.iter = i;

//         Some((count,next_item.unwrap()))

//     }
// }
