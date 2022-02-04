use std::marker::PhantomData;

use crate::aggregate::Aggregate;
use crate::AggregateError;

/// A framework for rigorously testing the aggregate logic, one of the ***most important***
/// parts of any DDD system.
///
/// ```
/// # use cqrs_actors::doc::MyAggregate;
/// use cqrs_actors::test::TestFramework;
///
/// let framework = TestFramework::<MyAggregate>::default();
/// ```
pub struct TestFramework<A> {
    _phantom: PhantomData<A>,
}

impl<A> TestFramework<A>
where
    A: Aggregate,
{
    /// Initiates an aggregate test with no previous events.
    ///
    /// ```
    /// # use cqrs_actors::doc::MyAggregate;
    /// use cqrs_actors::test::TestFramework;
    ///
    /// let executor = TestFramework::<MyAggregate>::default()
    ///     .given_no_previous_events();
    /// ```
    #[must_use]
    pub fn given_no_previous_events(&self) -> AggregateTestExecutor<A> {
        AggregateTestExecutor { events: Vec::new() }
    }
    /// Initiates an aggregate test with a collection of previous events.
    ///
    /// ```
    /// # use cqrs_actors::doc::{MyAggregate, MyEvents};
    /// use cqrs_actors::test::TestFramework;
    ///
    /// let executor = TestFramework::<MyAggregate>::default()
    ///     .given(vec![MyEvents::SomethingWasDone]);
    /// ```
    #[must_use]
    pub fn given(&self, events: Vec<A::Event>) -> AggregateTestExecutor<A> {
        AggregateTestExecutor { events }
    }
}

impl<A> Default for TestFramework<A>
where
    A: Aggregate,
{
    fn default() -> Self {
        TestFramework {
            _phantom: PhantomData,
        }
    }
}

/// Holds the initial event state of an aggregate and accepts a command.
pub struct AggregateTestExecutor<A>
where
    A: Aggregate,
{
    events: Vec<A::Event>,
}

impl<A> AggregateTestExecutor<A>
where
    A: Aggregate,
{
    /// Consumes a command and using the state details previously passed provides a validator object
    /// to test against.
    ///
    /// ```
    /// # use cqrs_actors::doc::{MyAggregate, MyCommands};
    /// use cqrs_actors::test::TestFramework;
    ///
    /// let executor = TestFramework::<MyAggregate>::default().given_no_previous_events();
    ///
    /// let validator = executor.when(MyCommands::DoSomething);
    /// ```
    pub fn when(self, command: A::Command) -> AggregateResultValidator<A> {
        let mut aggregate = A::default();
        for event in self.events {
            aggregate.apply(event);
        }
        let result = aggregate.handle(command);
        AggregateResultValidator { result }
    }
}

/// Validation object for the `TestFramework` package.
pub struct AggregateResultValidator<A>
where
    A: Aggregate,
{
    result: Result<Vec<A::Event>, AggregateError<A::Error>>,
}

impl<A: Aggregate> AggregateResultValidator<A> {
    /// Verifies that the expected events have been produced by the command.
    ///
    /// ```
    /// # use cqrs_actors::doc::{MyAggregate, MyCommands, MyEvents};
    /// use cqrs_actors::test::TestFramework;
    ///
    /// let validator = TestFramework::<MyAggregate>::default()
    ///     .given_no_previous_events()
    ///     .when(MyCommands::DoSomething);
    ///
    /// validator.then_expect_events(vec![MyEvents::SomethingWasDone]);
    /// ```
    pub fn then_expect_events(self, expected_events: Vec<A::Event>) {
        let events = match self.result {
            Ok(expected_events) => expected_events,
            Err(err) => {
                panic!("expected success, received aggregate error: '{}'", err);
            }
        };
        assert_eq!(&events[..], &expected_events[..]);
    }
    /// Verifies that an `AggregateError` with the expected message is produced with the command.
    ///
    /// ```
    /// # use cqrs_actors::doc::{MyAggregate, MyCommands, MyEvents};
    /// use cqrs_actors::test::TestFramework;
    ///
    /// let validator = TestFramework::<MyAggregate>::default()
    ///     .given_no_previous_events()
    ///     .when(MyCommands::BadCommand);
    ///
    /// validator.then_expect_error("the expected error message");
    /// ```
    pub fn then_expect_error(self, error_message: &str) {
        match self.result {
            Ok(events) => {
                panic!("expected error, received events: '{:?}'", events);
            }
            Err(err) => match err {
                AggregateError::UserError(err) => {
                    assert_eq!(err.to_string(), error_message.to_string());
                }
                _ => {
                    panic!("expected user error but found technical error: {}", err)
                }
            },
        };
    }
}

#[cfg(test)]
mod test_framework_tests {}
