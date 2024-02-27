//    MyRulesIoT  Project is a rules engine for MQTT based on MyRulesIoT lib
//    Copyright (C) 2022 Adrián Romero Corchado.
//
//    This file is part of MyRulesIoT.
//
//    MyRulesIoT is free software: you can redistribute it and/or modify
//    it under the terms of the GNU General Public License as published by
//    the Free Software Foundation, either version 3 of the License, or
//    (at your option) any later version.
//
//    MyRulesIoT is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//    GNU General Public License for more details.
//
//    You should have received a copy of the GNU General Public License
//    along with MyRulesIoT.  If not, see <http://www.gnu.org/licenses/>.
//

use std::collections::HashMap;

use rumqttc::{AsyncClient, ClientError, EventLoop, QoS};

use myrulesiot::mqtt;
use myrulesiot::mqtt::ConnectionValues;
use myrulesiot::mqtt::EngineFunction;

pub async fn connect_mqtt() -> Result<(AsyncClient, EventLoop), ClientError> {
    // Defines connection properties
    let connection_info = ConnectionValues {
        id: String::from("rustclient-231483"),
        host: String::from("adrian-elitedesk.local"),
        clean_session: true,
        ..Default::default()
    };
    let subscriptions = vec![
        (String::from("myhelloiot/#"), QoS::AtMostOnce),
        (
            String::from("zigbee2mqtt/0x000b57fffe323b4d"),
            QoS::AtMostOnce,
        ), // presence sensor
        (
            String::from("zigbee2mqtt/0x000b57fffe4fc5ca"),
            QoS::AtMostOnce,
        ), // remote control
        (String::from("SYSMR/system_action/#"), QoS::AtMostOnce),
    ];

    mqtt::new_connection(connection_info, subscriptions).await
}

pub fn app_engine_functions() -> HashMap<String, EngineFunction> {
    HashMap::from([
        (
            String::from("forward_action"),
            myrulesiot::rules::forward::engine_forward_action as EngineFunction,
        ),
        (
            String::from("forward_user_action"),
            myrulesiot::rules::forward::engine_forward_user_action as EngineFunction,
        ),
    ])
}

// pub fn app_map_reducers() -> Vec<mqtt::FnMQTTReducer> {
//     // 0x000b57fffe22a715 Bulb salon
//     // 0x000b57fffe4b66f7 Bulb main room
//     // 0x000b57fffe4fc5ca Remote
//     // 0x000b57fffe323b4d Light sensor
//     vec![
//         Box::new(savelist::save_value("SYSMR/user_action/tick")),
//         Box::new(forward::box_forward_user_action_tick("myhelloiot/timer")),
//         Box::new(zigbee::light_toggle(
//             zigbee::actuator_toggle("zigbee2mqtt/0x000b57fffe4fc5ca"),
//             "zigbee2mqtt/0x000b57fffe22a715",
//         )),
//         Box::new(zigbee::light_toggle(
//             zigbee::actuator_toggle("zigbee2mqtt/0x000b57fffe4fc5ca"),
//             "zigbee2mqtt/0x000b57fffe4b66f7",
//         )),
//         // Box::new(rules::light_actions("myhelloiot/light1")),
//         // Box::new(rules::modal_value("myhelloiot/alarm")),
//         // Box::new(savelist::save_list(
//         //     "myhelloiot/temperature",
//         //     &chrono::Duration::seconds(20),
//         //     40,
//         // )),
//         // Box::new(forward::forward_action(
//         //     "zigbee2mqtt/0x000b57fffe4fc5ca",
//         //     "ESPURNA04/relay/0/set",
//         // )),
//         // Box::new(lights::toggle(
//         //     zigbee::actuator_toggle("zigbee2mqtt/0x000b57fffe4fc5ca"),
//         //     "ESPURNA04/relay/0",
//         //     "ESPURNA04/relay/0/set",
//         // )),
//         // Box::new(lights::light_time(
//         //     zigbee::actuator_toggle("zigbee2mqtt/0x000b57fffe4fc5ca"),
//         //     "ESPURNA04/relay/0",
//         //     "ESPURNA04/relay/0/set",
//         // )),
//         // Box::new(lights::light_on(
//         //     zigbee::actuator_brightness_up("zigbee2mqtt/0x000b57fffe4fc5ca"),
//         //     "ESPURNA04/relay/0",
//         //     "ESPURNA04/relay/0/set",
//         // )),
//         // Box::new(lights::light_off(
//         //     zigbee::actuator_brightness_down("zigbee2mqtt/0x000b57fffe4fc5ca"),
//         //     "ESPURNA04/relay/0",
//         //     "ESPURNA04/relay/0/set",
//         // )),
//         // Box::new(lights::light_time_reset(
//         //     "ESPURNA04/relay/0",
//         //     "ESPURNA04/relay/0/set",
//         // )),
//         // Box::new(lights::status("ESPURNA04/relay/0")),
//         // Box::new(devices::simulate_relay("ESPURNA04/relay/0")),
//     ]
// }
