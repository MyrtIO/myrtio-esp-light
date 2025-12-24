use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
use esp_bootloader_esp_idf::{
    ota::OtaImageState,
    ota_updater::OtaUpdater,
    partitions::PARTITION_TABLE_MAX_LEN,
};
#[cfg(feature = "log")]
use esp_println::println;

use crate::{
    core::net::http::AsyncChunkedReader,
    domain::{
        entity::BootSlot,
        ports::{
            BootSectorSelector,
            BootSectorWriter as _,
            FirmwareError,
            FirmwareHandler,
            HttpFirmwareUpdater,
        },
    },
    infrastructure::{
        repositories::BootManager,
        services::storage::{FLASH_STORAGE, state::StorageState},
    },
};

const ALIGN: usize = 4;
const ERASE_SECTOR: u32 = 4096;

#[derive(Default)]
pub struct OtaService;

impl OtaService {
    pub fn new() -> Self {
        Self {}
    }
}

impl OtaService {}

impl HttpFirmwareUpdater for OtaService {
    async fn update_firmware_from_http(
        &self,
        conn: &mut impl AsyncChunkedReader,
    ) -> Result<(), FirmwareError> {
        StorageState::wait_for_idle().await;

        let guard = FLASH_STORAGE.lock().await;
        let mut cell = guard.borrow_mut();
        let flash_ref = cell.as_mut().unwrap();

        let mut part_buffer = [0u8; PARTITION_TABLE_MAX_LEN];
        let mut updater = OtaUpdater::new(flash_ref, &mut part_buffer)
            .map_err(|_| FirmwareError::InvalidPartitionTable)?;

        let (mut partition, _part_type) = updater
            .next_partition()
            .map_err(|_| FirmwareError::InvalidPartitionTable)?;

        let content_length = conn.content_length();
        #[cfg(feature = "log")]
        println!(
            "ota: target partition {:?}, content_length={}",
            _part_type, content_length
        );

        let part_capacity = partition.capacity() as u32;
        let erase_size = content_length.saturating_add(ERASE_SECTOR - 1)
            / ERASE_SECTOR
            * ERASE_SECTOR;
        let erase_size = erase_size.min(part_capacity);
        #[cfg(feature = "log")]
        println!("ota: erasing {} bytes", erase_size);
        if partition.erase(0, erase_size).is_err() {
            return Err(FirmwareError::Erase);
        }

        let mut written: u32 = 0;
        let mut received: usize = 0;
        let mut tail = [0xFFu8; ALIGN];
        let mut tail_len: usize = 0;
        let mut first_bytes: [u8; 4] = [0; 4];
        let mut chunk_count: u32 = 0;

        let mut is_eof = false;
        while !is_eof {
            conn.read_and_then(|chunk| {
                if chunk.is_empty() {
                    is_eof = true;
                } else {
                    // Capture first 4 bytes for debugging
                    if received == 0 && chunk.len() >= 4 {
                        first_bytes.copy_from_slice(&chunk[..4]);
                    }
                    write_aligned_data(
                        &mut partition,
                        chunk,
                        &mut written,
                        &mut tail,
                        &mut tail_len,
                    )
                    .unwrap();
                    received += chunk.len();
                    chunk_count += 1;
                }
            })
            .await
            .map_err(|_| FirmwareError::Read)?;
        }

        #[cfg(feature = "log")]
        println!(
            "ota: received {} bytes in {} chunks, written {} bytes",
            received, chunk_count, written
        );
        #[cfg(feature = "log")]
        println!(
            "ota: first 4 bytes: {:02X} {:02X} {:02X} {:02X}",
            first_bytes[0], first_bytes[1], first_bytes[2], first_bytes[3]
        );

        // Write final tail
        if tail_len > 0 {
            #[cfg(feature = "log")]
            println!("ota: writing final tail of {} bytes", tail_len);
            partition
                .write(written, &tail)
                .map_err(|_| FirmwareError::Write)?;
        }

        updater
            .activate_next_partition()
            .and_then(|()| updater.set_current_ota_state(OtaImageState::New))
            .map_err(|_| FirmwareError::Activate)?;

        #[cfg(feature = "log")]
        println!("ota: update complete, activating partition");
        Ok(())
    }
}

impl OtaService {
    async fn set_boot_sector(
        &mut self,
        slot: BootSlot,
    ) -> Result<(), FirmwareError> {
        StorageState::set(StorageState::UpdatingBootSector);

        let guard = FLASH_STORAGE.lock().await;
        let mut cell = guard.borrow_mut();
        let flash_ref = cell.as_mut().unwrap();
        let mut repo = BootManager::new(flash_ref);
        repo.write_boot_sector(slot)
    }
}

impl BootSectorSelector for OtaService {
    async fn boot_system(&mut self) -> Result<(), FirmwareError> {
        self.set_boot_sector(BootSlot::System).await
    }

    async fn boot_factory(&mut self) -> Result<(), FirmwareError> {
        self.set_boot_sector(BootSlot::Factory).await
    }
}

impl FirmwareHandler for OtaService {}

#[allow(clippy::cast_possible_truncation)]
fn write_aligned_data<F: embedded_storage::nor_flash::NorFlash>(
    partition: &mut F,
    data: &[u8],
    written: &mut u32,
    tail: &mut [u8; 4],
    tail_len: &mut usize,
) -> Result<(), FirmwareError> {
    let mut idx = 0;

    // Complete partial word
    if *tail_len > 0 {
        let need = 4 - *tail_len;
        let take = need.min(data.len());
        tail[*tail_len..*tail_len + take].copy_from_slice(&data[..take]);
        *tail_len += take;
        idx += take;

        if *tail_len == 4 {
            partition
                .write(*written, tail)
                .map_err(|_| FirmwareError::Flash)?;
            *written += 4;
            *tail_len = 0;
            tail.fill(0xFF);
        }
    }

    // Write aligned bulk
    let rem = &data[idx..];
    let aligned_len = rem.len() & !3;
    if aligned_len > 0 {
        partition
            .write(*written, &rem[..aligned_len])
            .map_err(|_| FirmwareError::Flash)?;
        *written += aligned_len as u32;
    }

    // Keep trailing bytes
    let tail_bytes = &rem[aligned_len..];
    if !tail_bytes.is_empty() {
        tail[..tail_bytes.len()].copy_from_slice(tail_bytes);
        *tail_len = tail_bytes.len();
    }

    Ok(())
}
