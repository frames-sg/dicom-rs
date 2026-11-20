# DICOM-rs `core`

[![CratesIO](https://img.shields.io/crates/v/dicom-core.svg)](https://crates.io/crates/dicom-core)
[![Documentation](https://docs.rs/dicom-core/badge.svg)](https://docs.rs/dicom-core)

This sub-project implements the essential data structures and mechanisms
for dealing with DICOM information and communication formats,
thus serving as a center piece for other crates in [DICOM-rs].

This crate is part of the [DICOM-rs] project
and is contained by the parent crate [`dicom`](https://crates.io/crates/dicom).

[DICOM-rs]: https://github.com/Enet4/dicom-rs


## Frames 0.9.1 calendar API

This fork uses Jiff for Gregorian dates and the system clock. Concrete dates,
times and fixed offsets returned by DICOM value and range methods live in
`dicom_core::calendar`. Code that used the upstream Chrono re-export must
update its imports and conversions. A fixed offset attaches to a local value
with `offset.at(local_datetime)`. Offset-aware values compare by instant.
The DICOM types preserve partial precision, six fractional digits and leap
seconds, including when a range crosses midnight.
