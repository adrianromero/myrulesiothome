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

use tokio::sync::mpsc;
use tokio::try_join;

use myrulesiot::mqtt::{self, EngineAction, EngineResult, EngineState};
use myrulesiot::runtime;

mod configuration;

fn read_engine_state_file() -> Result<EngineState, Box<dyn Error>> {
    let file = fs::File::open("./engine_state.json")?;
    let engine_state = serde_json::from_reader(&file)?;
    Ok(engine_state)
}

fn write_engine_state_file(state: &EngineState) -> Result<(), Box<dyn Error>> {
    let file = fs::File::create("engine_state.json")?;
    serde_json::to_writer_pretty(&file, state)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let settings = Config::builder()
        .add_source(config::File::with_name("./homerules"))
        .add_source(config::Environment::with_prefix("HOMERULES"))
        .build()?;

    let init_state = read_engine_state_file().unwrap_or_else(|_err| {
        log::warn!("Cannot load initial state from file. Loading default.");
        EngineState::default()
    });

    log::info!("Starting myrulesiot...");
    // Load state from disk

    let (client, eventloop) = configuration::connect_mqtt(&settings).await?;

    let (sub_tx, sub_rx) = mpsc::channel::<EngineAction>(10);
    let (pub_tx, pub_rx) = mpsc::channel::<EngineResult>(10);

    let timertask = mqtt::task_timer_loop(&sub_tx, &chrono::Duration::milliseconds(250));
    let mqttsubscribetask = mqtt::task_subscription_loop(&sub_tx, eventloop);
    let mqttpublishtask = mqtt::task_publication_loop(pub_rx, client); // or pub_tx.subscribe() if broadcast

    let enginetask = runtime::task_runtime_loop(
        &pub_tx,
        sub_rx,
        mqtt::MasterEngine::new(
            settings
                .get_string("application.identifier")
                .unwrap_or_else(|_| String::from("HOMERULES")),
            configuration::app_engine_functions(),
        ),
        init_state,
    );

    std::mem::drop(sub_tx);
    std::mem::drop(pub_tx);

    let (final_state, _, _, _) =
        try_join!(enginetask, mqttpublishtask, mqttsubscribetask, timertask)?;

    // Open a file for writing
    write_engine_state_file(&final_state).unwrap_or_else(|_err| {
        log::warn!("Cannot write final state to file.");
    });

    log::info!("Exiting myrulesiot...");
    Ok(())
}
