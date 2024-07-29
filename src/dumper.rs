use std::fs;

use anyhow::{bail, Result};

use chrono::Utc;

use memflow::prelude::v1::*;

use pelite::image::{IMAGE_DOS_SIGNATURE, IMAGE_NT_HEADERS_SIGNATURE};
use pelite::pe64::headers_mut;
use pelite::util::AlignTo;

pub fn dump_process(
    process: &mut IntoProcessInstanceArcBox<'_>,
    file_name: Option<String>,
) -> Result<()> {
    let module = process.primary_module()?;

    let mut image = vec![0; module.size as _];

    process.read_raw_into(module.base, &mut image).data_part()?;

    let (dos_header, nt_headers, _, section_headers) = unsafe { headers_mut(&mut image) };

    if dos_header.e_magic != IMAGE_DOS_SIGNATURE {
        bail!("invalid dos header magic");
    }

    if !dos_header.e_lfanew.aligned_to(4) {
        bail!("dos header is misaligned");
    }

    if nt_headers.Signature != IMAGE_NT_HEADERS_SIGNATURE {
        bail!("invalid nt header signature");
    }

    for section in section_headers {
        section.PointerToRawData = section.VirtualAddress;
        section.SizeOfRawData = section.VirtualSize;
    }

    let mut file_name = file_name.unwrap_or_else(|| module.name.to_string());

    if let Some(index) = file_name.rfind('.') {
        file_name = file_name[..index].to_string();
    }

    let timestamp = Utc::now().format("%Y-%m-%d-%H%M%S").to_string();

    fs::write(format!("{}_{}_dump.exe", file_name, timestamp), &image)?;

    Ok(())
}
