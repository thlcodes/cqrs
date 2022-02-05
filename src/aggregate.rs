use actix::{Actor, Context, Handler, Message};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::{AggregateError, DomainEvent};

/// Result alias
pub type Result<A, E> = std::result::Result<A, AggregateError<E>>;

/// In CQRS (and Domain Driven Design) an `Aggregate` is the fundamental component that
/// encapsulates the state and application logic (aka business rules) for the application.
/// An `Aggregate` is always an entity along with all objects associated with it.
///
/// # Examples
/// ```
/// # use cqrs_actors::doc::{CustomerEvent, CustomerCommand};
/// # use cqrs_actors::{Aggregate, AggregateError, UserErrorPayload, Result};
/// # use serde::{Serialize,Deserialize};
/// # use actix::{Actor, Context, Handler, Message};
/// #[derive(Serialize, Deserialize)]
/// pub struct Customer {
///     pub customer_id: String,
///     pub name: String,
///     pub email: String,
/// }
///
/// impl Aggregate for Customer {
///     type Command = CustomerCommand;
///     type Event = CustomerEvent;
///     type Error = UserErrorPayload;
///
///     fn aggregate_type() -> &'static str {
///         "customer"
///     }
/// }
///
/// impl Actor for Customer {
///     type Context = Context<Self>;
/// }
///
/// impl Handler<CustomerCommand> for Customer {
///     type Result = Result<Vec<CustomerEvent>, UserErrorPayload>;
///
///     fn handle(&mut self, msg: CustomerCommand, _ctx: &mut Self::Context) -> Self::Result {
///         match msg {
///             CustomerCommand::AddCustomerName { changed_name } => {
///                 if self.name.as_str() != "" {
///                     return Err("a name has already been added for this customer".into());
///                 }
///                 Ok(vec![CustomerEvent::NameAdded { changed_name }])
///             }
///             CustomerCommand::UpdateEmail { .. } => Ok(Default::default()),
///         }
///     }
/// }
///
/// impl Handler<CustomerEvent> for Customer {
///     type Result = ();
///
///     fn handle(&mut self, msg: CustomerEvent, _ctx: &mut Self::Context) -> Self::Result {
///         match msg {
///             CustomerEvent::NameAdded { changed_name } => {
///                 self.name = changed_name;
///             }
///             CustomerEvent::EmailUpdated { new_email } => {
///                 self.email = new_email;
///             }
///         }
///     }
/// }
///
/// impl Default for Customer {
///   fn default() -> Self {
///       Customer {
///           customer_id: "".to_string(),
///           name: "".to_string(),
///           email: "".to_string(),
///       }
///   }
/// }
/// ```
///
pub trait Aggregate:
    Default
    + Serialize
    + DeserializeOwned
    + Actor<Context = Context<Self>>
    + Handler<Self::Command>
    + Handler<Self::Event>
    + Sync
    + Send
{
    /// Specifies the inbound command used to make changes in the state of the Aggregate.
    /// This is most easily accomplished with an enum;
    type Command: Message<Result = Result<Vec<Self::Event>, Self::Error>> + Sync + Send;
    /// Specifies the published events representing some change in state of the Aggregate.
    /// This is most easily accomplished with an enum;
    type Event: DomainEvent + Message<Result = ()> + Sync + Send;
    /// The error returned when a command fails due to business logic.
    /// Usually used to provide feedback to the user as to the nature of why the command was refused.
    type Error: std::error::Error + Send + Sync;
    /// The aggregate type is used as the identifier for this aggregate and its events upon
    /// serialization. The value returned should be unique.
    fn aggregate_type() -> &'static str;
}
