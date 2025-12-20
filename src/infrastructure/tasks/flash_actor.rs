use core::sync::atomic::{AtomicBool, Ordering};

use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use embedded_storage::nor_flash::{NorFlash, NorFlashError, ReadNorFlash};
use esp_bootloader_esp_idf::ota::OtaImageState;
use esp_bootloader_esp_idf::ota_updater::OtaUpdater;
use esp_bootloader_esp_idf::partitions::PARTITION_TABLE_MAX_LEN;
use esp_hal::peripherals::FLASH;
use esp_println::println;
use esp_storage::FlashStorage;
use heapless::Vec;
use static_cell::StaticCell;

use crate::domain::entity::LightState;
use crate::domain::ports::PersistentLightStateHandler;
use crate::infrastructure::config;
use crate::infrastructure::repositories::{LightNorFlashStorage, LightStorageDriver};
use crate::infrastructure::services::persistence::LightStateReceiver;

pub(crate) static OTA_ACTIVE: AtomicBool = AtomicBool::new(false);

pub(crate) const OTA_CHUNK_SIZE: usize = 1024;
const OTA_CH_CAP: usize = 4;

#[allow(clippy::large_enum_variant)]
pub enum OtaMsg {
    Begin { expected_size: u32 },
    Data { bytes: Vec<u8, OTA_CHUNK_SIZE> },
    Finish,
    Abort,
}

pub type OtaSender = Sender<'static, CriticalSectionRawMutex, OtaMsg, OTA_CH_CAP>;
type OtaReceiver = Receiver<'static, CriticalSectionRawMutex, OtaMsg, OTA_CH_CAP>;

static OTA_CHANNEL: Channel<CriticalSectionRawMutex, OtaMsg, OTA_CH_CAP> = Channel::new();

pub fn get_ota_sender() -> OtaSender {
    OTA_CHANNEL.sender()
}

static INIT_STATE_SIGNAL: Signal<CriticalSectionRawMutex, Option<LightState>> = Signal::new();

pub async fn wait_initial_state() -> Option<LightState> {
    INIT_STATE_SIGNAL.wait().await
}

static FLASH_STORAGE_CELL: StaticCell<FlashStorage<'static>> = StaticCell::new();

const PERSISTENCE_DELAY: Duration = Duration::from_millis(config::STORAGE.write_debounce_ms);

