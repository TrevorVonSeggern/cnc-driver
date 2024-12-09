use arrayvec::ArrayVec;
use crate::util::search_pattern;

#[derive(Default)]
pub struct StateListStackAlloc<T, TId, const SIZE: usize> where T : Default {
    pub type_ids: ArrayVec<TId, SIZE>,
    pub data: ArrayVec<T, SIZE>,
}

impl<T, TId, const SIZE: usize> StateListStackAlloc<T, TId, SIZE> where T : Default {
    pub fn new() -> Self {
        return Self {
            type_ids: Default::default(),
            data: Default::default(),
        }
    }
}

pub trait StateList<T, TId> where T : Default {
    fn push(&mut self, id: TId, data: T);
    fn drain(&mut self, index: usize, count: usize);
    fn replace(&mut self, index: usize, count: usize, replacement_id: TId, replacement: &mut T);
    fn get_type_ids_slice(&self) -> &[TId];
    fn get_data_slice_mut(&mut self) -> &mut [T];
    fn set_type_id(&mut self, id: TId, index: usize);
}

impl<T, TId, const SIZE: usize> StateList<T, TId> for StateListStackAlloc<T, TId, SIZE> where T : Default {
    fn push(&mut self, id: TId, data: T) {
        self.type_ids.push(id);
        self.data.push(data);
    }

    fn drain(&mut self, index: usize, count: usize) {
        if count != 0 {
            self.type_ids.drain(index..(index + count));
            self.data.drain(index..(index + count));
        }
    }

    fn replace(&mut self, index: usize, count: usize, replacement_id: TId, replacement: &mut T) {
        self.drain(index+1, count-1);
        self.type_ids[index] = replacement_id;
        self.data[index] = core::mem::take(replacement);
    }

    fn get_type_ids_slice(&self) -> &[TId] {
        self.type_ids.as_slice()
    }

    fn get_data_slice_mut(&mut self) -> &mut [T] {
        self.data.as_mut_slice()
    }

    fn set_type_id(&mut self, id: TId, index: usize) {
        self.type_ids[index] = id;
    }
}

pub struct Rule<'f, T, TId> {
    pub id: TId,
    pub pattern: &'f [TId],
    pub func: &'f dyn Fn(&mut [T]) -> Option<T>,
}

pub struct ParserStackAlloc<'f, T, TId, const SIZE: usize> {
    pub rules: [Rule<'f, T, TId>; SIZE],
}

impl<'f, T, TId, const SIZE: usize> ParserStackAlloc<'f, T, TId, SIZE>
    where
        T: Clone, T: Default,
        TId : PartialEq, TId : Clone
{
    pub fn parse(&self, state: &mut impl StateList<T, TId>)
    {
        let mut update_counter = 0;
        let mut prev_counter = -1;
        for _ in 0..90 {
            for rule in self.rules.iter() {
                let i = search_pattern(state.get_type_ids_slice(), rule.pattern);
                if i != -1 {
                    let u = i as usize;
                    let mut replace = (rule.func)(&mut state.get_data_slice_mut()[u..(u + rule.pattern.len())]);
                    if let Some(replace) = replace.as_mut() {
                        state.replace(u, rule.pattern.len(), rule.id.clone(), replace); 
                    }
                    else {
                        state.drain(u + 1, rule.pattern.len() - 1);
                        state.set_type_id(rule.id.clone(), u);
                    }
                    update_counter += 1;
                    break;
                }
            }
            if update_counter == prev_counter {
                break;
            }
            prev_counter = update_counter;
        }
    }

    //pub fn add_relabel_rule(&mut self, from: TId, to: TId) {
        //self.rules.push(Rule::<T, TId>{id:to, pattern: vec![from], func: Box::new(move |_| {
            //return None;
        //})});
    //}
}

