use anyhow::Result;
use object::{
    pe,
    read::pe::{PeFile32, PeFile64, ResourceDirectoryEntryData, ResourceNameOrId},
    FileKind, LittleEndian,
};

pub fn get_archive(data: &[u8]) -> Result<(bool, &[u8])> {
    let (sections, data_dir) = match FileKind::parse(data)? {
        FileKind::Pe64 => {
            let file = PeFile64::parse(data)?;
            anyhow::Ok((file.section_table(), file.data_directories()))
        }
        FileKind::Pe32 => {
            let file = PeFile32::parse(data)?;
            anyhow::Ok((file.section_table(), file.data_directories()))
        }
        _ => return Err(anyhow::anyhow!("Not a PE file")),
    }?;

    let res_dir = data_dir
        .resource_directory(data, &sections)?
        .ok_or(anyhow::anyhow!("No resource directory"))?;

    let table = res_dir.root()?;
    let rc_data = table
        .entries
        .iter()
        .find_map(|entry| match entry.name_or_id() {
            ResourceNameOrId::Id(id) if id == pe::RT_RCDATA => match entry.data(res_dir) {
                Ok(data) => match data {
                    ResourceDirectoryEntryData::Table(table) => Some(table),
                    _ => None,
                },
                Err(_) => None,
            },
            _ => None,
        })
        .ok_or(anyhow::anyhow!("No RCData resource"))?;

    let has_decompressor = rc_data
        .entries
        .iter()
        .any(|entry| match entry.name_or_id() {
            ResourceNameOrId::Name(name) => {
                name.to_string_lossy(res_dir).unwrap_or_default() == "DECOMPRESSOR"
            }
            _ => false,
        });

    let archive_directory = rc_data
        .entries
        .iter()
        .find_map(|entry| match entry.name_or_id() {
            ResourceNameOrId::Name(name)
                if name.to_string_lossy(res_dir).unwrap_or_default() == "ARCHIVE" =>
            {
                match entry.data(res_dir) {
                    Ok(data) => match data {
                        ResourceDirectoryEntryData::Data(data) => Some(data),
                        ResourceDirectoryEntryData::Table(table) => {
                            let first_entry = table.entries.first()?;
                            match first_entry.data(res_dir) {
                                Ok(data) => match data {
                                    ResourceDirectoryEntryData::Data(data) => Some(data),
                                    _ => None,
                                },
                                Err(_) => None,
                            }
                        }
                    },
                    Err(_) => None,
                }
            }
            _ => None,
        })
        .ok_or(anyhow::anyhow!("No ARCHIVE resource"))?;

    let archive_data = sections
        .pe_data_at(data, archive_directory.offset_to_data.get(LittleEndian))
        .ok_or_else(|| anyhow::anyhow!("Invalid archive data"))?
        .get(..archive_directory.size.get(LittleEndian) as usize)
        .ok_or_else(|| anyhow::anyhow!("Invalid archive size"))?;

    Ok((!has_decompressor, archive_data))
}
