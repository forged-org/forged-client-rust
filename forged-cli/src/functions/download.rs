use std::{io::Cursor, path::Path};

use anyhow::anyhow;
use cynic::QueryBuilder;
use probe_rs::{
    flashing::{BinOptions, FlashLoader},
    Probe,
};

use crate::Error;
use probe_rs_cli_util::{
    common_options::{CargoOptions, FlashOptions},
    flash,
};

use crate::{
    queries::{Binary, Chip, Chips},
    Result,
};

pub async fn download(
    client: &mut forged::Client,
    chip: Option<String>,
    version: Option<String>,
) -> Result<()> {
    let query = client.run_query(Chips::build(())).await?;
    let chips = query.current_provisioner.project.chips;

    let chips_string =
        chips
            .iter()
            .map(|chip| chip.name.clone())
            .fold(String::new(), |acc, chip| {
                if acc.is_empty() {
                    chip.to_string()
                } else {
                    format!("{acc}, {chip}")
                }
            });

    let chip = if let Some(chip_name) = chip {
        chips
            .iter()
            .find(|chip| chip.name == chip_name)
            .ok_or_else(|| {
                anyhow!("Chip `{chip_name}` not found. Available chips: [ {chips_string} ]")
            })?
    } else {
        match chips.len() {
            0 => {
                return Err(Error::Other(anyhow!(
                    "No chips have been configured for this project. Add one to the project first."
                )))
            }
            1 => chips.first().unwrap(),
            _ => {
                return Err(Error::Other(anyhow!(
                "Multiple chips found for this project. Please specify one. Available chips: [ {chips_string} ]"
            )))
            }
        }
    };

    let binaries = &chip.binaries;
    let binary = if let Some(version) = version {
        let version = semver::Version::parse(&version)?;
        binaries
            .iter()
            .find(|bin| bin.version() == version)
            .ok_or_else(|| {
                let mut versions: Vec<semver::Version> = binaries.iter().map(|bin| bin.version()).collect();
                versions.sort();
                versions.reverse();

                anyhow!(
                    "Binary version `{version}` not found for chip `{}`. Available versions: [ {} ]",
                    chip.name,
                    versions.iter().fold(String::new(), |acc, version| {
                        if acc.is_empty() {
                            version.to_string()
                        } else {
                            format!("{acc}, {version}")
                        }
                    }
                ))
            })?
    } else {
        // Otherwise, find the newest binary.
        let Some(binary) = binaries.iter().max_by_key(|x| x.version()) else {
            return Err(Error::Other(anyhow!(
                "No binaries have been uploaded for chip {}",
                chip.name
            )));
        };
        binary
    };

    println!(
        " -> Flashing firmware v{} onto {} ({})",
        binary.version(),
        chip.name,
        chip.part_number
    );
    println!("⛅ Grabbing binaries from the server ...");

    let result = run_flash_download(client, &chip, binary).await;

    if result.is_err() {
        println!("❌ Flashing procedure failed.");
        return result;
    }

    Ok(())
}

async fn run_flash_download(
    client: &mut forged::Client,
    chip: &Chip,
    binary: &Binary,
) -> Result<()> {
    let probe = Probe::list_all()
        .get(0)
        .ok_or_else(|| anyhow!("No probe found"))?
        .open()
        .map_err(probe_rs::Error::Probe)?;
    {
        let protocol_speed = probe.speed_khz();

        log::info!("Protocol speed {} kHz", protocol_speed);
    }

    // Create a new session
    let mut session = probe.attach(&chip.part_number, probe_rs::Permissions::default())?;

    let target = session.target();

    // Create the flash loader
    let mut loader = FlashLoader::new(target.memory_map.to_vec(), target.source().clone());

    let n_parts = binary.parts.len();
    for (index, part) in binary.parts.clone().into_iter().enumerate() {
        let binary = client
            .binary_part(chip.id, binary.id, part.id, None)
            .await?;

        println!(
            "📦 Flashing part {}/{n_parts}{}",
            index + 1,
            part.analysis
                .map(|analysis| format!(" ({} bytes)", analysis.nvm_size))
                .unwrap_or_default()
        );

        match part.kind {
            crate::queries::BinaryKind::Elf => loader
                .load_elf_data(&mut Cursor::new(binary))
                .map_err(|_| anyhow!("Failed to flash."))?,
            crate::queries::BinaryKind::Bin => loader
                .load_bin_data(
                    &mut Cursor::new(binary),
                    BinOptions {
                        base_address: part.memory_offset.map(|o| o as u64),
                        skip: 0,
                    },
                )
                .map_err(|_| anyhow!("Failed to flash."))?,
            crate::queries::BinaryKind::Hex => loader
                .load_hex_data(&mut Cursor::new(binary))
                .map_err(|_| anyhow!("Failed to flash."))?,
        }
    }

    let flash_options = FlashOptions {
        disable_double_buffering: false,
        version: false,
        list_chips: false,
        list_probes: false,
        disable_progressbars: false,
        reset_halt: false,
        log: None,
        restore_unwritten: true,
        flash_layout_output_path: None,
        elf: None,
        work_dir: None,
        cargo_options: CargoOptions::default(),
        probe_options: probe_rs_cli_util::common_options::ProbeOptions {
            allow_erase_all: true,
            chip: None,
            chip_description_path: None,
            protocol: None,
            probe_selector: None,
            connect_under_reset: false,
            speed: None,
            dry_run: false,
        },
    };
    flash::run_flash_download(&mut session, Path::new(""), &flash_options, loader, false)?;
    Ok(())
}
