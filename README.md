# cqrs

**A lightweight, opinionated CQRS and event sourcing framework targeting serverless architectures.**

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


Things that could be helpful:
- [User guide](https://doc.rust-cqrs.org) along with an introduction to CQRS and event sourcing.
- [Demo application](https://github.com/serverlesstechnology/cqrs-demo) using the warp http server.
- [Change log](https://github.com/serverlesstechnology/cqrs/blob/master/change_log.md)


[![Crates.io](https://img.shields.io/crates/v/cqrs-es)](https://crates.io/crates/cqrs-es)
[![docs](https://img.shields.io/badge/API-docs-blue.svg)](https://docs.rs/cqrs-es)

