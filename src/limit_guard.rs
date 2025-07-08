//! OS resource limit guard. Sets CPU/Memory/FD caps for the current process.
//!
//! Unix impl uses `rlimit` crate; Windows impl uses Job Objects (windows crate).
//! If corresponding feature flags are absent or the platform is unsupported, no-op
//! stubs are used.
#[derive(Debug, Default)]
pub struct LimitGuard;

impl LimitGuard {
    /// Apply limits. Each parameter = `Some(limit)` to set, `None` to leave unchanged.
    pub fn apply(cpu_seconds: Option<u64>, memory_bytes: Option<u64>, open_files: Option<u64>) -> anyhow::Result<()> {
        #[cfg(all(unix, feature = "limit_guard_unix"))]
        {
            use rlimit::{setrlimit, Resource};
            if let Some(sec) = cpu_seconds {
                setrlimit(Resource::CPU, sec, sec)?;
            }
            if let Some(mem) = memory_bytes {
                // RLIMIT_AS limits virtual memory in bytes.
                setrlimit(Resource::AS, mem, mem)?;
            }
            if let Some(fd) = open_files {
                setrlimit(Resource::NOFILE, fd, fd)?;
            }
        }
        #[cfg(all(windows, feature = "limit_guard_windows"))]
        {
            use windows::Win32::System::JobObjects::*;
            use windows::Win32::Foundation::*;
            use windows::Win32::System::Threading::GetCurrentProcess;
            unsafe {
                let job = CreateJobObjectW(None, None);
                if job.is_invalid() {
                    return Err(anyhow::anyhow!("CreateJobObject failed"));
                }
                let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
                info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_JOB_MEMORY_HIGH | JOB_OBJECT_LIMIT_PROCESS_MEMORY_HIGH;
                if let Some(mem) = memory_bytes {
                    info.JobMemoryLimit = mem as usize;
                    info.ProcessMemoryLimit = mem as usize;
                }
                let res = SetInformationJobObject(job, JobObjectExtendedLimitInformation, &info as *const _ as *const _, std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32);
                if res.as_bool() == false {
                    return Err(anyhow::anyhow!("SetInformationJobObject failed"));
                }
                let res = AssignProcessToJobObject(job, GetCurrentProcess());
                if res.as_bool() == false {
                    return Err(anyhow::anyhow!("AssignProcessToJobObject failed"));
                }
            }
        }
        // Other platforms or no feature: silently ignore.
        let _ = cpu_seconds;
        let _ = memory_bytes;
        let _ = open_files;
        Ok(())
    }
} 