use super::Value;
use crate::Node;

/// Wrapper around the tape that allows interacting with it via a `Object`-like API.

pub struct Object<'tape, 'input>(pub(super) &'tape [Node<'input>]);

pub struct ObjectIter<'tape, 'input>(&'tape [Node<'input>]);
pub struct ObjectKeys<'tape, 'input>(&'tape [Node<'input>]);
pub struct ObjectValues<'tape, 'input>(&'tape [Node<'input>]);

//value_trait::Object for
impl<'tape, 'input> Object<'tape, 'input> {
    /// FIXME: docs

    #[must_use]
    pub fn get(&self, k: &str) -> Option<Value<'tape, 'input>> {
        let Node::Object { mut len, .. } = self.0.first()? else {
            return None;
        };
        let mut idx = 1;
        while len > 0 {
            let s = self.0.get(idx)?.as_str()?;
            idx += 1;
            len -= 1;
            let count = self.0.get(idx)?.count();
            if s == k {
                return Some(Value(&self.0[idx..idx + count]));
            }
            idx += count;
        }
        None
    }
    /// FIXME: docs
    #[must_use]
    pub fn iter<'i>(
        &'i self,
    ) -> Box<dyn Iterator<Item = (&'input str, Value<'tape, 'input>)> + 'i> {
        Box::new(ObjectIter(&self.0[1..]))
    }
    /// FIXME: docs
    #[must_use]
    pub fn keys<'i>(&'i self) -> Box<dyn Iterator<Item = &'input str> + 'i> {
        Box::new(ObjectKeys(&self.0[1..]))
    }
    /// FIXME: docs
    #[must_use]
    pub fn values<'i>(&'i self) -> Box<dyn Iterator<Item = Value<'tape, 'input>> + 'i> {
        Box::new(ObjectValues(&self.0[1..]))
    }
    /// FIXME: docs
    #[must_use]
    pub fn len(&self) -> usize {
        let Some(Node::Object { len, .. }) = self.0.first() else {
            unreachable!("invalid tape object");
        };
        *len
    }
    /// FIXME: docs
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'tape, 'input> Iterator for ObjectIter<'tape, 'input> {
    type Item = (&'input str, Value<'tape, 'input>);

    fn next(&mut self) -> Option<Self::Item> {
        let (k, v) = self.0.split_first()?;
        let k = k.as_str()?;
        let count = v.first()?.count();
        let (head, tail) = v.split_at(count);
        self.0 = tail;
        Some((k, Value(head)))
    }
}

impl<'tape, 'input> Iterator for ObjectKeys<'tape, 'input> {
    type Item = &'input str;
    fn next(&mut self) -> Option<Self::Item> {
        let (k, v) = self.0.split_first()?;
        let k = k.as_str()?;
        let count = v.first()?.count();
        let (_, tail) = v.split_at(count);
        self.0 = tail;
        Some(k)
    }
}

impl<'tape, 'input> Iterator for ObjectValues<'tape, 'input> {
    type Item = Value<'tape, 'input>;
    fn next(&mut self) -> Option<Self::Item> {
        let (_, v) = self.0.split_first()?;
        let count = v.first()?.count();
        let (head, tail) = v.split_at(count);
        self.0 = tail;
        Some(Value(head))
    }
}

#[cfg(test)]
mod test {
    use crate::to_tape;

    #[test]
    fn get_ints() -> crate::Result<()> {
        let mut input = br#"{"snot": 1, "badger":2, "cake":3, "cookie":4}"#.to_vec();
        let t = to_tape(input.as_mut_slice())?;
        let v = t.as_value();
        let a = v.as_object().expect("is an object");
        assert_eq!(a.get("snot").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(a.get("badger").and_then(|v| v.as_u64()), Some(2));
        assert_eq!(a.get("cake").and_then(|v| v.as_u64()), Some(3));
        assert_eq!(a.get("cookie").and_then(|v| v.as_u64()), Some(4));
        assert_eq!(a.get("monster"), None);
        Ok(())
    }

    #[test]
    fn get_complex() -> crate::Result<()> {
        let mut input =
            br#"{"snot": 1, "badger":[2, 2.5], "cake":{"frosting": 3}, "cookie":4}"#.to_vec();
        let t = to_tape(input.as_mut_slice())?;
        let v = t.as_value();
        let a = v.as_object().expect("is an object");
        assert_eq!(a.get("snot").and_then(|v| v.as_u64()), Some(1));
        let badger = a
            .get("badger")
            .and_then(|v| v.as_array())
            .expect("is an array");
        assert_eq!(badger.get(0).and_then(|v| v.as_u64()), Some(2));
        assert_eq!(badger.get(1).and_then(|v| v.as_f64()), Some(2.5));
        let cake = a
            .get("cake")
            .and_then(|v| v.as_object())
            .expect("is an object");
        assert_eq!(cake.get("frosting").and_then(|v| v.as_u64()), Some(3));
        assert_eq!(a.get("cookie").and_then(|v| v.as_u64()), Some(4));
        assert_eq!(a.get("monster"), None);
        Ok(())
    }
    #[test]
    fn iter_ints() -> crate::Result<()> {
        let mut input = br#"{"snot": 1, "badger":2, "cake":3, "cookie":4}"#.to_vec();
        let t = to_tape(input.as_mut_slice())?;
        let v = t.as_value();
        let v = v
            .as_object()
            .expect("is an object")
            .iter()
            .map(|(k, v)| (k, v.as_u64().expect("integer")))
            .collect::<Vec<_>>();
        assert_eq!(
            v,
            vec![("snot", 1), ("badger", 2), ("cake", 3), ("cookie", 4)]
        );

        Ok(())
    }

    #[test]
    fn keys_ints() -> crate::Result<()> {
        let mut input = br#"{"snot": 1, "badger":2, "cake":3, "cookie":4}"#.to_vec();
        let t = to_tape(input.as_mut_slice())?;
        let v = t.as_value();
        let v = v
            .as_object()
            .expect("is an object")
            .keys()
            .collect::<Vec<_>>();
        assert_eq!(v, vec!["snot", "badger", "cake", "cookie"]);

        Ok(())
    }

    #[test]
    fn values_ints() -> crate::Result<()> {
        let mut input = br#"{"snot": 1, "badger":2, "cake":3, "cookie":4}"#.to_vec();
        let t = to_tape(input.as_mut_slice())?;
        let v = t.as_value();
        let v = v
            .as_object()
            .expect("is an object")
            .values()
            .map(|v| v.as_u64().expect("integer"))
            .collect::<Vec<_>>();
        assert_eq!(v, vec![1, 2, 3, 4]);

        Ok(())
    }
    #[test]
    fn iter_complex() -> crate::Result<()> {
        let mut input =
            br#"{"snot": 1, "badger":[2, 2.5], "cake":{"frosting": 3}, "cookie":4}"#.to_vec();
        let t = to_tape(input.as_mut_slice())?;
        let v = t.as_value();
        let v = v
            .as_object()
            .expect("is an object")
            .iter()
            .map(|(k, v)| (k, v.as_u64()))
            .collect::<Vec<_>>();
        assert_eq!(
            v,
            vec![
                ("snot", Some(1)),
                ("badger", None),
                ("cake", None),
                ("cookie", Some(4))
            ]
        );
        Ok(())
    }
    #[test]
    fn keys_complex() -> crate::Result<()> {
        let mut input =
            br#"{"snot": 1, "badger":[2, 2.5], "cake":{"frosting": 3}, "cookie":4}"#.to_vec();
        let t = to_tape(input.as_mut_slice())?;
        let v = t.as_value();
        let v = v
            .as_object()
            .expect("is an object")
            .keys()
            .collect::<Vec<_>>();
        assert_eq!(v, vec!["snot", "badger", "cake", "cookie"]);

        Ok(())
    }

    #[test]
    fn values_complex() -> crate::Result<()> {
        let mut input =
            br#"{"snot": 1, "badger":[2, 2.5], "cake":{"frosting": 3}, "cookie":4}"#.to_vec();
        let t = to_tape(input.as_mut_slice())?;
        let v = t.as_value();
        let v = v
            .as_object()
            .expect("is an object")
            .values()
            .map(|v| v.as_u64())
            .collect::<Vec<_>>();
        assert_eq!(v, vec![Some(1), None, None, Some(4)]);

        Ok(())
    }
}