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

use rumqttc::{AsyncClient, ClientError, EventLoop, QoS};
use std::collections::HashMap;

use myrulesiot::mqtt;
use myrulesiot::mqtt::{ActionMessage, ConnectionMessage, ConnectionValues};

use myrulesiot::rules::forward;
use myrulesiot::rules::lights;
use myrulesiot::rules::savelist;
use myrulesiot::rules::zigbee;

pub async fn connect_mqtt() -> Result<(AsyncClient, EventLoop), ClientError> {
    // Defines connection properties
    let connection_info = ConnectionValues {
        id: String::from("rustclient-231483"),
        host: String::from("localhost"),
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
        (String::from("SYSMR/system_action"), QoS::AtMostOnce),
        (String::from("ESPURNA04/#"), QoS::AtMostOnce),
    ];

    mqtt::new_connection(connection_info, subscriptions).await
}

type FnReducer =
    Box<dyn Fn(&mut HashMap<String, Vec<u8>>, &ActionMessage) -> Vec<ConnectionMessage> + Send>;

type ReducersVec = Vec<FnReducer>;

pub fn app_map_reducers() -> ReducersVec {
    vec![
        Box::new(savelist::save_value("SYSMR/user_action/tick")),
        Box::new(forward::forward_user_action_tick("myhelloiot/timer")),
        // Box::new(rules::light_actions("myhelloiot/light1")),
        // Box::new(rules::modal_value("myhelloiot/alarm")),
        Box::new(savelist::save_list(
            "myhelloiot/temperature",
            &chrono::Duration::seconds(20),
            40,
        )),
        // Box::new(forward::forward_action(
        //     "zigbee2mqtt/0x000b57fffe4fc5ca",
        //     "ESPURNA04/relay/0/set",
        // )),
        // Box::new(lights::toggle(
        //     zigbee::actuator_toggle("zigbee2mqtt/0x000b57fffe4fc5ca"),
        //     "ESPURNA04/relay/0",
        //     "ESPURNA04/relay/0/set",
        // )),
        Box::new(lights::light_time(
            zigbee::actuator_toggle("zigbee2mqtt/0x000b57fffe4fc5ca"),
            "ESPURNA04/relay/0",
            "ESPURNA04/relay/0/set",
        )),
        Box::new(lights::light_on(
            zigbee::actuator_brightness_up("zigbee2mqtt/0x000b57fffe4fc5ca"),
            "ESPURNA04/relay/0",
            "ESPURNA04/relay/0/set",
        )),
        Box::new(lights::light_off(
            zigbee::actuator_brightness_down("zigbee2mqtt/0x000b57fffe4fc5ca"),
            "ESPURNA04/relay/0",
            "ESPURNA04/relay/0/set",
        )),
        Box::new(lights::light_time_reset(
            "ESPURNA04/relay/0",
            "ESPURNA04/relay/0/set",
        )),
        Box::new(lights::status("ESPURNA04/relay/0")),
        // Box::new(devices::simulate_relay("ESPURNA04/relay/0")),
    ]
}
