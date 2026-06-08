use crate::events::BridgeEvent;
use crate::types::{CityRef, PlayerId, Plot, TeamId, UnitRef};
use serde_json::json;

#[test]
fn decodes_known_event_payload() {
    let event = BridgeEvent::from_name_args(
        "city_acquired".to_string(),
        json!({
            "old_player": 1,
            "player": 2,
            "city": 9,
            "conquest": 1,
            "trade": 0,
            "x": 10,
            "y": 11
        }),
    )
    .unwrap();

    assert_eq!(
        event,
        BridgeEvent::CityAcquired {
            old_player: PlayerId(1),
            city: CityRef { player: 2, id: 9 },
            conquest: true,
            trade: false,
            plot: Plot::new(10, 11),
        }
    );
}

#[test]
fn decodes_new_reporter_event_payloads() {
    let event = BridgeEvent::from_name_args(
        "combat_result".to_string(),
        json!({
            "winner_player": 0,
            "winner_unit": 12,
            "winner_unit_type": 3,
            "winner_x": 10,
            "winner_y": 11,
            "loser_player": 1,
            "loser_unit": 9,
            "loser_unit_type": 4,
            "loser_x": 10,
            "loser_y": 11
        }),
    )
    .unwrap();

    assert_eq!(
        event,
        BridgeEvent::CombatResult {
            winner: UnitRef::new(0, 12),
            winner_unit_type: 3,
            winner_plot: Plot::new(10, 11),
            loser: UnitRef::new(1, 9),
            loser_unit_type: 4,
            loser_plot: Plot::new(10, 11),
        }
    );

    let event = BridgeEvent::from_name_args(
        "vassal_state".to_string(),
        json!({ "master": 0, "vassal": 1, "is_vassal": 1 }),
    )
    .unwrap();

    assert_eq!(
        event,
        BridgeEvent::VassalState {
            master: TeamId(0),
            vassal: TeamId(1),
            is_vassal: true,
        }
    );
}

#[test]
fn decodes_input_callback_payloads() {
    let kbd = BridgeEvent::from_name_args(
        "kbd_event".to_string(),
        json!({
            "evt": 6,
            "key": 65,
            "cursor_x": 100,
            "cursor_y": 120,
            "x": 10,
            "y": 11
        }),
    )
    .unwrap();

    assert_eq!(
        kbd,
        BridgeEvent::KbdEvent {
            evt: 6,
            key: 65,
            cursor_x: 100,
            cursor_y: 120,
            plot: Some(Plot::new(10, 11)),
        }
    );

    let mouse = BridgeEvent::from_name_args(
        "mouse_event".to_string(),
        json!({
            "evt": 1,
            "cursor_x": 70,
            "cursor_y": 80,
            "x": -1,
            "y": -1,
            "interface_consumed": 1
        }),
    )
    .unwrap();

    assert_eq!(
        mouse,
        BridgeEvent::MouseEvent {
            evt: 1,
            cursor_x: 70,
            cursor_y: 80,
            plot: None,
            interface_consumed: true,
        }
    );
}

#[test]
fn preserves_unknown_event_payload() {
    let event =
        BridgeEvent::from_name_args("future_event".to_string(), json!({ "payload": 1 })).unwrap();

    assert_eq!(
        event,
        BridgeEvent::Unknown {
            name: "future_event".to_string(),
            args: json!({ "payload": 1 }),
        }
    );
    assert_eq!(event.name(), "future_event");
}

#[test]
fn returns_protocol_name_for_known_event() {
    let event = BridgeEvent::BeginPlayerTurn {
        turn: 7,
        player: PlayerId(0),
    };

    assert_eq!(event.name(), "begin_player_turn");
}
