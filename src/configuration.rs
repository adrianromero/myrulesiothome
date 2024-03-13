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

use myrulesiot::mqtt::EngineFunction;
use myrulesiot::rules::forward;
use myrulesiot::rules::zigbee;

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
