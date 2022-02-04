# cqrs

**A lightweight, opinionated CQRS and event sourcing framework targeting serverless architectures.**   
**Now with actors via [Actix](https://actix.rs)**

Command Query Responsibility Segregation (CQRS) is a pattern in
[Domain Driven Design](https://martinfowler.com/tags/domain%20driven%20design.html)
that uses separate write and read models for application objects and interconnects them with events.
Event sourcing uses the generated events as the source of truth for the
state of the application.

Together these provide a number of benefits:
- Removes coupling between tests and application logic allowing limitless refactoring.
- Greater isolation of the [aggregate](https://martinfowler.com/bliki/DDD_Aggregate.html).
- Ability to create views that more accurately model our business environment.
- A horizontally scalable read path.
- **Now with Actors via [Actix](https://actix.rs)!**


[![Crates.io](https://img.shields.io/crates/v/cqrs-actors)](https://crates.io/crates/cqrs-actors)
[![docs](https://img.shields.io/badge/API-docs-blue.svg)](https://docs.rs/cqrs-actors)

