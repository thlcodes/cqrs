use std::{any::Any, collections::HashMap, sync::Mutex};

use thiserror::Error;

use actix::{Actor, Addr};

/// Possible registry errors
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum RegistryError {
    #[error("generic error: {0}")]
    GenericError(String),
    #[error("lock failed")]
    LockError,
    #[error("an invalid registry entry was found for id {0}")]
    InvalidRegistryEntry(String),
}

/// This registry takes actors ...
#[derive(Default)]
pub struct ActorRegistry {
    map: Mutex<HashMap<String, Box<dyn Any + Sync + Send>>>,
}

impl ActorRegistry {
    /// Get an an already registered & running actor for given id.
    /// If no (alive) actor for that id was found, a new actor is
    /// initialized with the given factory fn
    pub fn get_with_factory<A: Actor, F: FnOnce(&str) -> Addr<A>>(
        &self,
        id: &str,
        factory: F,
    ) -> Result<Addr<A>, RegistryError> {
        let mut map = self.map.lock().map_err(|_| RegistryError::LockError)?;

        // try to find an actor ref by the given id
        if let Some(addr) = map.get(&id.to_owned()) {
            let addr = addr
                .downcast_ref::<Addr<A>>()
                .cloned()
                .ok_or_else(|| RegistryError::InvalidRegistryEntry(id.into()))?;
            // return if actor is alive
            if addr.connected() {
                return Ok(addr);
            }
        }
        let addr = factory(id);
        map.insert(id.to_owned(), Box::new(addr.clone()));
        Ok(addr)
    }
}

#[cfg(test)]
mod tests {
    use actix::{ActorContext, Context, Handler, Message};

    use super::*;

    struct TestActor {
        id: String,
        i: u8,
    }

    #[derive(Message)]
    #[rtype(result = "String")]
    struct Hi;

    #[derive(Message)]
    #[rtype(result = "u8")]
    struct Count;

    #[derive(Message)]
    #[rtype(result = "()")]
    struct Stop;

    impl Actor for TestActor {
        type Context = Context<Self>;
    }

    impl Handler<Hi> for TestActor {
        type Result = String;

        fn handle(&mut self, _msg: Hi, _ctx: &mut Self::Context) -> Self::Result {
            format!("Hi from {}", self.id.as_str())
        }
    }

    impl Handler<Stop> for TestActor {
        type Result = ();

        fn handle(&mut self, _msg: Stop, ctx: &mut Self::Context) -> Self::Result {
            ctx.stop()
        }
    }

    impl Handler<Count> for TestActor {
        type Result = u8;

        fn handle(&mut self, _msg: Count, _ctx: &mut Self::Context) -> Self::Result {
            self.i += 1;
            self.i
        }
    }

    #[actix::test]
    async fn test_get_if_not_exits() {
        let reg = ActorRegistry::default();
        let id = String::from("act_123");
        let res = reg.get_with_factory(id.as_str(), |id| {
            TestActor {
                id: id.to_owned(),
                i: 0,
            }
            .start()
        });
        assert!(res.is_ok(), "get failed: {:?}", res.err());
        let addr = res.unwrap();
        let got = addr.send(Hi).await.unwrap();
        let want = format!("Hi from {}", id);
        assert_eq!(want, got, "'{}' != '{}'", want, got);
    }

    #[actix::test]
    async fn test_get_if_exists() {
        let reg = ActorRegistry::default();
        let id = String::from("act_123");

        // get actor the first time, start it since it does not exist
        let res = reg.get_with_factory(id.as_str(), |id| {
            TestActor {
                id: id.to_owned(),
                i: 0,
            }
            .start()
        });
        assert!(res.is_ok(), "first get failed: {:?}", res.err());
        let addr = res.unwrap();
        let got = addr.send(Count).await.unwrap();
        let want = 1;
        assert_eq!(want, got, "'{}' != '{}'", want, got);

        // get actor the second time, state should be persisted
        let res = reg.get_with_factory(id.as_str(), |_| -> Addr<TestActor> {
            panic!("this should not be called")
        });
        assert!(res.is_ok(), "second get failed: {:?}", res.err());
        let addr = res.unwrap();
        let got = addr.send(Count).await.unwrap();
        let want = 2;
        assert_eq!(want, got, "'{}' != '{}'", want, got);
    }

    #[actix::test]
    async fn test_get_if_exists_but_stopped() {
        let reg = ActorRegistry::default();
        let id = String::from("act_123");

        // get actor the first time, start it since it does not exist
        let res = reg.get_with_factory(id.as_str(), |id| {
            TestActor {
                id: id.to_owned(),
                i: 0,
            }
            .start()
        });
        assert!(res.is_ok(), "first get failed: {:?}", res.err());
        let addr = res.unwrap();
        let got = addr.send(Count).await.unwrap();
        let want = 1;
        assert_eq!(want, got, "'{}' != '{}'", want, got);

        // send stop msg to current actor
        addr.do_send(Stop);

        // now sends to that actor should fail
        assert!(
            addr.send(Count).await.is_err(),
            "send did not fail, but should have"
        );

        // get (stopped) actor the second time, thus creating a new instance
        let res = reg.get_with_factory(id.as_str(), |id| {
            TestActor {
                id: id.to_owned(),
                i: 0,
            }
            .start()
        });
        assert!(res.is_ok(), "second get failed: {:?}", res.err());
        let addr = res.unwrap();
        let got = addr.send(Count).await.unwrap();
        let want = 1;
        assert_eq!(want, got, "'{}' != '{}'", want, got);
    }
}