#[embassy_executor::task]
pub async fn flash_actor_task(flash: FLASH<'static>, persistence_rx: LightStateReceiver) {
    println!("flash_actor: starting");

    let flash = FLASH_STORAGE_CELL.init(FlashStorage::new(flash)) as *mut FlashStorage<'static>;
    let driver = LightStorageDriver::new(flash);
    let mut storage = LightNorFlashStorage::new(driver);

    let initial_state = storage.get_persistent_light_state().await;
    INIT_STATE_SIGNAL.signal(initial_state);

    let ota_rx = OTA_CHANNEL.receiver();

    let mut pending_state: Option<LightState> = None;

    loop {
        if OTA_ACTIVE.load(Ordering::Relaxed) {
            // OTA_ACTIVE is only set by this task, but if something goes wrong, keep
            // draining until we see a Begin to restart a session.
            if let OtaMsg::Begin { expected_size } = ota_rx.receive().await {
                if let Err(()) = ota_session(flash, &ota_rx, expected_size).await {
                    println!("flash_actor: ota session failed");
                }
            }
        } else {
            match pending_state {
                None => {
                    match select(persistence_rx.receive(), ota_rx.receive()).await {
                        Either::First(state) => pending_state = Some(state),
                        Either::Second(msg) => {
                            if let OtaMsg::Begin { expected_size } = msg {
                                pending_state = None;
                                if let Err(()) =
                                    ota_session(flash, &ota_rx, expected_size).await
                                {
                                    println!("flash_actor: ota session failed");
                                }
                            }
                        }
                    }
                }
                Some(_) => {
                    let receive_fut = persistence_rx.receive();
                    let timer_fut = Timer::after(PERSISTENCE_DELAY);
                    let ota_fut = ota_rx.receive();

                    match select(select(receive_fut, timer_fut), ota_fut).await {
                        Either::First(Either::First(state)) => pending_state = Some(state),
                        Either::First(Either::Second(())) => {
                            if let Some(state) = pending_state.take() {
                                if storage.save_persistent_light_state(state).await.is_err() {
                                    println!("flash_actor: failed to save light state");
                                }
                            }
                        }
                        Either::Second(msg) => {
                            if let OtaMsg::Begin { expected_size } = msg {
                                pending_state = None;
                                if let Err(()) =
                                    ota_session(flash, &ota_rx, expected_size).await
                                {
                                    println!("flash_actor: ota session failed");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn ota_session(
    flash: *mut FlashStorage<'static>,
    ota_rx: &OtaReceiver,
    expected_size: u32,
) -> Result<(), ()> {
    println!("flash_actor: ota begin (expected {} bytes)", expected_size);
    OTA_ACTIVE.store(true, Ordering::Relaxed);

    // Safety: flash is owned by this task (single owner).
    let flash = unsafe { &mut *flash };

    let mut buffer = [0u8; PARTITION_TABLE_MAX_LEN];
    let Ok(mut updater) = OtaUpdater::new(&mut *flash, &mut buffer) else {
        println!("flash_actor: failed to create OtaUpdater");
        OTA_ACTIVE.store(false, Ordering::Relaxed);
        return Err(());
    };
    let Ok((mut partition, part_type)) = updater.next_partition() else {
        println!("flash_actor: failed to get next partition");
        OTA_ACTIVE.store(false, Ordering::Relaxed);
        return Err(());
    };
    println!("flash_actor: target partition {:?}", part_type);

    // Erase the partition before writing. NOR flash requires erase (sets bits to 1)
    // before write (can only flip 1→0). Round up to erase block size.
    #[allow(clippy::cast_possible_truncation)] // ESP32 is 32-bit; partition size fits in u32
    let part_capacity = partition.capacity() as u32;
    // Most ESP flash backends operate on 4 KiB sectors.
    const ERASE_SECTOR: u32 = 4096;
    // Erase only what's needed for the incoming image (rounded up), but never more than the
    // partition capacity.
    let erase_size = expected_size
        .saturating_add(ERASE_SECTOR - 1)
        / ERASE_SECTOR
        * ERASE_SECTOR;
    let erase_size = erase_size.min(part_capacity);
    println!(
        "flash_actor: erasing partition ({} bytes, capacity {} bytes)...",
        erase_size, part_capacity
    );
    if let Err(e) = partition.erase(0, erase_size) {
        println!(
            "flash_actor: partition erase failed (kind={:?})",
            e.kind()
        );
        OTA_ACTIVE.store(false, Ordering::Relaxed);
        return Err(());
    }
    println!("flash_actor: partition erased, ready to write");

    // Many ESP flash backends require 4-byte aligned writes. Stream chunks from the OTA task
    // may be arbitrarily sized, so we buffer the tail and only write aligned blocks.
    const ALIGN: usize = 4;
    const ALIGN_U32: u32 = 4;
    let mut written: u32 = 0;
    let mut received: u32 = 0;
    let mut tail = [0xFFu8; ALIGN];
    let mut tail_len: usize = 0;

    loop {
        match ota_rx.receive().await {
            OtaMsg::Data { bytes } => {
                let data = bytes.as_slice();
                received = received.saturating_add(u32::try_from(data.len()).unwrap_or(u32::MAX));

                let mut idx = 0usize;

                // Complete a partial word from the previous chunk, if any.
                if tail_len != 0 {
                    let need = ALIGN - tail_len;
                    let take = need.min(data.len());
                    tail[tail_len..tail_len + take].copy_from_slice(&data[..take]);
                    tail_len += take;
                    idx += take;

                    if tail_len == ALIGN {
                        if let Err(e) = partition.write(written, &tail) {
                            println!(
                                "flash_actor: ota write failed (tail, kind={:?})",
                                e.kind()
                            );
                            OTA_ACTIVE.store(false, Ordering::Relaxed);
                            return Err(());
                        }
                        written = written.saturating_add(ALIGN_U32);
                        tail_len = 0;
                        tail.fill(0xFF);
                    }
                }

                // Write aligned bulk from the remainder.
                let rem = &data[idx..];
                let aligned_len = rem.len() & !(ALIGN - 1);
                if aligned_len != 0 {
                    if let Err(e) = partition.write(written, &rem[..aligned_len]) {
                        println!(
                            "flash_actor: ota write failed (bulk, kind={:?})",
                            e.kind()
                        );
                        OTA_ACTIVE.store(false, Ordering::Relaxed);
                        return Err(());
                    }
                    written = written.saturating_add(u32::try_from(aligned_len).unwrap_or(u32::MAX));
                }

                // Keep any trailing bytes for the next chunk.
                let tail_bytes = &rem[aligned_len..];
                if !tail_bytes.is_empty() {
                    tail[..tail_bytes.len()].copy_from_slice(tail_bytes);
                    tail_len = tail_bytes.len();
                }
            }
            OtaMsg::Finish => {
                // Pad and write the final partial word (erased flash is 0xFF).
                if tail_len != 0 {
                    for b in &mut tail[tail_len..] {
                        *b = 0xFF;
                    }
                    if let Err(e) = partition.write(written, &tail) {
                        println!(
                            "flash_actor: ota write failed (final tail, kind={:?})",
                            e.kind()
                        );
                        OTA_ACTIVE.store(false, Ordering::Relaxed);
                        return Err(());
                    }
                    written = written.saturating_add(ALIGN_U32);
                    tail.fill(0xFF);
                }

                println!(
                    "flash_actor: ota finish (received {} bytes, written {} bytes)",
                    received, written
                );

                if updater.activate_next_partition().is_err()
                    || updater.set_current_ota_state(OtaImageState::New).is_err()
                {
                    println!("flash_actor: ota finalize failed");
                    OTA_ACTIVE.store(false, Ordering::Relaxed);
                    return Err(());
                }

                OTA_ACTIVE.store(false, Ordering::Relaxed);
                esp_hal::system::software_reset();
            }
            OtaMsg::Abort => {
                println!("flash_actor: ota abort");
                OTA_ACTIVE.store(false, Ordering::Relaxed);
                return Err(());
            }
            OtaMsg::Begin { expected_size } => {
                println!(
                    "flash_actor: ota begin received during ota; restarting (expected {})",
                    expected_size
                );
                // Restart by returning; caller will re-enter ota_session with new Begin.
                OTA_ACTIVE.store(false, Ordering::Relaxed);
                return Err(());
            }
        }
    }
}


