// SPDX-FileCopyrightText: 2024 Greenbone AG
//
// SPDX-License-Identifier: GPL-2.0-or-later

use configparser::ini::Ini;
use std::{
    io::Result,
    process::{Child, Command},
};

/// Check if it is possible to start openvas
pub fn check() -> bool {
    Command::new("openvas").spawn().is_ok()
}

/// Check if it is possible to start openvas with the sudo command
pub fn check_sudo() -> bool {
    Command::new("sudo").args(["-n", "openvas"]).spawn().is_ok()
}

pub fn read_openvas_config() -> Result<Ini> {
    let oconfig = Command::new("openvas").arg("-s").output()?;

    let mut config = Ini::new();
    let oconfig = oconfig.stdout.iter().map(|x| *x as char).collect();
    config
        .read(oconfig)
        .expect("Error reading openvas configuration");
    Ok(config)
}

/// Start a new scan with the openvas executable with the given string. Before a scan can be
/// started all data needed for the scan must put into redis before.
pub fn start(id: &str, sudo: bool, nice: Option<i8>) -> Result<Child> {
    match nice {
        Some(niceness) => match sudo {
            true => Command::new("nice")
                .args([
                    "-n",
                    &niceness.to_string(),
                    "sudo",
                    "-n",
                    "openvas",
                    "--start-scan",
                    id,
                ])
                .spawn(),
            false => Command::new("nice")
                .args(["-n", &niceness.to_string(), "openvas", "--start-scan", id])
                .spawn(),
        },
        None => match sudo {
            true => Command::new("sudo")
                .args(["-n", "openvas", "--start-scan", id])
                .spawn(),
            false => Command::new("openvas").args(["--start-scan", id]).spawn(),
        },
    }
}

/// Stops a running scan
pub fn stop(id: &str, sudo: bool) -> Result<Child> {
    match sudo {
        true => Command::new("sudo")
            .args(["-n", "openvas", "--stop-scan", id])
            .spawn(),
        false => Command::new("openvas").args(["--stop-scan", id]).spawn(),
    }
}