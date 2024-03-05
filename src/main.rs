//    MyRulesIoT  Project is a rules engine for MQTT based on MyRulesIoT lib
//    Copyright (C) 2022-2024my  Adrián Romero Corchado.
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

use std::error::Error;

use tokio::sync::mpsc;
use tokio::try_join;

use myrulesiot::mqtt::{self, EngineAction, EngineResult, EngineState};
use myrulesiot::runtime;

mod configuration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    log::info!("Starting myrulesiot...");
    // Load state from disk

    let (client, eventloop) = configuration::connect_mqtt().await?;

    let (sub_tx, sub_rx) = mpsc::channel::<EngineAction>(10);
    let (pub_tx, pub_rx) = mpsc::channel::<EngineResult>(10);

    //let timertask = mqtt::task_timer_loop(&sub_tx, &chrono::Duration::milliseconds(250));
    let mqttsubscribetask = mqtt::task_subscription_loop(&sub_tx, eventloop);
    let mqttpublishtask = mqtt::task_publication_loop(pub_rx, client); // or pub_tx.subscribe() if broadcast

    let enginetask = runtime::task_runtime_loop(
        &pub_tx,
        sub_rx,
        mqtt::MasterEngine::new(
            String::from("HOMERULES"),
            configuration::app_engine_functions(),
        ),
        EngineState::default(),
    );

    // Push function
    sub_tx.send(EngineAction {
        topic: "HOMERULES/command/functions_push".into(),
        payload: b"{\"name\":\"ikea_actuator\", \"parameters\": {\"topic\":\"zigbee2mqtt/Tradfri Remote\",\"command\":\"toggle\"}}".into(),
        timestamp: 0,
    })
    .await.unwrap();

    // Push function
    sub_tx.send(EngineAction {
        topic: "HOMERULES/command/functions_push".into(),
        payload: b"{\"name\":\"shelly_relay\", \"parameters\": {\"topic\":\"shellies/shellyswitch01/relay/1/command\"}}".into(),
        timestamp: 0,
    })
    .await.unwrap();

    // Push function
    sub_tx.send(EngineAction {
        topic: "HOMERULES/command/functions_push".into(),
        payload: b"{\"name\":\"ikea_actuator\", \"parameters\": {\"topic\":\"zigbee2mqtt/Tradfri Remote\",\"command\":\"brightness_down_click\"}}".into(),
        timestamp: 0,
    })
    .await.unwrap();

    // Push function
    sub_tx.send(EngineAction {
        topic: "HOMERULES/command/functions_push".into(),
        payload: b"{\"name\":\"shelly_relay\", \"parameters\": {\"topic\":\"HOMERULES/command/exit\"}}".into(),
        timestamp: 0,
    })
    .await.unwrap();

    std::mem::drop(sub_tx);
    std::mem::drop(pub_tx);

    let _ = try_join!(enginetask, mqttpublishtask, mqttsubscribetask)?;

    // Save state to disk

    log::info!("Exiting myrulesiot...");
    Ok(())
}
