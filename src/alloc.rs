use std::ffi::{c_void, CString};

pub type Allocator = tikv_jemallocator::Jemalloc;

pub const fn allocator() -> Allocator {
    Allocator {}
}

pub fn set_jemalloc_param<T>(name: &str, mut value: T) {
    let name_buffer = CString::new(name).unwrap();

    let res = unsafe {
        tikv_jemalloc_sys::mallctl(
            name_buffer.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut value as *mut T as *mut c_void,
            std::mem::size_of::<T>(),
        )
    };

    if res != 0 {
        log::error!("Failed to set {name}: {}", errno::Errno(res));
    }
}

#[cfg(all(feature = "alloc-profiling", unix))]
pub mod profiling {
    use std::ffi::CString;
    use std::os::raw::c_char;
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    pub use tikv_jemalloc_ctl::Error;
    use tikv_jemalloc_ctl::{epoch, stats};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct JemallocStats {
        pub allocated: u64,
        pub active: u64,
        pub metadata: u64,
        pub resident: u64,
        pub mapped: u64,
        pub retained: u64,
        pub dirty: u64,
        pub fragmentation: u64,
    }

    pub fn fetch_stats() -> Result<JemallocStats, Error> {
        // Stats are cached. Need to advance epoch to refresh.
        epoch::advance()?;

        Ok(JemallocStats {
            allocated: stats::allocated::read()? as u64,
            active: stats::active::read()? as u64,
            metadata: stats::metadata::read()? as u64,
            resident: stats::resident::read()? as u64,
            mapped: stats::mapped::read()? as u64,
            retained: stats::retained::read()? as u64,
            dirty: (stats::resident::read()? - stats::active::read()? - stats::metadata::read()?)
                as u64,
            fragmentation: (stats::active::read()? - stats::allocated::read()?) as u64,
        })
    }

    const PROF_ACTIVE: &[u8] = b"prof.active\0";
    const PROF_DUMP: &[u8] = b"prof.dump\0";

    pub fn start() -> Result<(), Error> {
        log::info!("starting profiler");
        unsafe { tikv_jemalloc_ctl::raw::update(PROF_ACTIVE, true)? };
        Ok(())
    }

    pub fn stop() -> Result<(), Error> {
        log::info!("stopping profiler");
        unsafe { tikv_jemalloc_ctl::raw::update(PROF_ACTIVE, false)? };
        Ok(())
    }

    /// Dump the profile to the `path`.
    pub fn dump<P>(path: P) -> Result<(), DumpError>
    where
        P: AsRef<Path>,
    {
        let mut bytes = CString::new(path.as_ref().as_os_str().as_bytes())
            .map_err(|_| DumpError::InvalidPath)?
            .into_bytes_with_nul();

        let ptr = bytes.as_mut_ptr() as *mut c_char;
        let res = unsafe { tikv_jemalloc_ctl::raw::write(PROF_DUMP, ptr) };
        match res {
            Ok(_) => {
                log::info!("saved the profiling dump to {:?}", path.as_ref());
                Ok(())
            }
            Err(e) => {
                log::error!(
                    "failed to dump the profiling info to {:?}: {e:?}",
                    path.as_ref()
                );
                Err(DumpError::JemallocError(e.to_string()))
            }
        }
    }

    #[derive(thiserror::Error, Debug)]
    pub enum DumpError {
        #[error("invalid path to the dump")]
        InvalidPath,
        #[error("failed to dump the profiling info: {0}")]
        JemallocError(String),
    }
}

// run with `MALLOC_CONF="prof:true" cargo test --ignored`
#[cfg(all(test, feature = "alloc-profiling"))]
mod test {
    const OPT_PROF: &[u8] = b"opt.prof\0";

    fn is_profiling_on() -> bool {
        match unsafe { tikv_jemalloc_ctl::raw::read(OPT_PROF) } {
            Err(e) => {
                // Shouldn't be possible since mem-profiling is set
                panic!("is_profiling_on: {e:?}");
            }
            Ok(prof) => prof,
        }
    }

    #[test]
    #[ignore]
    fn test_profile() {
        use super::*;
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("profile.txt");

        profiling::start().unwrap();
        assert!(is_profiling_on());

        profiling::dump(path.as_path()).unwrap();
        profiling::stop().unwrap();
    }
}
