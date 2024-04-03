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

use config::Config;
use std::error::Error;
use std::fs;
use std::path::Path;

use tokio::sync::mpsc;
use tokio::{task, try_join};

use myrulesiot::master::{
    self, EngineAction, EngineResult, EngineState, MasterEngine, ReducerFunction,
};
use myrulesiot::mqtt::{self, ConnectionValues, Subscription};
use myrulesiot::rules;
use myrulesiot::runtime;

const FUNCTIONS_PATH: &str = "./engine_functions.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // Settings
    let settings = Config::builder()
        .add_source(config::File::with_name("./homerules"))
        .add_source(config::Environment::with_prefix("HOMERULES"))
        .build()?;

    let prefix_id = settings
        .get_string("application.identifier")
        .unwrap_or_else(|_| String::from("HOMERULES"));

    // Functions
    let functions = if let Ok(true) = Path::new(FUNCTIONS_PATH).try_exists() {
        let f = fs::read(FUNCTIONS_PATH).map_err(|error| {
            format!(
                "Cannot read ReducerFunctions file {}: {}",
                FUNCTIONS_PATH, error
            )
        })?;
        serde_json::from_slice::<Vec<ReducerFunction>>(&f).map_err(|error| {
            format!(
                "Cannot parse JSON ReducerFunctions file {}: {}",
                FUNCTIONS_PATH, error
            )
        })?;
        f
    } else {
        Vec::from(b"[]")
    };

    let (sub_tx, sub_rx) = mpsc::channel::<EngineAction>(10);
    let (pub_tx, pub_rx) = mpsc::channel::<EngineResult>(10);
    let mut multi_pub_rx = runtime::MultiRX::new(pub_rx);

    // MQTT Connection
    let connection_info: ConnectionValues = settings.get::<ConnectionValues>("mqtt.connection")?;
    let mut subscriptions: Vec<Subscription> = settings
        .get::<Vec<Subscription>>("mqtt.subscriptions")
        .unwrap_or(vec![]);
    subscriptions.push(Subscription {
        topic: format!("{}/command/#", prefix_id),
        qos: 0,
    });

    log::info!("Connecting to MQTT broker: {:?}", &connection_info);
    let (client, eventloop) = mqtt::new_connection(connection_info, subscriptions)
        .await
        .map_err(|error| format!("MQTT error: {}", error))?;

    // MQTT
    let mqttsubscribetask = mqtt::task_subscription_loop(sub_tx.clone(), eventloop);
    let mqttpublishtask = mqtt::task_publication_loop(multi_pub_rx.create(), client);

    // Senders of EngineAction's
    let timertask = master::task_timer_loop(sub_tx.clone(), chrono::Duration::milliseconds(250));
    let load_functions_task = master::task_load_functions_loop(sub_tx.clone(), functions);

    // Receivers of EngineResult's
    let save_functions_task = master::task_save_functions_loop(multi_pub_rx.create());

    let multitask = multi_pub_rx.task_publication_loop();

    // THE RUNTIME ENGINE
    let enginetask = runtime::task_runtime_loop(
        pub_tx.clone(),
        sub_rx,
        MasterEngine::new(prefix_id, rules::default_engine_functions()),
        EngineState::default(),
    );

    std::mem::drop(sub_tx);
    std::mem::drop(pub_tx);

    log::info!("Starting myrulesiot...");
    let (_, _, _, _, _, save_functions_result, _) = try_join!(
        task::spawn(enginetask),
        task::spawn(timertask),
        task::spawn(load_functions_task),
        task::spawn(mqttsubscribetask),
        task::spawn(mqttpublishtask),
        task::spawn(save_functions_task),
        task::spawn(multitask)
    )?;
    log::info!("Exiting myrulesiot...");

    if let Some(functions) = save_functions_result {
        let payload: serde_json::Value = serde_json::from_slice(&functions)?;
        let file = fs::File::create(FUNCTIONS_PATH)?;
        serde_json::to_writer_pretty(&file, &payload)?;
    }

    Ok(())
}
