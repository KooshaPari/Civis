//! `civ-watch` — local hot-reload sandbox harness for Civis 3D.
//!
//! Background `Simulation` ticks at ~10 Hz; SSE snapshots at `GET /events`;
//! latest snapshot at `GET /snapshot`; procedural heightmap at `GET /terrain`;
//! sandbox controls under `POST /control/*` (place_voxel, spawn_civilian,
//! damage, speed). Dashboard static build under `web/dashboard/dist` is
//! served at `GET /`.

mod app;
mod control_routes;
mod mods_api;
mod saves_api;
mod server;
mod sim_worker;
mod snapshot;
mod sse;
pub mod terrain;

#[cfg(test)]
mod api_tests;

pub use server::run;
