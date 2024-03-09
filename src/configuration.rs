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

use config::Config;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

use myrulesiot::rules::forward;
use myrulesiot::rules::zigbee;
use rumqttc::{AsyncClient, ClientError, EventLoop};

use myrulesiot::mqtt;
use myrulesiot::mqtt::ConnectionValues;
use myrulesiot::mqtt::EngineFunction;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Subscription {
    topic: String,
    qos: i32,
}

pub async fn connect_mqtt(settings: &Config) -> Result<(AsyncClient, EventLoop), ClientError> {
    // Defines connection properties
    let connection_info = ConnectionValues {
        id: settings
            .get_string("mqtt.connection.client_id")
            .unwrap_or_else(|_| String::from("")),
        host: settings
            .get_string("mqtt.connection.host")
            .unwrap_or_else(|_| String::from("localhost")),
        port: settings.get_int("mqtt.connection.port").unwrap_or(1883) as u16,
        keep_alive: settings.get_int("mqtt.connection.keep_alive").unwrap_or(5) as u16,
        inflight: settings.get_int("mqtt.connection.inflight").unwrap_or(10) as u16,
        clean_session: settings
            .get_bool("mqtt.connection.clean_session")
            .unwrap_or(false),
        cap: settings.get_int("mqtt.connection.cap").unwrap_or(10) as usize,
    };

    let s: Vec<Subscription> = settings
        .get::<Vec<Subscription>>("mqtt.subscriptions")
        .unwrap_or(vec![]);

    let mut subscriptions: Vec<(String, i32)> = s.into_iter().map(|s| (s.topic, s.qos)).collect();

    let identifier = settings
        .get_string("application.identifier")
        .unwrap_or_else(|_| String::from("HOMERULES"));

    subscriptions.push((format!("{}/command/#", identifier), 0));

    mqtt::new_connection(connection_info, subscriptions).await
}

pub fn app_engine_functions() -> HashMap<String, EngineFunction> {
    HashMap::from([
        (
            String::from("ikea_actuator"),
            zigbee::engine_ikea_actuator as EngineFunction,
        ),
        (
            String::from("shelly_relay"),
            zigbee::engine_shelly_relay as EngineFunction,
        ),
        (
            String::from("forward_action"),
            forward::engine_forward_action as EngineFunction,
        ),
        (
            String::from("forward_user_action"),
            forward::engine_forward_user_action as EngineFunction,
        ),
    ])
}
