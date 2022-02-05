use actix::{Actor, Context, Handler, Message};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use cqrs_actors::mem_store::MemStore;
use cqrs_actors::test::TestFramework;
use cqrs_actors::{
    ActorRegistry, Aggregate, AggregateError, CqrsFramework, DomainEvent, EventEnvelope, EventStore,
};
use cqrs_actors::{Query, Result, UserErrorPayload};

#[derive(Debug, Serialize, Deserialize)]
pub struct TestAggregate {
    id: String,
    description: String,
    tests: Vec<String>,
}

impl Aggregate for TestAggregate {
    type Command = TestCommand;
    type Event = TestEvent;
    type Error = UserErrorPayload;

    fn aggregate_type() -> &'static str {
        "TestAggregates"
    }
}

impl Actor for TestAggregate {
    type Context = Context<Self>;
}

impl Handler<TestCommand> for TestAggregate {
    type Result = Result<Vec<TestEvent>, <Self as Aggregate>::Error>;

    fn handle(&mut self, msg: TestCommand, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            TestCommand::CreateTest(command) => {
                let event = TestEvent::Created(Created { id: command.id });
                Ok(vec![event])
            }

            TestCommand::ConfirmTest(command) => {
                for test in &self.tests {
                    if test == &command.test_name {
                        return Err("test already performed".into());
                    }
                }
                let event = TestEvent::Tested(Tested {
                    test_name: command.test_name,
                });
                Ok(vec![event])
            }

            TestCommand::DoSomethingElse(command) => {
                let event = TestEvent::SomethingElse(SomethingElse {
                    description: command.description,
                });
                Ok(vec![event])
            }
        }
    }
}

impl Handler<TestEvent> for TestAggregate {
    type Result = ();

    fn handle(&mut self, msg: TestEvent, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            TestEvent::Created(e) => {
                self.id = e.id;
            }
            TestEvent::Tested(e) => {
                self.tests.push(e.test_name);
            }
            TestEvent::SomethingElse(e) => {
                self.description = e.description;
            }
        }
    }
}

