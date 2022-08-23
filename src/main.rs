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
use std::error::Error;

use tokio::sync::mpsc;
use tokio::try_join;

use myrulesiot::engine;
use myrulesiot::mqtt::{
    self, ActionMessage, ConnectionMessage, ConnectionReducer, ConnectionResult, ConnectionState,
};
use myrulesiot::timer;

mod configuration;

fn app_final(_: &HashMap<String, Vec<u8>>, action: &ActionMessage) -> bool {
    action.matches_action("SYSMR/system_action", "exit".into())
}

fn app_reducer() -> ConnectionReducer {
    let reducers = configuration::app_map_reducers();
    Box::new(move |state: ConnectionState, action: ActionMessage| {
        let mut messages = Vec::<ConnectionMessage>::new();
        let mut newmap = state.info.clone();

        for f in &reducers {
            messages.append(&mut f(&mut newmap, &action));
        }

        let is_final = app_final(&state.info, &action);

        ConnectionState {
            info: newmap,
            messages,
            is_final,
        }
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    log::info!("Starting myrulesiot...");
    let (client, eventloop) = configuration::connect_mqtt().await?;

    let (sub_tx, sub_rx) = mpsc::channel::<ActionMessage>(10);
    let (pub_tx, pub_rx) = mpsc::channel::<ConnectionResult>(10);

    let timertask = timer::task_timer_loop(&sub_tx, &chrono::Duration::milliseconds(250));
    let mqttsubscribetask = mqtt::task_subscription_loop(&sub_tx, eventloop);
    let mqttpublishtask = mqtt::task_publication_loop(pub_rx, client); // or pub_tx.subscribe() if broadcast

    let enginetask = engine::task_runtime_loop(pub_tx, sub_rx, mqtt::create_engine(app_reducer()));

    let _ = try_join!(enginetask, mqttpublishtask, mqttsubscribetask, timertask)?;

    log::info!("Exiting myrulesiot...");
    Ok(())
}
