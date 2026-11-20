use windows::core::{Interface, HSTRING};
use windows::Win32::Foundation::{CloseHandle, HANDLE, RPC_E_CHANGED_MODE};
use windows::Win32::Globalization::GetOEMCP;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, IPersistFile, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

use crate::shortcut::ShortcutSpec;

struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.0) };
    }
}

fn exe_name(entry: &PROCESSENTRY32W) -> String {
    let length = entry.szExeFile.iter().position(|&unit| unit == 0).unwrap_or(entry.szExeFile.len());
    String::from_utf16_lossy(&entry.szExeFile[..length])
}

pub fn oem_code_page() -> u32 {
    unsafe { GetOEMCP() }
}

pub fn process_ids_named(process_name: &str) -> Vec<u32> {
    let target = process_name.to_lowercase();
    let Ok(snapshot) = (unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }) else {
        return Vec::new();
    };
    let snapshot = OwnedHandle(snapshot);
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut ids = Vec::new();
    let mut found = unsafe { Process32FirstW(snapshot.0, &mut entry) }.is_ok();
    while found {
        if exe_name(&entry).to_lowercase() == target {
            ids.push(entry.th32ProcessID);
        }
        found = unsafe { Process32NextW(snapshot.0, &mut entry) }.is_ok();
    }
    ids
}

pub fn is_process_running(process_name: &str) -> bool {
    !process_ids_named(process_name).is_empty()
}

pub fn terminate_processes_named(process_name: &str) -> bool {
    terminate_processes(&process_ids_named(process_name))
}

pub fn terminate_processes(ids: &[u32]) -> bool {
    let mut terminated = false;
    for &id in ids {
        let Ok(handle) = (unsafe { OpenProcess(PROCESS_TERMINATE, false, id) }) else {
            continue;
        };
        let handle = OwnedHandle(handle);
        terminated |= unsafe { TerminateProcess(handle.0, 0) }.is_ok();
    }
    terminated
}

struct ComApartment;

impl ComApartment {
    fn enter() -> Option<Self> {
        let result = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        if result.is_ok() {
            Some(ComApartment)
        } else if result == RPC_E_CHANGED_MODE {
            None
        } else {
            log::warn!("CoInitializeEx failed: {result:?}");
            None
        }
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

pub fn create_shortcut(spec: &ShortcutSpec) -> windows::core::Result<()> {
    let _apartment = ComApartment::enter();
    let link: IShellLinkW = unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)? };
    unsafe {
        link.SetPath(&HSTRING::from(spec.target.as_os_str()))?;
        link.SetArguments(&HSTRING::from(spec.arguments.as_str()))?;
        link.SetWorkingDirectory(&HSTRING::from(spec.working_dir.as_os_str()))?;
        link.SetIconLocation(&HSTRING::from(spec.icon.as_os_str()), 0)?;
        link.SetDescription(&HSTRING::from(spec.description.as_str()))?;
    }
    let persist: IPersistFile = link.cast()?;
    unsafe { persist.Save(&HSTRING::from(spec.shortcut_path.as_os_str()), true) }
}