impl Default for TestAggregate {
    fn default() -> Self {
        TestAggregate {
            id: "".to_string(),
            description: "".to_string(),
            tests: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Message)]
#[rtype(result = "()")]
pub enum TestEvent {
    Created(Created),
    Tested(Tested),
    SomethingElse(SomethingElse),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Created {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Tested {
    pub test_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SomethingElse {
    pub description: String,
}

impl DomainEvent for TestEvent {
    fn event_type(&self) -> &'static str {
        match self {
            TestEvent::Created(_) => "Created",
            TestEvent::Tested(_) => "Tested",
            TestEvent::SomethingElse(_) => "SomethingElse",
        }
    }

    fn event_version(&self) -> &'static str {
        "1.0"
    }
}

#[derive(Message)]
#[rtype(result = "Result<Vec<TestEvent>, UserErrorPayload>")]
pub enum TestCommand {
    CreateTest(CreateTest),
    ConfirmTest(ConfirmTest),
    DoSomethingElse(DoSomethingElse),
}

pub struct CreateTest {
    pub id: String,
}

pub struct ConfirmTest {
    pub test_name: String,
}

pub struct DoSomethingElse {
    pub description: String,
}

struct TestView {
    events: Arc<RwLock<Vec<EventEnvelope<TestAggregate>>>>,
}

impl TestView {
    fn new(events: Arc<RwLock<Vec<EventEnvelope<TestAggregate>>>>) -> Self {
        TestView { events }
    }
}
#[async_trait]
impl Query<TestAggregate> for TestView {
    async fn dispatch(&self, _aggregate_id: &str, events: &[EventEnvelope<TestAggregate>]) {
        for event in events {
            let mut event_list = self.events.write().unwrap();
            event_list.push(event.clone());
        }
    }
}

pub type TestEventEnvelope = EventEnvelope<TestAggregate>;

fn metadata() -> HashMap<String, String> {
    let now = "2021-03-18T12:32:45.930Z".to_string();
    let mut metadata = HashMap::new();
    metadata.insert("time".to_string(), now);
    metadata
}

#[actix::test]
async fn test_mem_store() {
    let registry = Arc::new(ActorRegistry::default());
    let event_store = MemStore::<TestAggregate>::new_with_registry(registry);
    let id = "test_id_A";
    let initial_events = event_store.load(id).await;
    assert_eq!(0, initial_events.len());
    let agg_context = event_store
        .load_aggregate(id)
        .await
        .expect("aggregate load failed");

    event_store
        .commit(
            vec![TestEvent::Created(Created {
                id: "test_event_A".to_string(),
            })],
            agg_context,
            metadata(),
        )
        .await
        .unwrap();
    let stored_events = event_store.load(id).await;
    assert_eq!(1, stored_events.len());
    let agg_context = event_store
        .load_aggregate(id)
        .await
        .expect("aggregate load failed");

    event_store
        .commit(
            vec![
                TestEvent::Tested(Tested {
                    test_name: "test A".to_string(),
                }),
                TestEvent::Tested(Tested {
                    test_name: "test B".to_string(),
                }),
                TestEvent::SomethingElse(SomethingElse {
                    description: "something else happening here".to_string(),
                }),
            ],
            agg_context,
            metadata(),
        )
        .await
        .unwrap();
    let stored_envelopes = event_store.load(id).await;

    let agg = TestAggregate::start_default();
    for stored_envelope in stored_envelopes {
        let event = stored_envelope.payload;
        agg.do_send(event);
    }
    println!("{:#?}", agg);
}

type ThisTestFramework = TestFramework<TestAggregate>;

#[actix::test]
async fn test_framework_test() {
    let test_name = "test A";
    let test_framework = ThisTestFramework::default();

    test_framework
        .given(vec![TestEvent::Created(Created {
            id: "test_id_A".to_string(),
        })])
        .when(TestCommand::ConfirmTest(ConfirmTest {
            test_name: test_name.to_string(),
        }))
        .await
        .then_expect_events(vec![TestEvent::Tested(Tested {
            test_name: test_name.to_string(),
        })]);

    test_framework
        .given(vec![TestEvent::Tested(Tested {
            test_name: test_name.to_string(),
        })])
        .when(TestCommand::ConfirmTest(ConfirmTest {
            test_name: test_name.to_string(),
        }))
        .await
        .then_expect_error("test already performed")
}

#[actix::test]
#[should_panic]
async fn test_framework_failure_test() {
    let test_name = "test A";
    let test_framework = ThisTestFramework::default();

    test_framework
        .given(vec![TestEvent::Tested(Tested {
            test_name: test_name.to_string(),
        })])
        .when(TestCommand::ConfirmTest(ConfirmTest {
            test_name: test_name.to_string(),
        }))
        .await
        .then_expect_events(vec![TestEvent::Tested(Tested {
            test_name: test_name.to_string(),
        })]);
}

#[actix::test]
#[should_panic]
async fn test_framework_failure_test_b() {
    let test_name = "test A";
    let test_framework = ThisTestFramework::default();

    test_framework
        .given(vec![TestEvent::Created(Created {
            id: "test_id_A".to_string(),
        })])
        .when(TestCommand::ConfirmTest(ConfirmTest {
            test_name: test_name.to_string(),
        }))
        .await
        .then_expect_error("some error message")
}

#[actix::test]
async fn framework_test() {
    let registry = Arc::new(ActorRegistry::default());
    let event_store = MemStore::new_with_registry(registry);
    let stored_events = event_store.get_events();
    let delivered_events = Default::default();
    let view = TestView::new(Arc::clone(&delivered_events));

    let cqrs = CqrsFramework::new(event_store, vec![Arc::new(view)]);
    let uuid = uuid::Uuid::new_v4().to_string();
    let id = uuid.clone();
    let metadata = metadata();
    cqrs.execute_with_metadata(
        &id,
        TestCommand::ConfirmTest(ConfirmTest {
            test_name: uuid.clone(),
        }),
        metadata,
    )
    .await
    .unwrap_or_default();

    assert_eq!(1, stored_events.read().unwrap().len());
    assert_eq!(1, delivered_events.read().unwrap().len());

    let test = "TEST_A";
    let id = uuid.clone();
    cqrs.execute(
        &id,
        TestCommand::ConfirmTest(ConfirmTest {
            test_name: test.to_string(),
        }),
    )
    .await
    .unwrap_or_default();

    assert_eq!(2, delivered_events.read().unwrap().len());
    let stored_event_count = stored_events
        .read()
        .unwrap()
        .get(uuid.clone().as_str())
        .unwrap()
        .len();
    assert_eq!(2, stored_event_count);

    let id = uuid.clone();
    let err = cqrs
        .execute(
            &id,
            TestCommand::ConfirmTest(ConfirmTest {
                test_name: test.to_string(),
            }),
        )
        .await
        .unwrap_err();
    match err {
        AggregateError::UserError(payload) => {
            assert_eq!("test already performed", payload.message.unwrap().as_str())
        }
        _ => panic!("not the expected error"),
    };

    assert_eq!(2, delivered_events.read().unwrap().len());
    let stored_event_count = stored_events
        .read()
        .unwrap()
        .get(uuid.clone().as_str())
        .unwrap()
        .len();
    assert_eq!(2, stored_event_count);
}
