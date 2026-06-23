//! Sandbox control route handlers.

use std::sync::atomic::Ordering;

use axum::{extract::State, response::Json};
use civ_agents::spawn_civilian_at;
use civ_tactics::DamageEvent;
use civ_voxel::{MaterialId, WorldCoord};

use crate::app::{
    AppState, ControlOk, DamageReq, MilitaryPin, PlaceVoxelReq, SpawnCivilianReq, SpawnEntityReq,
    SpeedReq,
};

pub(crate) async fn place_voxel_handler(
    State(state): State<AppState>,
    Json(req): Json<PlaceVoxelReq>,
) -> Json<ControlOk> {
    let mut sim = state.sim.lock().await;
    sim.voxel_mut().write(
        WorldCoord {
            x: req.x,
            y: req.y,
            z: req.z,
        },
        MaterialId(req.material),
    );
    Json(ControlOk {
        ok: true,
        message: None,
    })
}

pub(crate) async fn spawn_civilian_handler(
    State(state): State<AppState>,
    Json(req): Json<SpawnCivilianReq>,
) -> Json<ControlOk> {
    let mut sim = state.sim.lock().await;
    let id = sim.state.tick.wrapping_add(1) ^ 0x00c0_ffee;
    let mut rng = sim.rng_mut().clone();
    let _ = spawn_civilian_at(
        &mut sim.world,
        id,
        civ_agents::Alignment::Faction(req.faction),
        req.x,
        req.y,
        civ_agents::ActorVisualKind::Humanoid,
        &mut rng,
    );
    *sim.rng_mut() = rng;
    Json(ControlOk {
        ok: true,
        message: None,
    })
}

pub(crate) async fn spawn_entity_handler(
    State(state): State<AppState>,
    Json(req): Json<SpawnEntityReq>,
) -> Json<ControlOk> {
    let mut sim = state.sim.lock().await;
    let mut spawn_civilian_like = |kind: civ_agents::ActorVisualKind| {
        let id = sim.state.tick.wrapping_add(1) ^ 0x00c0_ffee;
        let mut rng = sim.rng_mut().clone();
        let _ = spawn_civilian_at(
            &mut sim.world,
            id,
            civ_agents::Alignment::Faction(req.faction),
            req.x,
            req.y,
            kind,
            &mut rng,
        );
        *sim.rng_mut() = rng;
    };
    match req.kind.as_str() {
        "civilian" => {
            spawn_civilian_like(civ_agents::ActorVisualKind::Humanoid);
        }
        "herd" => {
            spawn_civilian_like(civ_agents::ActorVisualKind::Herd);
        }
        "vehicle" => {
            use civ_engine::{spawn_military_at, UnitType};
            let _ = spawn_military_at(&mut sim.world, req.faction, req.x, req.y, UnitType::Knight);
            let mut military = state.military.lock().await;
            let id = sim.state.tick.wrapping_add(1) ^ 0xdeadbee_u64;
            military.push(MilitaryPin {
                id,
                x: req.x.clamp(0.0, 1.0),
                y: req.y.clamp(0.0, 1.0),
                unit_type: "Vehicle".to_string(),
                faction: req.faction,
                strength: 1.0,
            });
        }
        "airport" => {
            use civ_engine::spawn_airport_at;
            let _ = spawn_airport_at(&mut sim.world, req.x, req.y);
        }
        "port" => {
            use civ_engine::spawn_port_at;
            let _ = spawn_port_at(&mut sim.world, req.x, req.y);
        }
        "hangar" => {
            use civ_engine::spawn_hangar_at;
            let _ = spawn_hangar_at(&mut sim.world, req.x, req.y);
        }
        _ => {
            return Json(ControlOk {
                ok: false,
                message: Some(
                    "kind must be civilian, vehicle, airport, port, hangar, or herd".to_string(),
                ),
            });
        }
    }
    Json(ControlOk {
        ok: true,
        message: None,
    })
}

pub(crate) async fn damage_handler(
    State(state): State<AppState>,
    Json(req): Json<DamageReq>,
) -> Json<ControlOk> {
    let mut sim = state.sim.lock().await;
    let event = DamageEvent {
        center: WorldCoord {
            x: req.x,
            y: req.y,
            z: req.z,
        },
        radius_voxels: req.radius,
        energy: req.energy,
    };
    sim.push_damage(event);
    Json(ControlOk {
        ok: true,
        message: None,
    })
}

pub(crate) async fn speed_handler(
    State(state): State<AppState>,
    Json(req): Json<SpeedReq>,
) -> Json<ControlOk> {
    if ![0u8, 1, 2, 4, 8].contains(&req.speed) {
        return Json(ControlOk {
            ok: false,
            message: Some("speed must be 0, 1, 2, 4, or 8".into()),
        });
    }
    state.speed.store(req.speed, Ordering::Relaxed);
    Json(ControlOk {
        ok: true,
        message: None,
    })
}
