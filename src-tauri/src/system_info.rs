use crate::search::SearchResult;
use crate::indexer::{SearchItem, ItemType};
use std::sync::OnceLock;
use std::sync::Mutex;
use std::time::Instant;

// ── RAM structures ──────────────────────────────────────────────────────────
#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct MEMORYSTATUSEX {
    dw_length: u32,
    dw_memory_load: u32,
    ull_total_phys: u64,
    ull_avail_phys: u64,
    ull_total_page_file: u64,
    ull_avail_page_file: u64,
    ull_total_virtual: u64,
    ull_avail_virtual: u64,
    ull_avail_extended_arch_mem: u64,
}

// ── CPU structures ──────────────────────────────────────────────────────────
#[repr(C)]
#[derive(Copy, Clone)]
struct FILETIME {
    dw_low_date_time: u32,
    dw_high_date_time: u32,
}

#[link(name = "kernel32")]
extern "system" {
    fn GlobalMemoryStatusEx(lpBuffer: *mut MEMORYSTATUSEX) -> i32;
    fn GetSystemTimes(
        lpIdleTime: *mut FILETIME,
        lpKernelTime: *mut FILETIME,
        lpUserTime: *mut FILETIME,
    ) -> i32;
}

// Cache structure to prevent flickering CPU usage on fast typing
struct CpuCache {
    last_time: Instant,
    last_idle: u64,
    last_sys: u64,
    last_calculated: u32,
}

static CPU_USAGE_CACHE: OnceLock<Mutex<Option<CpuCache>>> = OnceLock::new();

fn get_cpu_cache() -> &'static Mutex<Option<CpuCache>> {
    CPU_USAGE_CACHE.get_or_init(|| Mutex::new(None))
}

fn filetime_to_u64(ft: &FILETIME) -> u64 {
    ((ft.dw_high_date_time as u64) << 32) | (ft.dw_low_date_time as u64)
}

pub fn get_ram_usage() -> Option<SearchResult> {
    let mut mem_info = MEMORYSTATUSEX {
        dw_length: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        dw_memory_load: 0,
        ull_total_phys: 0,
        ull_avail_phys: 0,
        ull_total_page_file: 0,
        ull_avail_page_file: 0,
        ull_total_virtual: 0,
        ull_avail_virtual: 0,
        ull_avail_extended_arch_mem: 0,
    };

    let success = unsafe { GlobalMemoryStatusEx(&mut mem_info) };
    if success == 0 {
        return None;
    }

    let total_gb = mem_info.ull_total_phys as f64 / (1024.0 * 1024.0 * 1024.0);
    let avail_gb = mem_info.ull_avail_phys as f64 / (1024.0 * 1024.0 * 1024.0);
    let used_gb = total_gb - avail_gb;

    let display = format!(
        "RAM Usage: {}% ({:.2} GB / {:.2} GB)",
        mem_info.dw_memory_load, used_gb, total_gb
    );

    let synthetic = SearchItem::new(
        display.clone(),
        String::new(),
        Some("cpu".to_string()),
        ItemType::File,
        "COMMAND".to_string(),
    );

    Some(SearchResult {
        item: synthetic,
        inline_display: Some(display),
    })
}

pub fn get_cpu_usage() -> Option<SearchResult> {
    let mut idle_ft = FILETIME { dw_low_date_time: 0, dw_high_date_time: 0 };
    let mut kernel_ft = FILETIME { dw_low_date_time: 0, dw_high_date_time: 0 };
    let mut user_ft = FILETIME { dw_low_date_time: 0, dw_high_date_time: 0 };

    let success = unsafe { GetSystemTimes(&mut idle_ft, &mut kernel_ft, &mut user_ft) };
    if success == 0 {
        return None;
    }

    let idle = filetime_to_u64(&idle_ft);
    let kernel = filetime_to_u64(&kernel_ft);
    let user = filetime_to_u64(&user_ft);
    let sys = kernel.saturating_add(user);

    let lock = get_cpu_cache();
    let mut guard = lock.lock().unwrap();
    let now = Instant::now();

    let mut cpu_percent = 0u32;

    if let Some(ref mut cache) = *guard {
        // If query occurs within 300ms, return cached calculation to prevent zero values
        if now.duration_since(cache.last_time).as_millis() < 300 {
            cpu_percent = cache.last_calculated;
        } else {
            let idle_diff = idle.saturating_sub(cache.last_idle);
            let sys_diff = sys.saturating_sub(cache.last_sys);

            if sys_diff > 0 {
                let usage = 100.0 - (idle_diff as f64 / sys_diff as f64 * 100.0);
                cpu_percent = usage.clamp(0.0, 100.0) as u32;
            }
            cache.last_time = now;
            cache.last_idle = idle;
            cache.last_sys = sys;
            cache.last_calculated = cpu_percent;
        }
    } else {
        *guard = Some(CpuCache {
            last_time: now,
            last_idle: idle,
            last_sys: sys,
            last_calculated: 0,
        });
    }

    let display = format!("CPU Load: {}%", cpu_percent);

    let synthetic = SearchItem::new(
        display.clone(),
        String::new(),
        Some("activity".to_string()),
        ItemType::File,
        "COMMAND".to_string(),
    );

    Some(SearchResult {
        item: synthetic,
        inline_display: Some(display),
    })
}

pub fn get_disk_spaces() -> Vec<SearchResult> {
    use windows::Win32::Storage::FileSystem::GetLogicalDrives;

    let mut results = Vec::new();
    let drive_mask = unsafe { GetLogicalDrives() };
    
    for i in 0..26 {
        if (drive_mask & (1 << i)) != 0 {
            let letter = (b'A' + i as u8) as char;
            let drive = format!("{}:\\", letter);
            let drive_wide: Vec<u16> = drive.encode_utf16().chain(Some(0)).collect();
            
            let mut free_bytes_available = 0u64;
            let mut total_number_of_bytes = 0u64;
            let mut total_number_of_free_bytes = 0u64;

            let success = unsafe {
                windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
                    windows::core::PCWSTR(drive_wide.as_ptr()),
                    Some(&mut free_bytes_available),
                    Some(&mut total_number_of_bytes),
                    Some(&mut total_number_of_free_bytes),
                )
            };

            if success.is_ok() && total_number_of_bytes > 0 {
                let total_gb = total_number_of_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                let free_gb = free_bytes_available as f64 / (1024.0 * 1024.0 * 1024.0);
                let used_gb = total_gb - free_gb;
                let percent_used = (used_gb / total_gb * 100.0) as u32;

                let display = format!(
                    "Disk ({}): {}% Used ({:.1} GB Free of {:.1} GB)",
                    drive.trim_end_matches('\\'), percent_used, free_gb, total_gb
                );

                let synthetic = SearchItem::new(
                    display.clone(),
                    String::new(),
                    Some("hard-drive".to_string()),
                    ItemType::File,
                    "COMMAND".to_string(),
                );

                results.push(SearchResult {
                    item: synthetic,
                    inline_display: Some(display),
                });
            }
        }
    }
    results
}

pub fn get_system_stats() -> Vec<SearchResult> {
    let mut stats = Vec::new();
    if let Some(cpu) = get_cpu_usage() {
        stats.push(cpu);
    }
    if let Some(ram) = get_ram_usage() {
        stats.push(ram);
    }
    stats.append(&mut get_disk_spaces());
    stats
}
