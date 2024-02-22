use crate::{pb::sf::substreams::v1::store_delta::Operation, store::Delta};

pub struct OperationIs<I: Iterator> {
    operation: Operation,
    negate: bool,
    underlying: I,
}

impl<I> OperationIs<I>
where
    I: Iterator,
    I::Item: Delta,
{
    pub(crate) fn new(operation: Operation, negate: bool, underlying: I) -> Self {
        Self {
            operation,
            negate,
            underlying,
        }
    }
}

impl<I> Iterator for OperationIs<I>
where
    I: Iterator,
    I::Item: Delta,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(x) = self.underlying.next() {
            let mut emit = x.get_operation() == self.operation;
            if self.negate {
                emit = !emit;
            }

            if emit {
                return Some(x);
            }
        }

        None
    }
}
