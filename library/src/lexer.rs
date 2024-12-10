use core::marker::PhantomData;

pub struct LexResult<T> {
    pub poped_chars: usize,
    pub result: T,
}

pub struct LexerStackAlloc<'f, T, const RULE_SIZE: usize> {
    pub rules: [&'f dyn Fn(&str) -> Option<LexResult<T>>; RULE_SIZE],
}

pub trait LexerTrait<'f, T> {
    fn get_rules(&self) -> &[&'f dyn Fn(&str) -> Option<LexResult<T>>];
    fn match_rules(&self, source: &str) -> Option<LexResult<T>>;
    fn iter(&self, source: &'f str) -> LexerIter<'_, 'f, Self, T> where Self : Sized;
}

impl<'f, T, const RULE_SIZE: usize> LexerTrait<'f, T> for LexerStackAlloc<'f, T, RULE_SIZE> {
    fn get_rules(&self) -> &[&'f dyn Fn(&str) -> Option<LexResult<T>>] {
        &self.rules
    }

    fn match_rules(&self, source: & str) -> Option<LexResult<T>> {
        for &rule in self.get_rules().iter() {
            let possible_match = (*rule)(source);
            if possible_match.is_some() {
                return possible_match;
            }
        }
        return None;
    }

    fn iter(&self, source: &'f str) -> LexerIter<'_, 'f, Self, T> {
        return LexerIter { lexer: self, source, position: 0, id_phantom: PhantomData::default() };
    }
}

pub struct LexerIter<'lexer, 'fstr, TLex, T> {
    pub lexer: &'lexer TLex,
    pub source: &'fstr str,
    pub position: usize,
    id_phantom: PhantomData<T>,
}

impl<'lexer, 'fstr, TLex, TId> Iterator for LexerIter<'lexer, 'fstr, TLex, TId>  where TLex : LexerTrait<'fstr, TId> {
    type Item = LexResult<TId>;
    fn next(&mut self) -> Option<Self::Item> {
        let (_, remainder) = self.source.split_at(self.position);
        debug_assert!(self.position <= self.source.len());
        return match self.lexer.match_rules(remainder) {
            None => None,
            Some(found) => {
                self.position += found.poped_chars;
                Some(found)
            },
        };
    }
}

#[cfg(test)]
mod tests {
    use arrayvec::ArrayVec;
    use super::*;

    #[test]
    fn lexer_construction_test() {
        let lexer = LexerStackAlloc::<i32, 1> {
            rules: [ &|_| None ],
        };
        assert_eq!(lexer.rules.len(), 1);
    }

    #[test]
    fn lexer_run_rule_test() {
        let lexer = LexerStackAlloc::<char, 4> {
            rules: [
                &|a| if a.starts_with("a") { Some(LexResult{ poped_chars: 1, result: 'a' }) } else { None },
                &|b| if b.starts_with("b") { Some(LexResult{ poped_chars: 1, result: 'b' }) } else { None },
                &|c| if c.starts_with("c") { Some(LexResult{ poped_chars: 1, result: 'c' }) } else { None },
                &|d| if d.starts_with("d") { Some(LexResult{ poped_chars: 1, result: 'd' }) } else { None },
            ],
        };
        let output: ArrayVec<_, 4> = lexer.iter("abcd").collect();
        assert_eq!(output.len(), 4);
        assert_eq!(output[0].result, 'a');
        assert_eq!(output[1].result, 'b');
        assert_eq!(output[2].result, 'c');
        assert_eq!(output[3].result, 'd');
    }
}
