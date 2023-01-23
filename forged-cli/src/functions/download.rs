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
    queries::{BinaryNewest, BinaryPart},
    Result,
};

pub async fn download(client: &mut forged::Client) -> Result<()> {
    println!("⛅ Grabbing binaries from the server ...");
    let query = client.run_query(BinaryNewest::build(())).await?;
    let binary = query
        .current_provisioner
        .project
        .binary_newest
        .ok_or_else(|| anyhow::anyhow!("No binaries present"))?;
    let result = run_flash_download(query.current_provisioner.project.chip.name, binary.parts);

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
        println!(
            "📦 Flashing part {index}/{n_parts} with {} bytes to target",
            part.image.len()
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
