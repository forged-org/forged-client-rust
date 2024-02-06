use std::{io::Cursor, path::Path};

use anyhow::anyhow;
use cynic::QueryBuilder;
use probe_rs::{
    flashing::{BinOptions, FlashLoader},
    Probe,
};
use probe_rs_cli_util::{
    common_options::{CargoOptions, FlashOptions},
    flash,
};

use crate::{
    queries::{Chips, BinaryPart},
    Result,
};

pub async fn download(client: &mut forged::Client, chip: Option<String>, version: Option<String>) -> Result<()> {
    println!("⛅ Grabbing binaries from the server ...");

    let query = client.run_query(Chips::build(())).await?;
    let chips = query.current_provisioner.project.chips;

    let chip = if let Some(chip_name) = option.chip {
        chips.find(|chip| chip.name == chip_name).ok_or_else(|| anyhow!("Chip {chip_name} not found. Available chips: {}", chips.iter().map(|chip| chip.name)))?
    } else {
        match chips.len() {
            0 => return Err(anyhow!("No chips have been configured for this project. Add one to the project first.")),
            n => return Err(anyhow!("Multiple chips found for this project. Please specify one. Available chips: {}", chips.iter().map(|chip| chip.name))),
            1 => chips.iter().first().unwrap(),
        }
    };

    let binaries = chip.binaries;
    let binary = if let Some(version) = option.version {
        binaries.find(|bin| bin.version == version).ok_or_else(|| anyhow!("Binary version {version} not found for chip {}. Available versions: {}", chip.name, binaries.iter().map(|bin| bin.version)))?
    } else {
        match binaries.len() {
            0 => return Err(anyhow!("No binaries have been configured for chip {}. Add a binary to this chip first.", chip.name)),
            n => return Err(anyhow!("Multiple binaries found for chip {}. Please specify one. Available versions: {}", chip.name, binaries.iter().map(|bin| bin.version))),
            1 => binaries.iter().first().unwrap(),
        }
    };

    let result = run_flash_download(chip.name, binary.parts);

    if result.is_err() {
        println!("❌ Flashing procedure failed.");
        return result;
    }

    Ok(())
}

fn run_flash_download(chip: impl AsRef<str>, parts: Vec<BinaryPart>) -> Result<()> {
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
    let mut session = probe.attach(chip.as_ref(), probe_rs::Permissions::default())?;

    let target = session.target();

    // Create the flash loader
    let mut loader = FlashLoader::new(target.memory_map.to_vec(), target.source().clone());

    let n_parts = parts.len();
    for (index, part) in parts.into_iter().enumerate() {

        // TODO: Download the actual binary from the server
        let binary = client.binary_part(binary_id, part_id)?;

        println!(
            "📦 Flashing part {index}/{n_parts} with {} bytes to target",
            part.analysis.nvm_size
        );

        match part.kind {
            crate::queries::BinaryKind::Elf => loader
                .load_elf_data(&mut Cursor::new(
                    part.image.into_iter().map(|v| v as u8).collect::<Vec<_>>(),
                ))
                .map_err(|_| anyhow!("Failed to flash."))?,
            crate::queries::BinaryKind::Bin => loader
                .load_bin_data(
                    &mut Cursor::new(part.image.into_iter().map(|v| v as u8).collect::<Vec<_>>()),
                    BinOptions {
                        base_address: part.memory_offset.map(|o| o as u64),
                        skip: 0,
                    },
                )
                .map_err(|_| anyhow!("Failed to flash."))?,
            crate::queries::BinaryKind::Hex => loader
                .load_hex_data(&mut Cursor::new(
                    part.image.into_iter().map(|v| v as u8).collect::<Vec<_>>(),
                ))
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
